use anyhow::{Error, Result, anyhow};
use log::{debug, error};
use nix::{ioctl_none, ioctl_read, ioctl_write_ptr};
use std::{fs::File, os::fd::AsRawFd};
use zeroize::{Zeroize, ZeroizeOnDrop};

/* numbers and comments taken from: include/uapi/linux/random.h */
const IOC_MAGIC: u8 = b'R';

/* ioctl()'s for the random number generator */

/* Get the entropy count. */
const RNDGETENTCNT: u8 = 0x0;

/* Add to (or subtract from) the entropy count.  (Superuser only.) */
const RNDADDTOENTCNT: u8 = 0x1;

/* Get the contents of the entropy pool.  (Superuser only.) (Removed in 2.6.9-rc2.) */
// const RNDGETPOOL: u8 = 0x2;

/* Add to (or subtract from) the entropy count.  (Superuser only.) */
const RNDADDENTROPY: u8 = 0x3;

/* Clear entropy count to 0.  (Superuser only.) */
const RNDZAPENTCNT: u8 = 0x4;

/* Clear the entropy pool and associated counters.  (Superuser only.) */
const RNDCLEARPOOL: u8 = 0x6;

/* Reseed CRNG.  (Superuser only.) */
const RNDRESEEDCRNG: u8 = 0x7;

/* Max input size for writing entropy to kernel */
const MAX_BUFFER_SIZE: usize = 2 * 1024;

#[repr(C)]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KernelRandPoolInfoHeader {
    entropy_bits: i32,
    buf_size_byte: i32,
}

#[repr(C)]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KernelRandPoolInfo {
    header: KernelRandPoolInfoHeader,
    buf: [u8; MAX_BUFFER_SIZE],
}

ioctl_read!(rnd_get_ent_cnt, IOC_MAGIC, RNDGETENTCNT, i32);

ioctl_write_ptr!(rnd_add_to_ent_cnt, IOC_MAGIC, RNDADDTOENTCNT, i32);

// we need the header struct size to match 2*sizeof(i32) in order to match the kernel
// ioctl magic
ioctl_write_ptr!(
    rnd_add_entropy,
    IOC_MAGIC,
    RNDADDENTROPY,
    KernelRandPoolInfoHeader
);

ioctl_none!(rnd_zap_ent_cnt, IOC_MAGIC, RNDZAPENTCNT);
ioctl_none!(rnd_clear_pool, IOC_MAGIC, RNDCLEARPOOL);
ioctl_none!(rnd_reseed_crng, IOC_MAGIC, RNDRESEEDCRNG);

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn get_ent_cnt() -> Result<i32> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();
    let mut ent_cnt = 0;

    let ret = unsafe { rnd_get_ent_cnt(fd, &mut ent_cnt) };
    if let Ok(0) = ret {
        Ok(ent_cnt)
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to fetch entropy level from kernel"))
    }
}

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn add_to_ent_cnt(ent_cnt: i32) -> Result<()> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    let ret = unsafe { rnd_add_to_ent_cnt(fd, &ent_cnt) };
    if let Ok(0) = ret {
        Ok(())
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to add to ent cnt"))
    }
}

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn add_randomness_to_kernel(entropy: &[u8], ent_bits: u32) -> Result<()> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    if usize::try_from(ent_bits)? > entropy.len() * 8 {
        return Err(anyhow!("Do not claim more entropy than buffer length * 8!"));
    }

    if entropy.len() > MAX_BUFFER_SIZE {
        return Err(anyhow!("This implementation currently can write up to {MAX_BUFFER_SIZE} Byte to kernel CRNG input pool"));
    }

    debug!(
        "Write {} Byte to /dev/random, accounted with {} Bit entropy",
        64, ent_bits
    );

    let mut pool_info = KernelRandPoolInfo {
        header: KernelRandPoolInfoHeader {
            entropy_bits: i32::try_from(ent_bits)?,
            buf_size_byte: i32::try_from(entropy.len())?,
        },
        buf: [0; MAX_BUFFER_SIZE],
    };
    pool_info.buf[0..entropy.len()].copy_from_slice(entropy);

    #[allow(clippy::ptr_as_ptr)]
    let res = unsafe {
        rnd_add_entropy(
            fd,
            std::ptr::addr_of!(pool_info) as *const KernelRandPoolInfoHeader,
        )
    };

    if let Ok(0) = res {
        Ok(())
    } else {
        error!("ioctl returned with error");
        Err(anyhow!("Failed to add entropy to kernel"))
    }
}

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn clear_entropy_count() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { rnd_zap_ent_cnt(fd) } {
        Ok(0) => {
            debug!("Cleared kernel CRNG entropy count to 0");
            Ok(())
        }
        _ => Err(anyhow!("Cannot clear CRNG entropy count to 0")),
    }
}

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn clear_pool() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { rnd_clear_pool(fd) } {
        Ok(0) => {
            debug!("Forcefully cleared kernel CRNG pool");
            Ok(())
        }
        _ => Err(anyhow!("Cannot clear CRNG pool")),
    }
}

///
/// # Errors
/// - access to kernel fails or no more fds available
pub fn force_kernel_crng_reseed() -> Result<(), Error> {
    let random_file = File::create("/dev/random")?;
    let fd = random_file.as_raw_fd();

    match unsafe { rnd_reseed_crng(fd) } {
        Ok(0) => {
            debug!("Forcefully reseeded kernel CRNG");
            Ok(())
        }
        _ => Err(anyhow!("Cannot reseed CRNG")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        add_randomness_to_kernel, add_to_ent_cnt, clear_entropy_count, clear_pool,
        force_kernel_crng_reseed, get_ent_cnt,
    };
    use nix::unistd::Uid;

    #[test]
    fn test_get_ent_cnt() {
        assert!(get_ent_cnt().is_ok(), "failed to get entropy count");
    }

    #[test]
    fn test_add_to_entropy_count() {
        if Uid::effective().is_root() {
            assert!(
                add_to_ent_cnt(32).is_ok(),
                "failed to add to CRNG entropy count"
            );
        }
    }

    #[test]
    fn test_add_entropy() {
        if Uid::effective().is_root() {
            assert!(
                add_randomness_to_kernel(&[0u8; 32], 256).is_ok(),
                "failed to add randomness to kernel"
            );
        }
    }

    #[test]
    fn test_clear_entropy_count() {
        if Uid::effective().is_root() {
            assert!(
                clear_entropy_count().is_ok(),
                "failed to clear CRNG entropy count"
            );
        }
    }

    #[test]
    fn test_clear_pool() {
        if Uid::effective().is_root() {
            assert!(clear_pool().is_ok(), "failed to clear CRNG pool");
        }
    }

    #[test]
    fn test_reseed_crng() {
        if Uid::effective().is_root() {
            assert!(force_kernel_crng_reseed().is_ok(), "failed to reseed CRNG");
        }
    }
}
