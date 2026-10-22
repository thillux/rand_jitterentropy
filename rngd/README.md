# jitter-rngd

Toy `rngd` implementation for `rand_jitterentropy` development.

It periodically reads 64 bytes from the CPU jitter RNG and feeds them as is,
credited with 512 bits, into the Linux kernel CRNG via the `RNDADDENTROPY`
ioctl. Intermittent health test failures skip a round, permanent ones end the
daemon.

Requires root privileges (`CAP_SYS_ADMIN`) to credit entropy to the kernel. It
is a development tool for this workspace, not a hardened system daemon.

## Usage

```shell
jitter-rngd [OPTIONS]

Options:
  -o, --oneshot                  Seed once, then exit
  -s, --seed-interval-s <S>      Seconds between seeding rounds [default: 10]
  -f, --force-crng-reseed        Force a kernel CRNG reseed after each round
      --status                   Print the Jitter RNG status as JSON, then exit
      --osr <OSR>                Oversampling rate [default: 6]
      --fips                     FIPS mode, health test failures stop the output
      --ntg1                     AIS 20/31 NTG.1 compliance, implies --fips
      --max-memsize <SIZE>       1kB ... 512MB
      --cache-all                Size the memory access by all caches
      --disable-memory-access
      --hashloop <N>             1 ... 128
      --force-internal-timer
      --disable-internal-timer
```

Without `--fips` or `--ntg1` health test failures only show up in the status,
the output keeps flowing. The options need jitterentropy >= 3.7.0.

| Signal | Action |
| --- | --- |
| `SIGUSR1` | Seed and force a kernel CRNG reseed right away |
| `SIGUSR2` | Print the Jitter RNG status as JSON |

## EGD interface

With the `egd` feature, `--egd-socket <PATH>` additionally serves the Entropy
Gathering Daemon protocol on a Unix socket, e.g. for OpenSSL's `RAND_egd()`.
Every client gets its own entropy collector, configured by the options above,
up to 16 clients at once. Reads (`0x01`, `0x02`), entropy level (`0x00`, a
fixed 4096 bits) and pid (`0x04`) are answered; written entropy (`0x03`) is
discarded. Access is controlled by the socket's file permissions.

`--no-kernel` skips seeding the kernel CRNG and only serves EGD, which needs
no root; SIGUSR1 is ignored then.

```shell
cargo build --release -p jitter-rngd --features egd
sudo jitter-rngd --ntg1 --egd-socket /run/egd-pool
jitter-rngd --ntg1 --egd-socket /tmp/egd-pool --no-kernel
```

Logging goes through `env_logger`, so `RUST_LOG=debug` shows every seeding
round:

```shell
RUST_LOG=debug sudo -E jitter-rngd --oneshot
```

## Building

The daemon links `libjitterentropy` statically and needs jitterentropy >=
3.7.0:

```shell
cargo build --release -p jitter-rngd
# or, as a static binary through the flake of this repository
nix build .#rngd
```
