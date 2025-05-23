use rand_core::TryRngCore;
use std::sync::Mutex;

static LIB_MUTEX_UNPRIV: Mutex<u32> = Mutex::new(0u32);

pub struct RandJitterEntropy {
    rand_data: *mut libjitterentropy_sys::jitterentropy::rand_data,
}

impl RandJitterEntropy {
    /// Create new handle for jitterentropy based True RNG.
    /// # Errors
    ///
    /// - unable to lock jitterentropy mutex for single initialization
    /// - unable to init jitterentropy
    /// - unable to allocate jitterentropy state handle
    pub fn new() -> Result<Self, std::io::Error> {
        let mut guard = LIB_MUTEX_UNPRIV
            .lock()
            .map_err(|_| std::io::Error::other("unable to lock jitterentropy mutex!"))?;

        let osr: std::os::raw::c_uint = 1;
        // enable all health tests
        let flags: std::os::raw::c_uint = libjitterentropy_sys::jitterentropy::JENT_FORCE_FIPS;

        let ret = if *guard == 0 {
            unsafe { libjitterentropy_sys::jitterentropy::jent_entropy_init_ex(osr, flags) == 0 }
        } else {
            true
        };

        if ret {
            *guard += 1;
        } else {
            return Err(std::io::Error::other("unable to init jitterentropy!"));
        }

        let rand_data =
            unsafe { libjitterentropy_sys::jitterentropy::jent_entropy_collector_alloc(osr, flags) };
        if rand_data.is_null() {
            Err(std::io::Error::other(
                "unable to allocate jitterentropy state!",
            ))
        } else {
            Ok(RandJitterEntropy { rand_data })
        }
    }
}

impl TryRngCore for RandJitterEntropy {
    type Error = std::io::Error;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        u32::try_from(self.try_next_u64()? & 0xFF_FF_FF_FF)
            .map_err(|_| std::io::Error::other("unable to convert u64 to u32!"))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes: [u8; 8] = [0; 8];
        self.try_fill_bytes(&mut bytes)?;

        Ok(u64::from_ne_bytes(bytes))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        let ret = unsafe {
            libjitterentropy_sys::jitterentropy::jent_read_entropy_safe(
                &mut self.rand_data,
                dst.as_mut_ptr().cast(),
                dst.len(),
            )
        };
        if ret
            == isize::try_from(dst.len()).map_err(|_| {
                std::io::Error::other("unable to convert destination size to type isize")
            })?
        {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "unable to get random bytes of length {}",
                dst.len()
            )))
        }
    }
}

impl Default for RandJitterEntropy {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for RandJitterEntropy {
    fn drop(&mut self) {
        unsafe {
            libjitterentropy_sys::jitterentropy::jent_entropy_collector_free(self.rand_data);
        }

        let mut guard = LIB_MUTEX_UNPRIV.lock().unwrap();

        *guard -= 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::RandJitterEntropy;
    use rand_core::TryRngCore;

    #[test]
    fn test_u32() {
        let mut rng = RandJitterEntropy::new().unwrap();
        for _ in 0..128 {
            let u = rng.try_next_u32();
            assert!(u.is_ok());
        }
    }

    #[test]
    fn test_u64() {
        let mut rng = RandJitterEntropy::new().unwrap();
        for _ in 0..128 {
            let u = rng.try_next_u64();
            assert!(u.is_ok());
        }
    }

    #[test]
    fn test_speed() {
        use std::time::Instant;
        let start = Instant::now();
        let mut num_bytes = 0usize;
        let mut rng = RandJitterEntropy::new().unwrap();

        loop {
            let mut b = [0u8; 32];
            rng.try_fill_bytes(&mut b).unwrap();

            let now = Instant::now();

            num_bytes += b.len();

            if (now - start).as_secs() > 2 {
                let datarate = f64::from(u32::try_from(num_bytes).unwrap())
                    / (now - start).as_secs_f64()
                    / 1024.0;
                println!("datarate: {datarate} KiB/s");
                break;
            }
        }
    }

    #[test]
    fn test_bytes() {
        let mut rng = RandJitterEntropy::new().unwrap();

        for buffer_size in 0..=256 {
            let mut buffer = vec![0u8; buffer_size];
            assert!(rng.try_fill_bytes(&mut buffer).is_ok());
            println!("{buffer_size}: {buffer:#04X?}");
        }
    }

    #[test]
    fn test_multi_instantiation() {
        for _ in 0..256 {
            let mut rng = RandJitterEntropy::new().unwrap();
            let u = rng.try_next_u32().unwrap();
            println!("Got {u}");
        }
    }

    #[test]
    fn test_multi_threading() {
        let mut threads = vec![];
        let mut rng = RandJitterEntropy::new().unwrap();
        let _ = rng.try_next_u64().unwrap();

        println!("Got bytes (single threaded)!");

        for _ in 0..6 {
            threads.push(std::thread::spawn(move || {
                for _ in 0..128 {
                    let mut rng = RandJitterEntropy::new().unwrap();
                    let _ = rng.try_next_u64().unwrap();
                }
            }));
        }

        for t in threads {
            let _ = t.join();
        }
    }
}
