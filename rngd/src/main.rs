use clap::Parser;
use linux_crng_ioctl::ioctl::{add_randomness_to_kernel, force_kernel_crng_reseed};
use log::{debug, error, info};
use rand::{rand_core::UnwrapErr, Rng};
use rand_jitterentropy::RandJitterEntropy;
use sha3::{Digest, Sha3_512};
use std::{process::ExitCode, time::Duration};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
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
}

const RNG_STATE_SIZE_BYTE: usize = 64;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct RandomState(pub [u8; RNG_STATE_SIZE_BYTE]);

impl Default for RandomState {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomState {
    #[must_use]
    pub fn new() -> Self {
        RandomState([0; RNG_STATE_SIZE_BYTE])
    }
}

fn main() -> ExitCode {
    env_logger::init();

    let args = ToolArgs::parse();

    info!("Starting jitter-rngd");

    let mut state = RandomState::new();

    let mut rngs: Vec<Box<dyn Rng>> = vec![Box::new(
        UnwrapErr(match RandJitterEntropy::new() {
            Ok(rng) => rng,
            Err(e) => {
                error!("Failed to create jitterentropy instance: {e}");
                return ExitCode::FAILURE;
            }
        }),
    )];

    loop {
        let mut output = RandomState::new();

        let mut hasher_state = Sha3_512::new();
        let mut hasher_output = Sha3_512::new();

        // domain separation
        hasher_state.update("STATE");
        hasher_output.update("RAND0");

        // add previous state back
        hasher_state.update(state.0);
        hasher_output.update(state.0);

        // mix in different rngs
        for rng in &mut rngs {
            rng.fill_bytes(&mut output.0);
            hasher_state.update(output.0);
            hasher_output.update(output.0);
        }

        let output_out = hasher_output.finalize();
        let state_out = hasher_state.finalize();

        state.0.copy_from_slice(&state_out[0..RNG_STATE_SIZE_BYTE]);
        output.0.copy_from_slice(&output_out[0..RNG_STATE_SIZE_BYTE]);

        debug!("Gathered entropy and hashed to buf!");

        let ent_bits = u32::try_from(output.0.len() * 8).unwrap();
        if let Err(e) = add_randomness_to_kernel(&output.0, ent_bits) {
            error!("Failed to add randomness to kernel: {e}");
            return ExitCode::FAILURE;
        }

        if args.force_crng_reseed
            && let Err(e) = force_kernel_crng_reseed()
        {
            error!("Failed to force kernel CRNG reseed: {e}");
            return ExitCode::FAILURE;
        }

        if args.oneshot {
            break;
        }

        std::thread::sleep(Duration::from_secs(args.seed_interval_s));
    }

    ExitCode::SUCCESS
}
