use clap::Parser;
use linux_crng_ioctl::ioctl::{add_randomness_to_kernel, force_kernel_crng_reseed};
use log::{debug, error, info};
use rand::{RngCore, TryRngCore};
use rand_jitterentropy::RandJitterEntropy;
use sha3::{Digest, Sha3_512};
use std::{process::ExitCode, time::Duration};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ToolArgs {
    #[arg(short, long, default_value_t = false)]
    oneshot: bool,

    #[arg(short, long, default_value_t = 10)]
    seed_interval_s: u64,

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

    let mut rngs: Vec<Box<dyn RngCore>> = vec![Box::new(
        match RandJitterEntropy::new() {
            Ok(rng) => rng,
            Err(e) => {
                error!("Failed to create jitterentropy instance: {}", e);
                return ExitCode::FAILURE;
            }
        }
        .unwrap_err(),
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

        let copy_len = &state.0.len();
        state.0.copy_from_slice(&state_out[0..*copy_len]);

        let copy_len = &output.0.len();
        output.0.copy_from_slice(&output_out[0..*copy_len]);

        debug!("Gathered entropy and hashed to buf!");

        add_randomness_to_kernel(&output.0, u32::try_from(output.0.len() * 8).unwrap()).unwrap();

        if args.force_crng_reseed {
            force_kernel_crng_reseed().unwrap();
        }

        if args.oneshot {
            break;
        }

        std::thread::sleep(Duration::from_secs(args.seed_interval_s));
    }

    ExitCode::SUCCESS
}
