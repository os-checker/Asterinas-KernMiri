import os
import csv
import sys
from collections import defaultdict
import numpy as np
import json
import matplotlib.pyplot as plt

map_files = {
    "file4KB": "4KB",
    "file8KB": "8KB",
    "file16KB": "16KB",
    "file32KB": "32KB",
    "file64KB": "64KB",
}


def process_csv_files(root_dir):
    results = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))

    # Find all result_nginx-*.json files under root_dir/result-{test_number}/nginx, and collect data
    for subdir in os.listdir(root_dir):
        result_dir = os.path.join(root_dir, subdir, "nginx")
        # Extract the test number
        if not subdir.startswith("result-"):
            continue
        test_number = subdir.split("-")[1]

        with_iommu_files = []
        without_iommu_files = []
        for file in os.listdir(result_dir):
            if file.startswith("result_nginx-") and file.endswith("no_iommu.json"):
                without_iommu_files.append(file)
                continue
            if file.startswith("result_nginx-") and file.endswith(".json"):
                with_iommu_files.append(file)

        # Open with IOMMU files, and collect data to results
        for file in with_iommu_files:
            key = ""
            for k, v in map_files.items():
                if k in file:
                    key = v
                    break
            json_path = os.path.join(result_dir, file)
            with open(json_path, "r") as jf:
                json_data = json.load(jf)
                for entry in json_data:
                    if entry["extra"] == "aster_result":
                        results[key][test_number]["Asterinas"] = entry["value"]
                    elif entry["extra"] == "linux_result":
                        results[key][test_number]["Linux"] = entry["value"]

        for file in without_iommu_files:
            key = ""
            for k, v in map_files.items():
                if k in file:
                    key = v
                    break
            json_path = os.path.join(result_dir, file)
            with open(json_path, "r") as jf:
                json_data = json.load(jf)
                for entry in json_data:
                    if entry["extra"] == "aster_result":
                        results[key][test_number]["Asterinas no IOMMU"] = entry["value"]

    # Prepare output CSV
    output_filename = "nginx_results.csv"
    headers = ["File size", "Linux(rps)", "Asterinas(rps)", "Asterinas no IOMMU(rps)"]

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
    operations = ["4KB", "8KB", "16KB", "32KB", "64KB"]
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
    ax.set_xlabel("File size")
    ax.set_xticks(x)
    ax.set_xticklabels(operations)
    ax.legend()
    plt.tight_layout()
    plt.savefig("nginx_results_bar.png")
    plt.close()

    print(f"Combined results saved to {output_filename}")
    print("Graph saved as nginx_results_bar.png")


if __name__ == "__main__":
    root_dir = "../../result-all"
    if len(sys.argv) > 1:
        root_dir = sys.argv[1]
    process_csv_files(root_dir)
