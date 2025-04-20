import os
import csv
import numpy as np
import matplotlib.pyplot as plt


def read_second_column(file_path):
    values = []
    with open(file_path, newline='') as f:
        reader = csv.reader(f)
        for row in reader:
            if len(row) < 2:
                continue
            try:
                values.append(int(row[1]))
            except ValueError:
                continue
    return values


def main():
    base_dirs = [f"{i}.0" for i in range(1, 31)]
    request_types = ["sharing", "signing", "generate"]

    means = []
    stds = []

    for req in request_types:
        all_values = []
        for d in base_dirs:
            path = os.path.join(d, f"{req}.csv")
            if not os.path.isfile(path):
                raise FileNotFoundError(f"Missing file: {path}")
            vals = read_second_column(path)
            all_values.extend(vals)

        mean_val = np.mean(all_values)
        std_val = np.std(all_values, ddof=1)
        means.append(mean_val)
        stds.append(std_val)

        print(f"{req}: mean={mean_val:.2f}, std={std_val:.2f}")

    plt.figure()
    x = np.arange(len(request_types))
    plt.bar(x, means, yerr=stds, capsize=5)
    plt.xticks(x, request_types)
    plt.ylabel("Value")
    plt.title("Mean and Standard Deviation per Request Type (30 samples)")
    plt.tight_layout()
    plt.show()


if __name__ == "__main__":
    main()
