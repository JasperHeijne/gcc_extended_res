import os
import re
import matplotlib.pyplot as plt
import numpy as np

# Path to your folder
folder = "C:/Users/jaspe/Documents/TuDelft/Reasearch_Intelligent_Decision_Making/pumpkin-gcc_extended_res/gcc_extended_res/test_bench/output_community_detection"
files = sorted([f for f in os.listdir(folder) if f.endswith(".txt")])

# Assuming file order: 5 per method
methods = ["Basic Filter", "Decomposition", "Regin"]
num_instances = 5

# Initialize storage
stats = {
    "Basic Filter": [],
    "Decomposition": [],
    "Regin": []
}

# Regex patterns for relevant stats
patterns = {
    "decisions": re.compile(r"engineStatisticsNumDecisions=(\d+)"),
    "conflicts": re.compile(r"engineStatisticsNumConflicts=(\d+)"),
    "lbd": re.compile(r"learnedClauseStatisticsAverageLbd=([\d\.]+)"),
    "time": re.compile(r"% time elapsed: ([\d\.]+) s")
}

# Helper function to extract stats from a file
def extract_stats(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    data = {}
    for key, pattern in patterns.items():
        match = pattern.search(content)
        if match:
            value = float(match.group(1)) if '.' in match.group(1) else int(match.group(1))
            data[key] = value
        else:
            data[key] = None
    return data

# Extract stats and group by method
for i, filename in enumerate(files):
    method_index = i // num_instances
    method = methods[method_index]
    path = os.path.join(folder, filename)
    stats[method].append(extract_stats(path))

# Transpose data for plotting (per metric)
def get_metric_per_method(metric):
    return {
        method: [run[metric] for run in runs]
        for method, runs in stats.items()
    }

# Plotting function
def plot_metric(metric_name, ylabel):
    metric_data = get_metric_per_method(metric_name)
    x = range(1, num_instances + 1)

    plt.figure(figsize=(8, 5))
    for method, values in metric_data.items():
        plt.plot(x, values, marker='o', label=method)
    # plt.title(f"Comparison of {ylabel} Across Problem Instances")
    plt.xlabel("Problem Instance")
    plt.ylabel(ylabel)
    plt.xticks(x)
    plt.legend()
    plt.grid(True)
    plt.tight_layout()
    plt.savefig(f"{metric_name}_comparison.png")
    plt.show()

def plot_metric_bar(metric_name, ylabel):
    metric_data = get_metric_per_method(metric_name)
    x = np.arange(num_instances)  # Positions for problem instances
    width = 0.25  # Width of each bar

    method_list = list(metric_data.keys())
    temp = method_list[1]
    method_list[1] = method_list[0]
    method_list[0] = temp
    num_methods = len(method_list)

    plt.figure(figsize=(10, 6))

    # Plot each method's bars offset from x
    for i, method in enumerate(method_list):
        offset = (i - 1) * width  # centers the bars around the instance
        print("offset: " + str(offset))
        print("x: " + str(x))
        values = [v if v is not None else 0 for v in metric_data[method]]
        plt.bar(x + offset, values, width, label=method)

    # Axis and labels
    plt.xlabel("Problem Instance")
    plt.ylabel(ylabel)
    # plt.title(f"{ylabel} Comparison per Problem Instance")
    plt.xticks(x, [f"Instance {i+1}" for i in x])
    plt.legend()
    plt.grid(axis='y')
    plt.tight_layout()
    plt.savefig(f"{metric_name}_comparison_bar.png")
    plt.show()

# Plot all metrics
plot_metric_bar("decisions", "Number of Decisions")
plot_metric_bar("conflicts", "Number of Conflicts")
plot_metric_bar("lbd", "Average LBD")
plot_metric_bar("time", "Runtime (s)")
