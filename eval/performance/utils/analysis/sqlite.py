import os
import csv
import sys
from collections import defaultdict
import numpy as np
import matplotlib.pyplot as plt
import re


def collect_data_from_txt(file_path):
    with open(file_path, "r") as f:
        content = f.read()

    # find sqlite
    version_match = re.search(
        r"SQLite \d+\.\d+\.\d+ \d{4}-\d{2}-\d{2} [0-9a-f]+", content
    )
    if not version_match:
        return {}

    start_pos = version_match.end()
    total_match = re.search(r"TOTAL.*?([\d\.]+)s", content[start_pos:])
    if not total_match:
        return {}

    data_block = content[start_pos : start_pos + total_match.start()]

    # Extract the data in each line
    pattern = re.compile(r"^\s*(\d+)\s*-\s*(.+?)[\. ]+\s*([\d\.]+)s", re.MULTILINE)
    results = {}
    test_name_map = {}

    for match in pattern.finditer(data_block):
        test_value = match.group(1)
        test_name = match.group(2).strip()
        time = match.group(3)
        results[test_value] = time
        test_name_map[test_value] = test_name

    # Add total
    total_time = total_match.group(1)
    results[999] = total_time
    test_name_map[999] = "TOTAL"
    return (results, test_name_map)


def process_txt_files(root_dir):
    results = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))
    value_name_map = {}
    # Find aster-output.txt, linux-output.txt, and without-iommu-aster-output.txt files under root_dir/result-{test_number}/sqlite, and collect data
    for subdir in os.listdir(root_dir):
        result_dir = os.path.join(root_dir, subdir, "sqlite")
        # Extract the test number
        if not subdir.startswith("result-"):
            continue
        test_number = subdir.split("-")[1]

        files = [
            "aster_output.txt",
            "linux_output.txt",
            "aster_output_no_iommu.txt",
        ]

        # Open files, and collect data to results
        # The first file is Asterinas, the second is Linux
        (data, value_name_map) = collect_data_from_txt(
            os.path.join(result_dir, files[0])
        )
        for key, value in data.items():
            results[key][test_number]["Asterinas"] = value
        (data, _) = collect_data_from_txt(os.path.join(result_dir, files[1]))
        for key, value in data.items():
            results[key][test_number]["Linux"] = value
        (data, _) = collect_data_from_txt(os.path.join(result_dir, files[2]))
        for key, value in data.items():
            results[key][test_number]["Asterinas no IOMMU"] = value

    # Prepare output CSV
    output_filename = "sqlite_results.csv"
    headers = [
        "Number",
        "Test Name",
        "Linux(s)",
        "Asterinas(s)",
        "Asterinas no IOMMU(s)",
    ]

    avg_results = defaultdict(lambda: defaultdict(dict))
    with open(output_filename, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(headers)

        # Write data to CSV
        for number in results.keys():
            # Get the average values for Linux, Asterinas, and Asterinas no IOMMU
            linux_values = [
                float(results[number][test_number]["Linux"])
                for test_number in sorted(results[number].keys())
            ]
            asterinas_values = [
                float(results[number][test_number]["Asterinas"])
                for test_number in sorted(results[number].keys())
            ]
            asterinas_no_iommu_values = [
                float(results[number][test_number]["Asterinas no IOMMU"])
                for test_number in sorted(results[number].keys())
            ]
            linux_avg = sum(linux_values) / len(linux_values)
            asterinas_avg = sum(asterinas_values) / len(asterinas_values)
            asterinas_no_iommu_avg = sum(asterinas_no_iommu_values) / len(
                asterinas_no_iommu_values
            )
            # Write the row to the CSV
            row = [
                number,
                value_name_map[number],
                linux_avg,
                asterinas_avg,
                asterinas_no_iommu_avg,
            ]
            avg_results[number]["Linux"] = linux_avg
            avg_results[number]["Asterinas"] = asterinas_avg
            avg_results[number]["Asterinas no IOMMU"] = asterinas_no_iommu_avg
            # Format the values to 2 decimal places
            row[2] = f"{row[2]:.2f}"
            row[3] = f"{row[3]:.2f}"
            row[4] = f"{row[4]:.2f}"
            writer.writerow(row)
            print(f"Writing row: {row}")

    # Draw the graph
    test_number = ["120", "200", "230", "400", "410"]
    labels = ["Linux", "Asterinas", "Asterinas no IOMMU"]
    x = np.arange(len(test_number))
    width = 0.25

    linux_vals = [
        avg_results[op]["Linux"] if op in avg_results else 0 for op in test_number
    ]
    asterinas_vals = [
        avg_results[op]["Asterinas"] if op in avg_results else 0 for op in test_number
    ]
    asterinas_no_iommu_vals = [
        avg_results[op]["Asterinas no IOMMU"] if op in avg_results else 0
        for op in test_number
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

    ax.set_ylabel("Latency(s)")
    ax.set_xlabel("Test Number")
    ax.set_xticks(x)
    ax.set_xticklabels(test_number)
    ax.legend()
    plt.tight_layout()
    plt.savefig("sqlite_results_bar.png")
    plt.close()

    print(f"Combined results saved to {output_filename}")
    print("Graph saved as sqlite_results_bar.png")


if __name__ == "__main__":
    root_dir = "../../result-all"
    if len(sys.argv) > 1:
        root_dir = sys.argv[1]
    process_txt_files(root_dir)
