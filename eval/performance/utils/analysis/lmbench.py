import os
import json
import csv
import sys
from collections import defaultdict

file_command_map = {
    "process_getppid_lat": "lat_syscall null",
    "process_ctx_lat": "lat_ctx 18",
    "process_fork_lat": "lat_proc fork",
    "process_exec_lat": "lat_proc exec",
    "process_shell_lat": "lat_proc shell",
    "mem_pagefault_lat": "lat_pagefault",
    "mem_mmap_lat": "lat_mmap 4m",
    "mem_mmap_bw": "bw_mmap 256m mmap_only",
    "pipe_lat": "lat_pipe",
    "pipe_bw": "bw_pipe",
    "fifo_lat": "lat_fifo",
    "unix_lat": "lat_unix",
    "unix_bw": "bw_unix",
    "vfs_open_lat": "lat_syscall open",
    "vfs_read_lat": "lat_syscall read",
    "vfs_write_lat": "lat_syscall write",
    "vfs_stat_lat": "lat_syscall stat",
    "vfs_fstat_lat": "lat_syscall fstat",
    "vfs_read_pagecache_bw": "bw_file_rd 512m io_only",
    "ramfs_copy_files_bw": "lmdd if=/tmp of=/tmp",
    "ramfs_copy_to_ext2_files_bw": "lmdd if=/tmp of=/ext2",
    "ext2_copy_to_ramfs_files_bw": "lmdd if=/ext2 of=/tmp",
    "ext2_copy_files_bw": "lmdd if=/ext2 of=/ext2",
    "udp_loopback_lat": "(Loopback)lat_udp",
    "tcp_loopback_lat": "(Loopback)lat_tcp",
    "tcp_loopback_bw_128": "(Loopback)bw_tcp 128",
    "tcp_loopback_bw_64k": "(Loopback)bw_tcp 64k",
    "udp_virtio_lat": "(VirtIO)lat_udp",
    "tcp_virtio_lat": "(VirtIO)lat_tcp",
    "tcp_virtio_bw_128": "(VirtIO)bw_tcp 128",
    "tcp_virtio_bw_64k": "(VirtIO)bw_tcp 64k",
}

# The command names in the CSV output
command_headers = [
    "lat_syscall null",
    "lat_ctx 18",
    "lat_proc fork",
    "lat_proc exec",
    "lat_proc shell",
    "lat_pagefault",
    "lat_mmap 4m",
    "bw_mmap 256m mmap_only",
    "lat_pipe",
    "bw_pipe",
    "lat_fifo",
    "lat_unix",
    "bw_unix",
    "lat_syscall open",
    "lat_syscall read",
    "lat_syscall write",
    "lat_syscall stat",
    "lat_syscall fstat",
    "bw_file_rd 512m io_only",
    "lmdd if=/tmp of=/tmp",
    "lmdd if=/tmp of=/ext2",
    "lmdd if=/ext2 of=/tmp",
    "lmdd if=/ext2 of=/ext2",
    "(Loopback)lat_udp",
    "(Loopback)lat_tcp",
    "(Loopback)bw_tcp 128",
    "(Loopback)bw_tcp 64k",
    "(VirtIO)lat_udp",
    "(VirtIO)lat_tcp",
    "(VirtIO)bw_tcp 128",
    "(VirtIO)bw_tcp 64k",
]


def process_lmbench_results(root_dir="result-all"):
    # Initialize data structure to store all results
    results = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))
    linux_counts = defaultdict(int)
    aster_counts = defaultdict(int)

    # Get and sort all result directories numerically
    result_dirs = [d for d in os.listdir(root_dir) if d.startswith("result-")]
    result_dirs.sort(key=lambda x: int(x.split("-")[1]))

    # First pass: collect all unique file names and determine max counts
    for result_dir in result_dirs:
        if not result_dir.startswith("result-"):
            continue

        lmbench_path = os.path.join(root_dir, result_dir, "lmbench")
        if not os.path.exists(lmbench_path):
            continue

        for json_file in os.listdir(lmbench_path):
            if not json_file.endswith(".json"):
                continue
            if not json_file.startswith("result_lmbench-"):
                continue

            with open(os.path.join(lmbench_path, json_file)) as f:
                data = json.load(f)

            for entry in data:
                if "linux" in entry["extra"].lower():
                    linux_counts[json_file] += 1
                elif "aster" in entry["extra"].lower():
                    aster_counts[json_file] += 1

    max_linux = max(linux_counts.values()) if linux_counts else 0
    max_aster = max(aster_counts.values()) if aster_counts else 0
    print(f"Max Linux runs: {max_linux}, Max Asterinas runs: {max_aster}")

    # Second pass: collect all data
    for result_dir in result_dirs:
        if not result_dir.startswith("result-"):
            continue

        run_number = result_dir.split("-")[1]
        lmbench_path = os.path.join(root_dir, result_dir, "lmbench")
        if not os.path.exists(lmbench_path):
            continue

        for json_file in os.listdir(lmbench_path):
            if not json_file.endswith(".json"):
                continue
            if not json_file.startswith("result_lmbench-"):
                continue

            with open(os.path.join(lmbench_path, json_file)) as f:
                data = json.load(f)

            # Extract the file name without the prefix and suffix
            json_file = json_file.replace("result_lmbench-", "").replace(".json", "")

            # Get the command name from the file name
            command_name = file_command_map.get(json_file)

            for entry in data:
                # Some unit is MB/sec, some is MB/s, we should make them consistent
                if entry["unit"] == "MB/sec":
                    entry["unit"] = "MB/s"

                key = f"{command_name},{entry['unit']}"
                print(f"Processing {key} for run {run_number}")
                if "linux" in entry["extra"].lower():
                    results[key][run_number]["Linux"] = entry["value"]
                elif "aster" in entry["extra"].lower():
                    results[key][run_number]["Asterinas"] = entry["value"]

    # Prepare CSV output
    output_file = "lmbench_results.csv"
    headers = ["Command", "Unit", "Linux", "Asterinas", "Asterinas-stdev", "Normalized"]

    with open(output_file, "w", newline="") as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(headers)

        # Output with target sort order: command_headers
        for command in command_headers:
            unit = "MB/s"
            key = f"{command},{unit}"
            if key not in results:
                unit = "µs"
                key = f"{command},{unit}"
            if key not in results:
                print(f"Key {key} not found in results, skipping")
                continue

            row = [command, unit]

            # Calculate Linux average value
            linux_values = [
                float(results[key][run_number]["Linux"])
                for run_number in sorted(results[key].keys())
            ]
            linux_avg = sum(linux_values) / len(linux_values)
            row.append(linux_avg)

            # Calculate Asterinas stdev and average value
            aster_values = [
                float(results[key][run_number]["Asterinas"])
                for run_number in sorted(results[key].keys())
            ]
            aster_avg = sum(aster_values) / len(aster_values)
            aster_stdev = (
                sum((x - aster_avg) ** 2 for x in aster_values)
                / (len(aster_values) - 1)
            ) ** 0.5
            row.append(aster_avg)
            row.append(aster_stdev)

            # Calculate normalized value
            normalized_value = 0
            if unit == "MB/s":
                normalized_value = aster_avg / linux_avg
            else:
                normalized_value = linux_avg / aster_avg

            row.append(normalized_value)
            # Reserve 3
            row[2] = f"{row[2]:.3f}"
            row[3] = f"{row[3]:.3f}"
            row[4] = f"{row[4]:.3f}"
            row[5] = f"{row[5]:.2f}"
            print(f"Writing row: {row}")
            writer.writerow(row)

    print(f"Results saved to {output_file}")


if __name__ == "__main__":
    # Use first argument as root directory
    root_dir = "../../result-all"
    if len(sys.argv) > 1:
        root_dir = sys.argv[1]
    process_lmbench_results(root_dir)
