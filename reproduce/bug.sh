#!/usr/bin/bash

set -eou pipefail

export PATH="$PWD/rust/build/x86_64-unknown-linux-gnu/stage2/bin:$PATH"
cargo -V
miri -V
cargo miri -V
cd reproduce/bug

# ostd needs this to compile
export RUSTFLAGS=--cfg=ktest MIRIFLAGS="-Zmiri-disable-stacked-borrows -Zmiri-ignore-leaks"

# Run KernMiri
cargo miri run 2> >(tee bug.txt)
