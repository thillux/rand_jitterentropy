use clap::Parser;
use linux_crng_ioctl::ioctl::{add_randomness_to_kernel, force_kernel_crng_reseed};
use log::{debug, error, info, warn};
use nix::{
    poll::{PollFd, PollFlags, PollTimeout, poll},
    sys::{
        signal::{SigSet, Signal},
        signalfd::{SfdFlags, SignalFd},
    },
};
use rand_core::TryRng;
use rand_jitterentropy::{
    JENT_CACHE_ALL, JENT_DISABLE_INTERNAL_TIMER, JENT_DISABLE_MEMORY_ACCESS, JENT_FORCE_FIPS,
    JENT_FORCE_INTERNAL_TIMER, JENT_HASHLOOP_1, JENT_HASHLOOP_2, JENT_HASHLOOP_4, JENT_HASHLOOP_8,
    JENT_HASHLOOP_16, JENT_HASHLOOP_32, JENT_HASHLOOP_64, JENT_HASHLOOP_128, JENT_MAX_MEMSIZE_1MB,
    JENT_MAX_MEMSIZE_1kB, JENT_MAX_MEMSIZE_2MB, JENT_MAX_MEMSIZE_2kB, JENT_MAX_MEMSIZE_4MB,
    JENT_MAX_MEMSIZE_4kB, JENT_MAX_MEMSIZE_8MB, JENT_MAX_MEMSIZE_8kB, JENT_MAX_MEMSIZE_16MB,
    JENT_MAX_MEMSIZE_16kB, JENT_MAX_MEMSIZE_32MB, JENT_MAX_MEMSIZE_32kB, JENT_MAX_MEMSIZE_64MB,
    JENT_MAX_MEMSIZE_64kB, JENT_MAX_MEMSIZE_128MB, JENT_MAX_MEMSIZE_128kB, JENT_MAX_MEMSIZE_256MB,
    JENT_MAX_MEMSIZE_256kB, JENT_MAX_MEMSIZE_512MB, JENT_MAX_MEMSIZE_512kB, JENT_NTG1,
    JitterEntropyError, RandJitterEntropy,
};
use std::{
    os::fd::AsFd,
    process::ExitCode,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[cfg(feature = "egd")]
mod egd;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    after_help = "Signals: SIGUSR1 seeds and forces a kernel CRNG reseed right away, \
                  SIGUSR2 prints the status of the Jitter RNG."
)]
#[allow(clippy::struct_excessive_bools)]
struct ToolArgs {
    /// Seed the kernel CRNG once, then exit
    #[arg(short, long, default_value_t = false)]
    oneshot: bool,

    /// Seconds to sleep between seeding rounds
    #[arg(short, long, default_value_t = 10)]
    seed_interval_s: u64,

    /// Force a kernel CRNG reseed after each seeding round
    #[arg(short, long, default_value_t = false)]
    force_crng_reseed: bool,

    /// Print the status of the Jitter RNG as JSON, then exit
    #[arg(long, default_value_t = false)]
    status: bool,

    /// Oversampling rate
    #[arg(long, default_value_t = RandJitterEntropy::DEFAULT_OSR)]
    osr: u32,

    /// FIPS mode: SP800-90B compliance, health test failures stop the output
    #[arg(long, default_value_t = false)]
    fips: bool,

    /// AIS 20/31 NTG.1 compliance, implies --fips
    #[arg(long, default_value_t = false)]
    ntg1: bool,

    /// Maximum memory size for the memory access, 1kB ... 512MB
    #[arg(long, value_parser = parse_max_memsize)]
    max_memsize: Option<u32>,

    /// Size the memory access by all caches instead of only L1
    #[arg(long, default_value_t = false)]
    cache_all: bool,

    /// Disable the memory access
    #[arg(long, default_value_t = false)]
    disable_memory_access: bool,

    /// Hash loop count per generated block, 1 ... 128
    #[arg(long, value_parser = parse_hashloop)]
    hashloop: Option<u32>,

    /// Force the internal timer
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "disable_internal_timer"
    )]
    force_internal_timer: bool,

    /// Disable the internal timer
    #[arg(long, default_value_t = false)]
    disable_internal_timer: bool,

    /// Serve the EGD protocol on this Unix socket, every client with its own
    /// entropy collector configured as above
    #[cfg(feature = "egd")]
    #[arg(long, conflicts_with_all = ["oneshot", "status"])]
    egd_socket: Option<std::path::PathBuf>,

    /// Do not seed the kernel CRNG, only serve EGD; needs no root
    #[cfg(feature = "egd")]
    #[arg(
        long,
        default_value_t = false,
        requires = "egd_socket",
        conflicts_with = "force_crng_reseed"
    )]
    no_kernel: bool,
}

impl ToolArgs {
    fn jent_flags(&self) -> u32 {
        [
            (self.fips, JENT_FORCE_FIPS),
            (self.ntg1, JENT_NTG1),
            (self.cache_all, JENT_CACHE_ALL),
            (self.disable_memory_access, JENT_DISABLE_MEMORY_ACCESS),
            (self.force_internal_timer, JENT_FORCE_INTERNAL_TIMER),
            (self.disable_internal_timer, JENT_DISABLE_INTERNAL_TIMER),
        ]
        .into_iter()
        .filter(|(set, _)| *set)
        .fold(
            self.max_memsize.unwrap_or(0) | self.hashloop.unwrap_or(0),
            |flags, (_, flag)| flags | flag,
        )
    }
}

fn parse_max_memsize(arg: &str) -> Result<u32, String> {
    const SIZES: [(&str, u32); 20] = [
        ("1kB", JENT_MAX_MEMSIZE_1kB),
        ("2kB", JENT_MAX_MEMSIZE_2kB),
        ("4kB", JENT_MAX_MEMSIZE_4kB),
        ("8kB", JENT_MAX_MEMSIZE_8kB),
        ("16kB", JENT_MAX_MEMSIZE_16kB),
        ("32kB", JENT_MAX_MEMSIZE_32kB),
        ("64kB", JENT_MAX_MEMSIZE_64kB),
        ("128kB", JENT_MAX_MEMSIZE_128kB),
        ("256kB", JENT_MAX_MEMSIZE_256kB),
        ("512kB", JENT_MAX_MEMSIZE_512kB),
        ("1MB", JENT_MAX_MEMSIZE_1MB),
        ("2MB", JENT_MAX_MEMSIZE_2MB),
        ("4MB", JENT_MAX_MEMSIZE_4MB),
        ("8MB", JENT_MAX_MEMSIZE_8MB),
        ("16MB", JENT_MAX_MEMSIZE_16MB),
        ("32MB", JENT_MAX_MEMSIZE_32MB),
        ("64MB", JENT_MAX_MEMSIZE_64MB),
        ("128MB", JENT_MAX_MEMSIZE_128MB),
        ("256MB", JENT_MAX_MEMSIZE_256MB),
        ("512MB", JENT_MAX_MEMSIZE_512MB),
    ];

    SIZES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(arg))
        .map(|(_, flag)| *flag)
        .ok_or_else(|| "expected a power of two from 1kB to 512MB".to_string())
}

fn parse_hashloop(arg: &str) -> Result<u32, String> {
    match arg.parse::<u32>() {
        Ok(1) => Ok(JENT_HASHLOOP_1),
        Ok(2) => Ok(JENT_HASHLOOP_2),
        Ok(4) => Ok(JENT_HASHLOOP_4),
        Ok(8) => Ok(JENT_HASHLOOP_8),
        Ok(16) => Ok(JENT_HASHLOOP_16),
        Ok(32) => Ok(JENT_HASHLOOP_32),
        Ok(64) => Ok(JENT_HASHLOOP_64),
        Ok(128) => Ok(JENT_HASHLOOP_128),
        _ => Err("expected a power of two from 1 to 128".to_string()),
    }
}

const SEED_SIZE_BYTE: usize = 64;

/// The instance cannot produce output anymore, as opposed to an intermittent
/// health test failure, after which jitterentropy takes itself up again.
fn is_permanent(e: JitterEntropyError) -> bool {
    matches!(
        e,
        JitterEntropyError::RctPermanentFailure
            | JitterEntropyError::AptPermanentFailure
            | JitterEntropyError::LagPermanentFailure
            | JitterEntropyError::RctMemPermanentFailure
            | JitterEntropyError::NullCollector
            | JitterEntropyError::ProgErr
    )
}

fn print_status(rng: &RandJitterEntropy) -> bool {
    match rng.status() {
        Ok(status) => {
            println!("{status}");
            true
        }
        Err(e) => {
            error!("Failed to get jitterentropy status: {e}");
            false
        }
    }
}

/// Feed one block of Jitter RNG output, as is, into the kernel. Returns false
/// if the daemon has to stop.
fn seed_kernel(rng: &mut RandJitterEntropy, reseed: bool, oneshot: bool) -> bool {
    let mut seed = Zeroizing::new([0u8; SEED_SIZE_BYTE]);

    match rng.try_fill_bytes(seed.as_mut()) {
        Ok(()) => debug!("Gathered {SEED_SIZE_BYTE} bytes from jitterentropy"),
        Err(e) if is_permanent(e) || oneshot => {
            error!("jitterentropy failed: {e}");
            print_status(rng);
            return false;
        }
        Err(e) => {
            warn!("jitterentropy failed, skipping this round: {e}");
            return true;
        }
    }

    let ent_bits = u32::try_from(SEED_SIZE_BYTE * 8).unwrap();
    if let Err(e) = add_randomness_to_kernel(seed.as_ref(), ent_bits) {
        error!("Failed to add randomness to kernel: {e}");
        return false;
    }

    if reseed && let Err(e) = force_kernel_crng_reseed() {
        error!("Failed to force kernel CRNG reseed: {e}");
        return false;
    }

    true
}

/// Sleep for the seed interval, printing the status on SIGUSR2. Returns early
/// with true on SIGUSR1, which asks for a forced reseed.
fn wait_for_round(signals: &SignalFd, interval: Duration, rng: &RandJitterEntropy) -> bool {
    let deadline = Instant::now() + interval;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }

        let timeout = PollTimeout::try_from(remaining).unwrap_or(PollTimeout::MAX);
        let mut fds = [PollFd::new(signals.as_fd(), PollFlags::POLLIN)];
        match poll(&mut fds, timeout) {
            Ok(0) | Err(nix::errno::Errno::EINTR) => continue,
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to wait for signals: {e}");
                std::thread::sleep(remaining);
                return false;
            }
        }

        let Ok(Some(info)) = signals.read_signal() else {
            continue;
        };
        match i32::try_from(info.ssi_signo).map(Signal::try_from) {
            Ok(Ok(Signal::SIGUSR1)) => return true,
            Ok(Ok(Signal::SIGUSR2)) => {
                print_status(rng);
            }
            _ => {}
        }
    }
}

fn main() -> ExitCode {
    env_logger::init();

    let args = ToolArgs::parse();
    let flags = args.jent_flags();

    // block the signals before jitterentropy may start its timer thread, which
    // inherits the mask, so that they only arrive through the signalfd
    let mut mask = SigSet::empty();
    mask.add(Signal::SIGUSR1);
    mask.add(Signal::SIGUSR2);
    let signals = match mask
        .thread_block()
        .and_then(|()| SignalFd::with_flags(&mask, SfdFlags::SFD_CLOEXEC))
    {
        Ok(signals) => signals,
        Err(e) => {
            error!("Failed to set up signal handling: {e}");
            return ExitCode::FAILURE;
        }
    };

    info!(
        "Starting jitter-rngd with jitterentropy {}, osr {}, flags {flags:#x}",
        RandJitterEntropy::version_pretty(),
        args.osr
    );
    if !args.fips && !args.ntg1 {
        warn!("Neither --fips nor --ntg1 given: health test failures do not stop the output");
    }

    let mut rng = match RandJitterEntropy::with_osr_and_flags(args.osr, flags) {
        Ok(rng) => rng,
        Err(e) => {
            error!("Failed to create jitterentropy instance: {e}");
            return ExitCode::FAILURE;
        }
    };

    if args.status {
        return if print_status(&rng) {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    #[cfg(feature = "egd")]
    if let Some(path) = &args.egd_socket
        && let Err(e) = egd::spawn(path, args.osr, flags)
    {
        error!("Failed to serve EGD on {}: {e}", path.display());
        return ExitCode::FAILURE;
    }

    #[cfg(feature = "egd")]
    let seeds_kernel = !args.no_kernel;
    #[cfg(not(feature = "egd"))]
    let seeds_kernel = true;

    if !seeds_kernel {
        info!("Not seeding the kernel CRNG, serving EGD only");
        loop {
            if wait_for_round(&signals, Duration::from_secs(3600), &rng) {
                warn!("SIGUSR1 ignored: not seeding the kernel CRNG");
            }
        }
    }

    let mut reseed = args.force_crng_reseed;
    loop {
        if !seed_kernel(&mut rng, reseed, args.oneshot) {
            return ExitCode::FAILURE;
        }

        if args.oneshot {
            return ExitCode::SUCCESS;
        }

        let interval = Duration::from_secs(args.seed_interval_s);
        let requested = wait_for_round(&signals, interval, &rng);
        if requested {
            info!("SIGUSR1: seeding and forcing a kernel CRNG reseed");
        }
        reseed = requested || args.force_crng_reseed;
    }
}
