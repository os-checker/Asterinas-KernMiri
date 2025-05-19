#!/bin/bash

source ../utils/bench_utils.sh

declare -a BENCH_NAMES=(
    "nginx/http_file4KB_bw"
    "nginx/http_file8KB_bw"
    "nginx/http_file16KB_bw"
    "nginx/http_file32KB_bw"
    "nginx/http_file64KB_bw"
    "nginx/http_file4KB_bw_no_iommu"
    "nginx/http_file8KB_bw_no_iommu"
    "nginx/http_file16KB_bw_no_iommu"
    "nginx/http_file32KB_bw_no_iommu"
    "nginx/http_file64KB_bw_no_iommu"
)

declare -a BENCH_METHODS=(
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
    "host_guest"
)

runInputBenchs

collectResults "nginx"
