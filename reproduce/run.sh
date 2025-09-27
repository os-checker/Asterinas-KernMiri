# Don't download toolchain artifact through rustup-toolchain-install-master,
# because the commit is too old and corresponding artifact is absent.
# Therefore we must set up miri toolchain from local source.
git apply reproduce/miri-script.patch

# Clone rust repo at the commit as specified by rust-version file
git submodule update --init rust

# Install the latest version that are available to download at this commit
rustup default nightly-2024-11-03
rustc --print sysroot

# Copy config.toml to rust folder: the path to bins may be replace by the output above
cp reproduce/config.toml rust/

cd rust

# LLVM requires
apt install -y cmake ninja-build libssl-dev build-essential pkg-config

# Prepare: don't override config.toml because it contains local
./x setup

# Build stage2 artifacts: LLVM, rustc, std, and tools
# LLVM may take a long time to build, like 1 hour on 4-cores server.
./x build --stage 2 rustfmt clippy cargo

# Add local rustc toolchain named miri which miri requires
rustup toolchain link miri build/aarch64-unknown-linux-gnu/stage2

