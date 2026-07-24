# jitter-rngd

Toy `rngd` implementation for `rand_jitterentropy` development.

It periodically gathers entropy from the CPU jitter RNG, conditions it with
SHA3-512 (with domain separation between the retained state and the output),
and feeds the result into the Linux kernel CRNG via the `RNDADDENTROPY` ioctl.

Requires root privileges (`CAP_SYS_ADMIN`) to credit entropy to the kernel.

## Usage

```shell
jitter-rngd [OPTIONS]

Options:
  -o, --oneshot                            Seed once, then exit
  -s, --seed-interval-s <SEED_INTERVAL_S>  Seconds between seeding rounds [default: 10]
  -f, --force-crng-reseed                  Force a kernel CRNG reseed after seeding
```
