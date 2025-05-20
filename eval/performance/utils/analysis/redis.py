import os
import csv
import sys
from collections import defaultdict
import numpy as np
import matplotlib.pyplot as plt


def collect_data_from_csv(file_path):
    data = {}
    with open(file_path, "r") as f:
        reader = csv.reader(f)
        header = next(reader)  # Skip the header
        for row in reader:
            test_name = row[0]
            rps = row[1]
            # Skip the unwanted row
            if "LPUSH (needed to benchmark LRANGE)" in test_name:
                continue
            data[test_name] = rps
    return data


def process_csv_files(root_dir):
    results = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))

    # Find all all-{number}.csv files under root_dir/result-{test_number}/redis, and collect data
    for subdir in os.listdir(root_dir):
        result_dir = os.path.join(root_dir, subdir, "redis")
        # Extract the test number
        if not subdir.startswith("result-"):
            continue
        test_number = subdir.split("-")[1]

        target_csv_files = []
        no_iommu_csv_files = []
        for file in os.listdir(result_dir):
            if file.startswith("all-") and file.endswith(".csv"):
                target_csv_files.append(file)
            if file.startswith("without-iommu-all-") and file.endswith(".csv"):
                no_iommu_csv_files.append(file)

        if len(target_csv_files) != 2 or len(no_iommu_csv_files) != 2:
            print(
                f"Warning: Expected 2 CSV files in {result_dir}, found {len(target_csv_files)}. Skipping this directory."
            )
            continue

        # Extract the pid from the file names, "all-{pid}.csv" and "without-iommu-all-{pid}.csv", then sort by pid
        # The smaller pid is Asterinas, the larger pid is Linux
        target_csv_files.sort(key=lambda x: int(x.split("-")[1].split(".")[0]))
        no_iommu_csv_files.sort(key=lambda x: int(x.split("-")[3].split(".")[0]))

        # Open files, and collect data to results
        # The first file is Asterinas, the second is Linux
        data = collect_data_from_csv(os.path.join(result_dir, target_csv_files[0]))
        for key, value in data.items():
            results[key][test_number]["Asterinas"] = value
        data = collect_data_from_csv(os.path.join(result_dir, target_csv_files[1]))
        for key, value in data.items():
            results[key][test_number]["Linux"] = value
        data = collect_data_from_csv(os.path.join(result_dir, no_iommu_csv_files[0]))
        for key, value in data.items():
            results[key][test_number]["Asterinas no IOMMU"] = value

    # Prepare output CSV
    output_filename = "redis_results.csv"
    headers = ["Operation", "Linux(rps)", "Asterinas(rps)", "Asterinas no IOMMU(rps)"]

    avg_results = defaultdict(lambda: defaultdict(dict))
    with open(output_filename, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(headers)

        # Write data to CSV
        for operation in results.keys():
            # Get the average values for Linux, Asterinas, and Asterinas no IOMMU
            linux_values = [
                float(results[operation][test_number]["Linux"])
                for test_number in sorted(results[operation].keys())
            ]
            asterinas_values = [
                float(results[operation][test_number]["Asterinas"])
                for test_number in sorted(results[operation].keys())
            ]
            asterinas_no_iommu_values = [
                float(results[operation][test_number]["Asterinas no IOMMU"])
                for test_number in sorted(results[operation].keys())
            ]
            linux_avg = sum(linux_values) / len(linux_values)
            asterinas_avg = sum(asterinas_values) / len(asterinas_values)
            asterinas_no_iommu_avg = sum(asterinas_no_iommu_values) / len(
                asterinas_no_iommu_values
            )
            # Write the row to the CSV
            row = [operation, linux_avg, asterinas_avg, asterinas_no_iommu_avg]
            avg_results[operation]["Linux"] = linux_avg
            avg_results[operation]["Asterinas"] = asterinas_avg
            avg_results[operation]["Asterinas no IOMMU"] = asterinas_no_iommu_avg
            # Format the values to 2 decimal places
            row[1] = f"{row[1]:.2f}"
            row[2] = f"{row[2]:.2f}"
            row[3] = f"{row[3]:.2f}"
            writer.writerow(row)
            print(f"Writing row: {row}")

    # Draw the graph
    operations = ["SET", "GET", "LPUSH", "LPOP", "LRANGE_600 (first 600 elements)"]
    labels = ["Linux", "Asterinas", "Asterinas no IOMMU"]
    x = np.arange(len(operations))
    width = 0.25

    linux_vals = [
        avg_results[op]["Linux"] if op in avg_results else 0 for op in operations
    ]
    asterinas_vals = [
        avg_results[op]["Asterinas"] if op in avg_results else 0 for op in operations
    ]
    asterinas_no_iommu_vals = [
        avg_results[op]["Asterinas no IOMMU"] if op in avg_results else 0
        for op in operations
    ]

    fig, ax = plt.subplots(figsize=(10, 6))

    ax.bar(
        x - width, linux_vals, width, label="Linux", color="#808080", edgecolor="black"
    )
    ax.bar(
        x, asterinas_vals, width, label="Asterinas", color="#b4e9ad", edgecolor="black"
    )
    ax.bar(
        x + width,
        asterinas_no_iommu_vals,
        width,
        label="Asterinas no IOMMU",
        color="#fad7a0",
        edgecolor="black",
    )

    ax.set_ylabel("Throughput(rps)")
    ax.set_xlabel("Operation")
    ax.set_xticks(x)
    ax.set_xticklabels(operations)
    ax.legend()
    plt.tight_layout()
    plt.savefig("redis_results_bar.png")
    plt.close()

    print(f"Combined results saved to {output_filename}")
    print("Graph saved as redis_results_bar.png")


if __name__ == "__main__":
    root_dir = "../../result-all"
    if len(sys.argv) > 1:
        root_dir = sys.argv[1]
    process_csv_files(root_dir)
