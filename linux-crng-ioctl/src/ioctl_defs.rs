/* numbers and comments taken from: include/uapi/linux/random.h */
use nix::{ioctl_none, ioctl_read, ioctl_write_ptr};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const IOC_MAGIC: u8 = b'R';

/* ioctl()'s for the random number generator */

/* Get the entropy count. */
pub const RNDGETENTCNT: u8 = 0x0;

/* Add to (or subtract from) the entropy count.  (Superuser only.) */
pub const RNDADDTOENTCNT: u8 = 0x1;

/* Get the contents of the entropy pool.  (Superuser only.) (Removed in 2.6.9-rc2.) */
// const RNDGETPOOL: u8 = 0x2;

/* Add to (or subtract from) the entropy count.  (Superuser only.) */
pub const RNDADDENTROPY: u8 = 0x3;

/* Clear entropy count to 0.  (Superuser only.) */
pub const RNDZAPENTCNT: u8 = 0x4;

/* Clear the entropy pool and associated counters.  (Superuser only.) */
pub const RNDCLEARPOOL: u8 = 0x6;

/* Reseed CRNG.  (Superuser only.) */
pub const RNDRESEEDCRNG: u8 = 0x7;

/* Max input size for writing entropy to kernel */
pub const MAX_BUFFER_SIZE: usize = 2 * 1024;

#[repr(C)]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KernelRandPoolInfoHeader {
    pub entropy_bits: i32,
    pub buf_size_byte: i32,
}

#[repr(C)]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KernelRandPoolInfo {
    pub header: KernelRandPoolInfoHeader,
    pub buf: [u8; MAX_BUFFER_SIZE],
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
