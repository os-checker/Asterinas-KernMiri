#!/usr/bin/bash

set -euo pipefail

# Don't download toolchain artifact through rustup-toolchain-install-master,
# because the commit is too old and corresponding artifact is absent.
# Therefore we must set up miri toolchain from local source.
git apply reproduce/miri-script.patch
git apply reproduce/2-miri-script.patch

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

# Copy cargo because it's not included
cp build/x86_64-unknown-linux-gnu/stage2-tools-bin/cargo build/x86_64-unknown-linux-gnu/stage2/bin

# Package stage2
export XZ_OPT="-T0 --memlimit=10G -9"
#time tar -cJvf stage2.tar.xz build/x86_64-unknown-linux-gnu/stage2
# tar -xJvf stage2.tar.xz # to restore build folder

cd ..

./miri toolchain
./miri build
./miri run tests/fail/rc_as_ptr.rs
./miri install
