#!/bin/bash

RUN_TIMES=10

# Use the input arguments as DIRS
if [ $# -gt 0 ]; then
    DIRS=("$@")
else
    # Default directories to run
    DIRS=("lmbench" "nginx" "redis" "sqlite")
fi

cd ..
mkdir -p ./result-all

for ((x = 1; x <= RUN_TIMES; x++)); do
    for dir in "${DIRS[@]}"; do
        pushd "$dir" >/dev/null
        bash run_all.sh
        popd >/dev/null
    done

    mkdir -p "./result-all/result-${x}"
    cp -r ./result/* "./result-all/result-${x}/"
    rm -rf ./result/*
done
