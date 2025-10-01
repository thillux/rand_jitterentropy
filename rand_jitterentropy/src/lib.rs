use rand_core::TryRngCore;
use std::sync::Mutex;

static LIB_MUTEX_UNPRIV: Mutex<u32> = Mutex::new(0u32);

pub struct RandJitterEntropy {
    rand_data: *mut libjitterentropy_sys::jitterentropy::rand_data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents all possible errors that can occur during Jitter RNG operations.
///
/// This enum covers both initialization errors and runtime errors that may occur
/// during entropy collection and health tests.
pub enum JitterEntropyError {
    /// Timer service not available
    NoTime = 1,
    /// Timer too coarse for RNG
    CoarseTime = 2,
    /// Timer is not monotonic increasing
    NoMonotonic = 3,
    /// Timer variations too small for RNG
    MinVariation = 4,
    /// Timer does not produce variations of variations (2nd derivation of time is zero)
    VarVar = 5,
    /// Timer variations of variations is too small
    MinVarVar = 6,
    /// Programming error or internal error
    ProgErr = 7,
    /// Too many stuck results during init
    Stuck = 8,
    /// Health test failed during initialization
    Health = 9,
    /// RCT failed during initialization
    Rct = 10,
    /// Hash self test failed
    Hash = 11,
    /// Can't allocate memory for initialization
    Memory = 12,
    /// GCD self-test failed
    Gcd = 13,
    /// Entropy collector is NULL
    NullCollector = -1,
    /// RCT (Repetition Count Test) failed during runtime
    RctFailed = -2,
    /// APT (Adaptive Proportion Test) failed during runtime
    AptFailed = -3,
    /// Timer initialization failure
    TimerInitFailed = -4,
    /// LAG (Lag Prediction Test) failure during runtime
    LagFailed = -5,
    /// RCT permanent failure (unrecoverable)
    RctPermanentFailure = -6,
    /// APT permanent failure (unrecoverable)
    AptPermanentFailure = -7,
    /// LAG permanent failure (unrecoverable)
    LagPermanentFailure = -8,
}

impl JitterEntropyError {
    /// Converts a C error code to a Result containing JitterEntropyError.
    ///
    /// # Arguments
    ///
    /// * `code` - The C error code returned from jitterentropy functions
    ///
    /// # Returns
    ///
    /// * `Ok(())` if code is 0
    /// * `Err(JitterEntropyError)` with the appropriate error variant for non-zero codes
    ///
    /// # Errors
    ///
    /// Returns `Err` with following variants based on the error code:
    ///
    /// Positive error codes (initialization errors):
    /// - `NoTime` (1) - Timer service not available
    /// - `CoarseTime` (2) - Timer too coarse for RNG
    /// - `NoMonotonic` (3) - Timer is not monotonic increasing
    /// - `MinVariation` (4) - Timer variations too small for RNG
    /// - `VarVar` (5) - Timer does not produce variations of variations
    /// - `MinVarVar` (6) - Timer variations of variations too small
    /// - `ProgErr` (7) - Programming error
    /// - `Stuck` (8) - Too many stuck results during init
    /// - `Health` (9) - Health test failed during initialization
    /// - `Rct` (10) - RCT failed during initialization
    /// - `Hash` (11) - Hash self test failed
    /// - `Memory` (12) - Can't allocate memory for initialization
    /// - `Gcd` (13) - GCD self-test failed
    ///
    /// Negative error codes (runtime errors):
    /// - `NullCollector` (-1) - Entropy collector is NULL
    /// - `RctFailed` (-2) - RCT failure during operation
    /// - `AptFailed` (-3) - APT failure during operation
    /// - `TimerInitFailed` (-4) - Timer initialization failed
    /// - `LagFailed` (-5) - LAG test failure during operation
    /// - `RctPermanentFailure` (-6) - Unrecoverable RCT failure
    /// - `AptPermanentFailure` (-7) - Unrecoverable APT failure
    /// - `LagPermanentFailure` (-8) - Unrecoverable LAG failure
    ///
    /// Any other error code will return `Err(ProgErr)`.
    pub fn from_c_code(code: i32) -> Result<(), Self> {
        match code {
            0 => Ok(()),
            1 => Err(Self::NoTime),
            2 => Err(Self::CoarseTime),
            3 => Err(Self::NoMonotonic),
            4 => Err(Self::MinVariation),
            5 => Err(Self::VarVar),
            6 => Err(Self::MinVarVar),
            7 => Err(Self::ProgErr),
            8 => Err(Self::Stuck),
            9 => Err(Self::Health),
            10 => Err(Self::Rct),
            11 => Err(Self::Hash),
            12 => Err(Self::Memory),
            13 => Err(Self::Gcd),
            -1 => Err(Self::NullCollector),
            -2 => Err(Self::RctFailed),
            -3 => Err(Self::AptFailed),
            -4 => Err(Self::TimerInitFailed),
            -5 => Err(Self::LagFailed),
            -6 => Err(Self::RctPermanentFailure),
            -7 => Err(Self::AptPermanentFailure),
            -8 => Err(Self::LagPermanentFailure),
            _ => Err(Self::ProgErr), // Unknown errors treated as programming errors
        }
    }
}

impl std::fmt::Display for JitterEntropyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTime => write!(f, "Timer service not available"),
            Self::CoarseTime => write!(f, "Timer too coarse for RNG"),
            Self::NoMonotonic => write!(f, "Timer is not monotonic increasing"),
            Self::MinVariation => write!(f, "Timer variations too small for RNG"),
            Self::VarVar => write!(f, "Timer does not produce variations of variations"),
            Self::MinVarVar => write!(f, "Timer variations of variations is too small"),
            Self::ProgErr => write!(f, "Programming error"),
            Self::Stuck => write!(f, "Too many stuck results during init"),
            Self::Health => write!(f, "Health test failed during initialization"),
            Self::Rct => write!(f, "RCT failed during initialization"),
            Self::Hash => write!(f, "Hash self test failed"),
            Self::Memory => write!(f, "Can't allocate memory for initialization"),
            Self::Gcd => write!(f, "GCD self-test failed"),
            Self::NullCollector => write!(f, "Entropy collector is NULL"),
            Self::RctFailed => write!(f, "RCT (Repetition Count Test) failed"),
            Self::AptFailed => write!(f, "APT (Adaptive Proportion Test) failed"),
            Self::TimerInitFailed => write!(f, "Timer initialization failed"),
            Self::LagFailed => write!(f, "LAG (Lag Prediction Test) failure"),
            Self::RctPermanentFailure => write!(f, "RCT permanent failure"),
            Self::AptPermanentFailure => write!(f, "APT permanent failure"),
            Self::LagPermanentFailure => write!(f, "LAG permanent failure"),
        }
    }
}

impl std::error::Error for JitterEntropyError {}

impl From<i32> for JitterEntropyError {
    fn from(code: i32) -> Self {
        JitterEntropyError::from_c_code(code).unwrap_err()
    }
}

impl RandJitterEntropy {
    /// Create new handle for jitterentropy based True RNG.
    ///
    /// # Errors
    ///
    /// Initialization can fail with the following errors:
    /// - `NullCollector` - Entropy collector allocation failed
    /// - `NoTime` - Timer service not available
    /// - `CoarseTime` - Timer too coarse for RNG
    /// - `NoMonotonic` - Timer is not monotonic increasing
    /// - `MinVariation` - Timer variations too small for RNG
    /// - `VarVar` - Timer does not produce variations of variations
    /// - `MinVarVar` - Timer variations of variations too small
    /// - `Stuck` - Too many stuck results during init
    /// - `Health` - Health test failed during initialization
    /// - `Rct` - RCT failed during initialization
    /// - `Hash` - Hash self test failed
    /// - `Memory` - Memory allocation failed
    /// - `Gcd` - GCD self-test failed
    /// - `RctFailed` - Runtime RCT failure
    /// - `AptFailed` - Runtime APT failure
    /// - `TimerInitFailed` - Timer initialization failure
    /// - `LagFailed` - Runtime LAG test failure
    /// - `RctPermanentFailure` - Permanent RCT failure
    /// - `AptPermanentFailure` - Permanent APT failure
    /// - `LagPermanentFailure` - Permanent LAG failure
    /// - `ProgErr` - Programming or internal error
    pub fn new() -> Result<Self, JitterEntropyError> {
        let mut guard = LIB_MUTEX_UNPRIV
            .lock()
            .map_err(|_| JitterEntropyError::ProgErr)?;

        let osr: std::os::raw::c_uint = 3;
        #[cfg(feature = "ntg1")]
        let flags: std::os::raw::c_uint = libjitterentropy_sys::jitterentropy::JENT_FORCE_FIPS | libjitterentropy_sys::jitterentropy::JENT_NTG1;
        #[cfg(not(feature = "ntg1"))]
        let flags: std::os::raw::c_uint = libjitterentropy_sys::jitterentropy::JENT_FORCE_FIPS;

        let ret = if *guard == 0 {
            unsafe {
                JitterEntropyError::from_c_code(
                    libjitterentropy_sys::jitterentropy::jent_entropy_init_ex(osr, flags),
                )?;
            };
            true
        } else {
            true
        };

        if ret {
            *guard += 1;
        } else {
            return Err(JitterEntropyError::ProgErr);
        }

        let rand_data = unsafe {
            libjitterentropy_sys::jitterentropy::jent_entropy_collector_alloc(osr, flags)
        };
        if rand_data.is_null() {
            Err(JitterEntropyError::NullCollector)
        } else {
            Ok(RandJitterEntropy { rand_data })
        }
    }
}

impl TryRngCore for RandJitterEntropy {
    type Error = JitterEntropyError;

    /// Generates a random u32 value.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Entropy collection fails (any runtime error from `JitterEntropyError`)
    /// - `ProgErr` if internal type conversion fails
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        u32::try_from(self.try_next_u64()? & 0xFF_FF_FF_FF).map_err(|_| JitterEntropyError::ProgErr)
    }

    /// Generates a random u64 value.
    ///
    /// # Errors
    ///
    /// Returns error if entropy collection fails with any runtime error from `JitterEntropyError`
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes: [u8; 8] = [0; 8];
        self.try_fill_bytes(&mut bytes)?;

        Ok(u64::from_ne_bytes(bytes))
    }

    /// Fills the provided buffer with random bytes.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Entropy collection fails (any runtime error from `JitterEntropyError`)
    /// - `ProgErr` if buffer length conversion fails
    /// - Runtime health test failures (`RctFailed`, `AptFailed`, `LagFailed`)
    /// - Permanent test failures (`RctPermanentFailure`, `AptPermanentFailure`, `LagPermanentFailure`)
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        let ret = unsafe {
            libjitterentropy_sys::jitterentropy::jent_read_entropy_safe(
                &mut self.rand_data,
                dst.as_mut_ptr().cast(),
                dst.len(),
            )
        };

        let expected_len = isize::try_from(dst.len()).map_err(|_| JitterEntropyError::ProgErr)?;

        if ret == expected_len {
            Ok(())
        } else {
            Err(JitterEntropyError::from_c_code(
                i32::try_from(ret).map_err(|_| JitterEntropyError::ProgErr)?,
            )
            .unwrap_err())
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
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(JitterEntropyError::from_c_code(0), Ok(()));
        assert_eq!(
            JitterEntropyError::from_c_code(1),
            Err(JitterEntropyError::NoTime)
        );
        assert_eq!(
            JitterEntropyError::from_c_code(13),
            Err(JitterEntropyError::Gcd)
        );
        assert_eq!(
            JitterEntropyError::from_c_code(-1),
            Err(JitterEntropyError::NullCollector)
        );
        assert_eq!(
            JitterEntropyError::from_c_code(-8),
            Err(JitterEntropyError::LagPermanentFailure)
        );
        assert_eq!(
            JitterEntropyError::from_c_code(99),
            Err(JitterEntropyError::ProgErr)
        );
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            JitterEntropyError::NoTime.to_string(),
            "Timer service not available"
        );
        assert_eq!(
            JitterEntropyError::NullCollector.to_string(),
            "Entropy collector is NULL"
        );
        assert_eq!(
            JitterEntropyError::RctPermanentFailure.to_string(),
            "RCT permanent failure"
        );
    }

    #[test]
    fn test_from_i32() {
        let err: JitterEntropyError = (-1).into();
        assert_eq!(err, JitterEntropyError::NullCollector);

        let err: JitterEntropyError = (-8).into();
        assert_eq!(err, JitterEntropyError::LagPermanentFailure);
    }

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
