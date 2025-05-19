#!/bin/bash

source ../utils/bench_utils.sh

declare -a BENCH_NAMES=(
    "redis/all_100k_conc20_rps"
    "redis/all_100k_conc20_rps_without_iommu"
)

declare -a BENCH_METHODS=(
    "host_guest"
    "host_guest"
)

runInputBenchs

collectResults "redis"

mv ../../../all-*.csv ../result/redis/
mv ../../../without-iommu-all-*.csv ../result/redis/
