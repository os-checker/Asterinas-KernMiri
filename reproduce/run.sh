# Clone rust repo at the commit as specified by rust-version file
git submodule update --init rust

# Install the latest version that are available to download at this commit
rustup default nightly-2024-11-03
rustc --print sysroot

# Copy config.toml to rust folder: the path to bins may be replace by the output above
cp reproduce/config.toml rust/

cd rust

# LLVM requires
apt install -y cmake ninja-build build-essential

# Prepare: don't override config.toml because it contains local
./x setup

# Build stage2 artifacts: LLVM, rustc, std, and tools
./x build --stage 2 rustfmt clippy cargo
