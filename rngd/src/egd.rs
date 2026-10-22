//! Entropy Gathering Daemon (EGD) protocol on a Unix socket.
//!
//! Requests start with a command byte:
//! - `0x00`: entropy level, answered with a 32 bit big endian bit count
//! - `0x01 n`: non-blocking read, answered with a count byte and that many bytes
//! - `0x02 n`: blocking read, answered with n bytes
//! - `0x03 bits(16 bit big endian) n data`: write entropy, no answer
//! - `0x04`: pid, answered with a length byte and the pid as decimal string

use log::{debug, error, info, warn};
use rand_core::TryRng;
use rand_jitterentropy::RandJitterEntropy;
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::{
        fs::FileTypeExt,
        net::{UnixListener, UnixStream},
    },
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};
use zeroize::Zeroizing;

/// Every client holds its own entropy collector, including its memory access
/// region, so bound their number.
const MAX_CLIENTS: usize = 16;

/// The Jitter RNG produces on demand, there is no pool which runs dry. Report
/// a fixed level, enough for any client to go ahead.
const ENTROPY_LEVEL_BITS: u32 = 4096;

/// Serve one client until it disconnects. `fill` produces the random bytes.
pub fn handle_client<S, F, E>(stream: &mut S, mut fill: F) -> io::Result<()>
where
    S: Read + Write,
    F: FnMut(&mut [u8]) -> Result<(), E>,
    E: std::fmt::Display,
{
    let mut cmd = [0u8; 1];
    let mut buf = Zeroizing::new([0u8; 255]);

    loop {
        match stream.read_exact(&mut cmd) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            res => res?,
        }

        match cmd[0] {
            0x00 => stream.write_all(&ENTROPY_LEVEL_BITS.to_be_bytes())?,
            0x01 => {
                let n = usize::from(read_u8(stream)?);
                // a failed health test only means no data for this request
                let n = match fill(&mut buf[..n]) {
                    Ok(()) => n,
                    Err(e) => {
                        warn!("EGD: jitterentropy failed: {e}");
                        0
                    }
                };
                stream.write_all(&[u8::try_from(n).unwrap()])?;
                stream.write_all(&buf[..n])?;
            }
            0x02 => {
                let n = usize::from(read_u8(stream)?);
                // a blocking read cannot be answered short, drop the client
                if let Err(e) = fill(&mut buf[..n]) {
                    return Err(io::Error::other(format!("jitterentropy failed: {e}")));
                }
                stream.write_all(&buf[..n])?;
            }
            0x03 => {
                // clients cannot contribute: the Jitter RNG has no pool to mix
                // into, and its output must not depend on what they send
                let mut header = [0u8; 3];
                stream.read_exact(&mut header)?;
                stream.read_exact(&mut buf[..usize::from(header[2])])?;
            }
            0x04 => {
                let pid = std::process::id().to_string();
                stream.write_all(&[u8::try_from(pid.len()).unwrap()])?;
                stream.write_all(pid.as_bytes())?;
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown EGD command {other:#04x}"),
                ));
            }
        }
    }
}

fn read_u8(stream: &mut impl Read) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    stream.read_exact(&mut byte)?;
    Ok(byte[0])
}

fn serve_client(mut stream: UnixStream, osr: u32, flags: u32) {
    let mut rng = match RandJitterEntropy::with_osr_and_flags(osr, flags) {
        Ok(rng) => rng,
        Err(e) => {
            error!("EGD: failed to create jitterentropy instance: {e}");
            return;
        }
    };

    if let Err(e) = handle_client(&mut stream, |buf| rng.try_fill_bytes(buf)) {
        debug!("EGD: client dropped: {e}");
    }
}

/// Listen on `path` and serve every client from its own thread and entropy
/// collector, created with `osr` and `flags`.
pub fn spawn(path: &Path, osr: u32, flags: u32) -> io::Result<()> {
    // a socket left behind by an earlier run blocks the bind
    if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_socket()) {
        fs::remove_file(path)?;
    }
    let listener = UnixListener::bind(path)?;
    info!("EGD: listening on {}", path.display());

    let clients = Arc::new(AtomicUsize::new(0));
    thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    warn!("EGD: accept failed: {e}");
                    continue;
                }
            };

            if clients.fetch_add(1, Ordering::AcqRel) >= MAX_CLIENTS {
                clients.fetch_sub(1, Ordering::AcqRel);
                warn!("EGD: too many clients, rejecting one");
                continue;
            }

            let clients = Arc::clone(&clients);
            thread::spawn(move || {
                serve_client(stream, osr, flags);
                clients.fetch_sub(1, Ordering::AcqRel);
            });
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Request bytes in, answer bytes out.
    struct Duplex {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl Read for Duplex {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for Duplex {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.output.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn run(request: &[u8], fail: bool) -> (io::Result<()>, Vec<u8>) {
        let mut stream = Duplex {
            input: Cursor::new(request.to_vec()),
            output: Vec::new(),
        };
        let res = handle_client(&mut stream, |buf: &mut [u8]| {
            if fail {
                return Err("induced");
            }
            buf.fill(0xaa);
            Ok(())
        });
        (res, stream.output)
    }

    #[test]
    fn test_level() {
        let (res, out) = run(&[0x00], false);
        assert!(res.is_ok());
        assert_eq!(out, ENTROPY_LEVEL_BITS.to_be_bytes());
    }

    #[test]
    fn test_read_nonblocking() {
        let (res, out) = run(&[0x01, 3], false);
        assert!(res.is_ok());
        assert_eq!(out, [3, 0xaa, 0xaa, 0xaa]);

        let (res, out) = run(&[0x01, 3], true);
        assert!(res.is_ok());
        assert_eq!(out, [0]);
    }

    #[test]
    fn test_read_blocking() {
        let (res, out) = run(&[0x02, 2, 0x02, 0], false);
        assert!(res.is_ok());
        assert_eq!(out, [0xaa, 0xaa]);

        let (res, out) = run(&[0x02, 2], true);
        assert!(res.is_err());
        assert_eq!(out, []);
    }

    #[test]
    fn test_write_is_discarded() {
        let (res, out) = run(&[0x03, 0, 16, 2, 1, 2, 0x00], false);
        assert!(res.is_ok());
        assert_eq!(out, ENTROPY_LEVEL_BITS.to_be_bytes());
    }

    #[test]
    fn test_pid() {
        let (res, out) = run(&[0x04], false);
        let pid = std::process::id().to_string();
        assert!(res.is_ok());
        assert_eq!(usize::from(out[0]), pid.len());
        assert_eq!(&out[1..], pid.as_bytes());
    }

    #[test]
    fn test_unknown_and_truncated() {
        assert!(run(&[0x05], false).0.is_err());
        assert!(run(&[0x01], false).0.is_err());
    }

    #[test]
    fn test_socket() {
        let dir = std::env::temp_dir().join(format!("jitter-rngd-egd-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("egd");
        spawn(&path, RandJitterEntropy::DEFAULT_OSR, 0).unwrap();

        let mut stream = UnixStream::connect(&path).unwrap();
        stream.write_all(&[0x02, 32]).unwrap();
        let mut buf = [0u8; 32];
        stream.read_exact(&mut buf).unwrap();
        assert_ne!(buf, [0u8; 32]);

        drop(stream);
        fs::remove_dir_all(&dir).unwrap();
    }
}
