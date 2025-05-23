release:
	(cd libjitterentropy-sys; cargo publish)
	(cd rand_jitterentropy; cargo publish)
	(cd linux-crng-ioctl; cargo publish)
