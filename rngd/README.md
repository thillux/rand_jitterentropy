# jitter-rngd

Toy `rngd` implementation for `rand_jitterentropy` development.

It periodically gathers entropy from the CPU jitter RNG, conditions it with
SHA3-512 (with domain separation between the retained state and the output),
and feeds the result into the Linux kernel CRNG via the `RNDADDENTROPY` ioctl.

Requires root privileges (`CAP_SYS_ADMIN`) to credit entropy to the kernel. It
is a development tool for this workspace, not a hardened system daemon.

## Usage

```shell
jitter-rngd [OPTIONS]

Options:
  -o, --oneshot                            Seed once, then exit
  -s, --seed-interval-s <SEED_INTERVAL_S>  Seconds between seeding rounds [default: 10]
  -f, --force-crng-reseed                  Force a kernel CRNG reseed after seeding
```

Logging goes through `env_logger`, so `RUST_LOG=debug` shows every seeding
round:

```shell
RUST_LOG=debug sudo -E jitter-rngd --oneshot
```

## Building

The daemon links `libjitterentropy` statically and enables NTG.1 compliant
operation (which needs jitterentropy >= 3.7.0):

```shell
cargo build --release -p jitter-rngd
# or, as a static binary through the flake of this repository
nix build .#rngd
```
