use anyhow::{Error, Result, anyhow};
use log::{debug, error};
use std::{fs::File, os::fd::AsRawFd};
use crate::ioctl_defs;

/// Gets the current entropy count from the kernel's random number generator.
///
/// This function reads the entropy count from `/dev/random`, which represents
/// the amount of entropy (in bits) that the kernel estimates is contained in
/// the entropy pool.
///
/// # Returns
/// - `Ok(i32)` - The current entropy count in bits
/// - `Err` - If there's an error accessing the kernel or no file descriptors are available
///
/// # Errors
/// - Returns error if unable to open `/dev/random`
/// - Returns error if the ioctl call to get entropy count fails
/// - Returns error if no more file descriptors are available
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # use linux_crng_ioctl::ioctl::get_ent_cnt;
/// # fn main() -> Result<()> {
/// let entropy_count = get_ent_cnt()?;
/// println!("Current entropy count: {} bits", entropy_count);
/// # Ok(())
/// # }
/// ```
pub fn get_ent_cnt() -> Result<i32> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();
    let mut ent_cnt = 0;

    let ret = unsafe { ioctl_defs::rnd_get_ent_cnt(fd, &mut ent_cnt) };
    if let Ok(0) = ret {
        Ok(ent_cnt)
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to fetch entropy level from kernel"))
    }
}

/// Adds to (or subtracts from) the kernel's entropy count estimation.
///
/// This function allows superusers to modify the kernel's entropy estimation.
/// Use with caution as incorrect entropy estimation can impact system security.
///
/// # Arguments
/// * `ent_cnt` - The number of bits to add (positive) or subtract (negative) from the entropy count
///
/// # Returns
/// - `Ok(())` - If the operation was successful
/// - `Err` - If there's an error accessing the kernel or insufficient permissions
///
/// # Errors
/// - Returns error if not running with root privileges
/// - Returns error if unable to open `/dev/random`
/// - Returns error if the ioctl call to modify entropy count fails
/// - Returns error if no more file descriptors are available
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// use linux_crng_ioctl::ioctl::add_to_ent_cnt;
/// # fn main() -> Result<()> {
/// add_to_ent_cnt(32)?; // Add 32 bits to entropy count
/// # Ok(())
/// # }
/// ```
///
/// # Security
/// Requires root privileges to execute successfully.
pub fn add_to_ent_cnt(ent_cnt: i32) -> Result<()> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    let ret = unsafe { ioctl_defs::rnd_add_to_ent_cnt(fd, &ent_cnt) };
    if let Ok(0) = ret {
        Ok(())
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to add to ent cnt"))
    }
}

/// Adds random data to the kernel's entropy pool.
///
/// This function allows adding entropy to the kernel's random number generator.
/// The entropy estimation must not exceed the actual entropy of the input data.
///
/// # Arguments
/// * `entropy` - Byte slice containing the random data to add
/// * `ent_bits` - Number of bits of entropy claimed to be in the data
///
/// # Returns
/// - `Ok(())` - If the entropy was successfully added
/// - `Err` - If there's an error accessing the kernel or insufficient permissions
///
/// # Errors
/// - Returns error if not running with root privileges
/// - Returns error if unable to open `/dev/random`
/// - Returns error if `ent_bits` claims more entropy than possible (`buffer_length` * 8)
/// - Returns error if buffer size exceeds `MAX_BUFFER_SIZE` (2048 bytes)
/// - Returns error if the ioctl call to add entropy fails
/// - Returns error if integer conversion fails for buffer size or entropy bits
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # use linux_crng_ioctl::ioctl::add_randomness_to_kernel;
/// # fn main() -> Result<()> {
/// let random_data = [0u8; 64];
/// add_randomness_to_kernel(&random_data, 256)?;
/// # Ok(())
/// # }
/// ```
///
/// # Security
/// - Requires root privileges
/// - Be careful not to overestimate entropy to maintain system security
pub fn add_randomness_to_kernel(entropy: &[u8], ent_bits: u32) -> Result<()> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    if usize::try_from(ent_bits)? > entropy.len() * 8 {
        return Err(anyhow!("Do not claim more entropy than buffer length * 8!"));
    }

    if entropy.len() > ioctl_defs::MAX_BUFFER_SIZE {
        return Err(anyhow!(
            "This implementation currently can write up to {} Byte to kernel CRNG input pool", ioctl_defs::MAX_BUFFER_SIZE
        ));
    }

    debug!(
        "Write {} Byte to /dev/random, accounted with {} Bit entropy",
        64, ent_bits
    );

    let mut pool_info = ioctl_defs::KernelRandPoolInfo {
        header: ioctl_defs::KernelRandPoolInfoHeader {
            entropy_bits: i32::try_from(ent_bits)?,
            buf_size_byte: i32::try_from(entropy.len())?,
        },
        buf: [0; ioctl_defs::MAX_BUFFER_SIZE],
    };
    pool_info.buf[0..entropy.len()].copy_from_slice(entropy);

    #[allow(clippy::ptr_as_ptr)]
    let res = unsafe {
        ioctl_defs::rnd_add_entropy(
            fd,
            std::ptr::addr_of!(pool_info) as *const ioctl_defs::KernelRandPoolInfoHeader,
        )
    };

    if let Ok(0) = res {
        Ok(())
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to add entropy to kernel"))
    }
}

/// Clears the kernel's entropy count to zero.
///
/// This function resets the kernel's entropy estimation without affecting
/// the actual entropy pool contents.
///
/// # Returns
/// - `Ok(())` - If the entropy count was successfully cleared
/// - `Err` - If there's an error accessing the kernel or insufficient permissions
///
/// # Errors
/// - Returns error if not running with root privileges
/// - Returns error if unable to open `/dev/random`
/// - Returns error if the ioctl call to clear entropy count fails
/// - Returns error if no more file descriptors are available
///
/// # Security
/// - Requires root privileges
/// - Use with caution as this affects system-wide entropy estimation
pub fn clear_entropy_count() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { ioctl_defs::rnd_zap_ent_cnt(fd) } {
        Ok(0) => {
            debug!("Cleared kernel CRNG entropy count to 0");
            Ok(())
        }
        _ => Err(anyhow!("Cannot clear CRNG entropy count to 0")),
    }
}

/// Clears the kernel's entropy pool and associated counters.
///
/// This function completely clears both the entropy pool and its estimation.
/// This is a more drastic operation than `clear_entropy_count()`.
///
/// # Returns
/// - `Ok(())` - If the pool was successfully cleared
/// - `Err` - If there's an error accessing the kernel or insufficient permissions
///
/// # Errors
/// - Returns error if not running with root privileges
/// - Returns error if unable to open `/dev/random`
/// - Returns error if the ioctl call to clear the pool fails
/// - Returns error if no more file descriptors are available
///
/// # Security
/// - Requires root privileges
/// - Use with extreme caution as this affects system-wide randomness generation
pub fn clear_pool() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { ioctl_defs::rnd_clear_pool(fd) } {
        Ok(0) => {
            debug!("Forcefully cleared kernel CRNG pool");
            Ok(())
        }
        _ => Err(anyhow!("Cannot clear CRNG pool")),
    }
}

/// Forces the kernel's CRNG (Cryptographic Random Number Generator) to reseed.
///
/// This function triggers an immediate reseed of the CRNG from the entropy pool.
///
/// # Returns
/// - `Ok(())` - If the CRNG was successfully reseeded
/// - `Err` - If there's an error accessing the kernel or insufficient permissions
///
/// # Errors
/// - Returns error if not running with root privileges
/// - Returns error if unable to open `/dev/random`
/// - Returns error if the ioctl call to reseed fails
/// - Returns error if no more file descriptors are available
///
/// # Security
/// - Requires root privileges
pub fn force_kernel_crng_reseed() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { ioctl_defs::rnd_reseed_crng(fd) } {
        Ok(0) => {
            debug!("Forcefully reseeded kernel CRNG");
            Ok(())
        }
        _ => Err(anyhow!("Cannot reseed CRNG")),
    }
}

#[cfg(test)]
mod tests {
    use crate::ioctl_defs;
    use crate::ioctl::{
        add_randomness_to_kernel, add_to_ent_cnt, clear_entropy_count, clear_pool,
        force_kernel_crng_reseed, get_ent_cnt
    };
    use nix::unistd::Uid;

    #[test]
    fn test_get_ent_cnt() {
        assert!(get_ent_cnt().is_ok(), "failed to get entropy count");
    }

    #[test]
    fn test_add_to_entropy_count() {
        if !Uid::effective().is_root() {
            println!("Skipping test: requires root privileges");
            return;
        }

        assert!(
            add_to_ent_cnt(32).is_ok(),
            "failed to add to CRNG entropy count"
        );
    }

    #[test]
    fn test_add_randomness_multiple_buffer_sizes() {
        if !Uid::effective().is_root() {
            println!("Skipping test: requires root privileges");
            return;
        }

        // Test different buffer sizes
        let test_sizes = [
            1,               // Minimum size
            64,              // Small buffer
            512,             // Medium buffer
            1024,            // 1KB
            ioctl_defs::MAX_BUFFER_SIZE, // Maximum allowed size
        ];

        for size in test_sizes {
            let buffer = vec![0x55; size]; // Fill with a test pattern
            let entropy_bits = u32::try_from(size * 8).unwrap(); // Claim maximum possible entropy

            let result = add_randomness_to_kernel(&buffer, entropy_bits);
            assert!(
                result.is_ok(),
                "Failed to add randomness with buffer size {size}: {result:?}"
            );
        }

        // Test error case: buffer larger than MAX_BUFFER_SIZE
        let oversized_buffer = vec![0x55; ioctl_defs::MAX_BUFFER_SIZE + 1];
        let result = add_randomness_to_kernel(&oversized_buffer, 8);
        assert!(
            result.is_err(),
            "{}", format!("Expected error for buffer size larger than {}", ioctl_defs::MAX_BUFFER_SIZE)
        );
    }

    #[test]
    fn test_add_entropy() {
        if !Uid::effective().is_root() {
            println!("Skipping test: requires root privileges");
            return;
        }

        assert!(
            add_randomness_to_kernel(&[0u8; 32], 256).is_ok(),
            "failed to add randomness to kernel"
        );
    }

    #[test]
    fn test_clear_entropy_count() {
        if !Uid::effective().is_root() {
            println!("Skipping test: requires root privileges");
            return;
        }

        assert!(
            clear_entropy_count().is_ok(),
            "failed to clear CRNG entropy count"
        );
    }

    #[test]
    fn test_clear_pool() {
        if Uid::effective().is_root() {
            assert!(clear_pool().is_ok(), "failed to clear CRNG pool");
        }
    }

    #[test]
    fn test_reseed_crng() {
        if !Uid::effective().is_root() {
            println!("Skipping test: requires root privileges");
            return;
        }

        assert!(force_kernel_crng_reseed().is_ok(), "failed to reseed CRNG");
    }
}