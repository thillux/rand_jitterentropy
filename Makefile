CARGO ?= cargo

.PHONY: all build test clippy fmt fmt-check doc check clean distclean nix nix-fmt release

all: build

build:
	$(CARGO) build --workspace --all-targets

test:
	$(CARGO) test --workspace

clippy:
	$(CARGO) clippy --workspace --all-targets

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

doc:
	$(CARGO) doc --workspace --no-deps

# everything the CI cares about
check: fmt-check clippy test

# build the statically linked daemon through the flake
nix:
	nix build .#rngd

nix-fmt:
	nix fmt

clean:
	$(CARGO) clean

# also drop the build results of the flake
distclean: clean
	rm -rf result result-bin result-dev

release:
	(cd libjitterentropy-sys; cargo publish)
	(cd rand_jitterentropy; cargo publish)
	(cd linux-crng-ioctl; cargo publish)
