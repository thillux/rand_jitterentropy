use anyhow::Error;
use std::fs::File;
use std::io::Read;

/// Reads the system's boot ID from `/proc/sys/kernel/random/boot_id`.
///
/// The boot ID is a unique identifier that changes each time the system boots.
///
/// # Returns
/// - `Ok(String)` - The boot ID string
/// - `Err` - If there's an error reading the boot ID
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/boot_id`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # use linux_crng_ioctl::proc::boot_id;
/// # fn main() -> Result<()> {
/// let boot_id = boot_id()?;
/// println!("System boot ID: {}", boot_id);
/// # Ok(())
/// # }
/// ```
pub fn boot_id() -> anyhow::Result<String, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/boot_id")?;
    let mut boot_id = String::new();
    proc_file.read_to_string(&mut boot_id)?;
    Ok(boot_id)
}

/// Reads the current available entropy from `/proc/sys/kernel/random/entropy_avail`.
///
/// This value represents the kernel's estimation of available entropy in bits.
///
/// # Returns
/// - `Ok(u32)` - The number of bits of available entropy
/// - `Err` - If there's an error reading the entropy value
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/entropy_avail`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
/// - Returns error if the content cannot be parsed as a u32
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # use linux_crng_ioctl::proc::entropy_avail;
/// # fn main() -> Result<()> {
/// let available_entropy = entropy_avail()?;
/// println!("Available entropy: {} bits", available_entropy);
/// # Ok(())
/// # }
/// ```
pub fn entropy_avail() -> anyhow::Result<u32, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/entropy_avail")?;
    let mut entropy_avail = String::new();
    proc_file.read_to_string(&mut entropy_avail)?;
    Ok(entropy_avail.trim().parse::<u32>()?)
}

/// Reads the entropy pool size from `/proc/sys/kernel/random/poolsize`.
///
/// Returns the size of the kernel's entropy pool in bits.
///
/// # Returns
/// - `Ok(u32)` - The size of the entropy pool in bits
/// - `Err` - If there's an error reading the pool size
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/poolsize`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
/// - Returns error if the content cannot be parsed as a u32
pub fn poolsize() -> anyhow::Result<u32, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/poolsize")?;
    let mut poolsize = String::new();
    proc_file.read_to_string(&mut poolsize)?;
    Ok(poolsize.trim().parse::<u32>()?)
}

/// Generates a new UUID using the kernel's random number generator.
///
/// Reads a new UUID from `/proc/sys/kernel/random/uuid`.
///
/// # Returns
/// - `Ok(String)` - A new random UUID string
/// - `Err` - If there's an error generating or reading the UUID
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/uuid`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # use linux_crng_ioctl::proc::uuid;
/// # fn main() -> Result<()> {
/// let uuid = uuid()?;
/// println!("Generated UUID: {}", uuid);
/// # Ok(())
/// # }
/// ```
pub fn uuid() -> anyhow::Result<String, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/uuid")?;
    let mut uuid = String::new();
    proc_file.read_to_string(&mut uuid)?;
    Ok(uuid.trim().to_string())
}

/// Reads the minimum reseed time for /dev/urandom.
///
/// Returns the minimum number of seconds between automatic reseeding
/// of the urandom pool from the entropy pool.
///
/// # Returns
/// - `Ok(u32)` - The minimum reseed time in seconds
/// - `Err` - If there's an error reading the value
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/urandom_min_reseed_secs`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
/// - Returns error if the content cannot be parsed as a u32
pub fn urandom_min_reseed_secs() -> anyhow::Result<u32, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/urandom_min_reseed_secs")?;
    let mut min_reseed_secs = String::new();
    proc_file.read_to_string(&mut min_reseed_secs)?;
    Ok(min_reseed_secs.trim().parse::<u32>()?)
}

/// Reads the `write_wakeup_threshold` from `/proc/sys/kernel/random/write_wakeup_threshold`.
///
/// This value determines the threshold at which writers to /dev/random are woken up.
///
/// # Returns
/// - `Ok(u32)` - The current write wakeup threshold
/// - `Err` - If there's an error reading the threshold
///
/// # Errors
/// - Returns error if unable to open `/proc/sys/kernel/random/write_wakeup_threshold`
/// - Returns error if unable to read from the file
/// - Returns error if the file content is not valid UTF-8
/// - Returns error if the content cannot be parsed as a u32
pub fn write_wakeup_threshold() -> anyhow::Result<u32, Error> {
    let mut proc_file = File::open("/proc/sys/kernel/random/write_wakeup_threshold")?;
    let mut write_wakeup_threshold = String::new();
    proc_file.read_to_string(&mut write_wakeup_threshold)?;
    Ok(write_wakeup_threshold.trim().parse::<u32>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proc_boot_id() {
        assert!(boot_id().is_ok());
    }

    #[test]
    fn test_proc_entropy_avail() {
        assert!(entropy_avail().is_ok());
    }

    #[test]
    fn test_proc_poolsize() {
        assert!(poolsize().is_ok());
    }

    #[test]
    fn test_proc_urandom_min_reseed_secs() {
        assert!(urandom_min_reseed_secs().is_ok());
    }

    #[test]
    fn test_proc_uuid() {
        assert!(uuid().is_ok());
    }

    #[test]
    fn test_write_wakeup_threshold() {
        assert!(write_wakeup_threshold().is_ok());
    }
}
