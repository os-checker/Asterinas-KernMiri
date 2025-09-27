#!/usr/bin/bash

set -eou pipefail

export PATH="$PWD/rust/build/x86_64-unknown-linux-gnu/stage2/bin:$PATH"
cargo -V
miri -V
cargo miri -V
cd reproduce/bug
RUSTFLAGS=--cfg=ktest cargo miri run 2>bug.txt
