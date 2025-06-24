import os
import re
import matplotlib.pyplot as plt
from pprint import pprint

problem = "majority"
root_folder = "/home/main/Desktop/uni/courses/dsait4220_ridm/majority/"

# Regex to match "majority_<n>_<d>_<(un)sat>"


def str_to_instance(s):
    instance_pattern = re.compile(r"majority_(\d+)_(\d+)_(sat|unsat)")
    match = instance_pattern.fullmatch(name)
    if match:
        n = int(match.group(1))
        d = int(match.group(2))
        is_sat = match.group(3) == "sat"

        return (n, d, is_sat)


def instance_to_str(instance):
    (n, d, is_sat) = instance
    return f"majority_{n}_{d}_{'sat' if is_sat else 'unsat'}"


# Extract existing instance indices
# present_indices = []
# for name in os.listdir(root_folder):
#     match = instance_pattern.fullmatch(name)
#     if match:
#         present_indices.append(int(match.group(1)))


def instance_outcome(path):
    unknown = "=====UNKNOWN====="
    sat = "----------"
    unsat = "=====UNSATISFIABLE====="
    outcomes = {
        unknown: 0,
        sat: 1,
        unsat: 2,
    }
    with open(path, "r") as file:
        for line in file:
            line = line.strip()
            if line in outcomes:
                return outcomes[line]

    return -1


instances_per_n = {}
instances_per_d = {}

present_indices = []
for name in os.listdir(root_folder):
    instance = str_to_instance(name)
    if instance:
        (n, d, is_sat) = instance
        if n not in instances_per_n:
            instances_per_n[n] = []
        instances_per_n[n].append(instance)
        if d not in instances_per_d:
            instances_per_d[d] = []
        instances_per_d[d].append(instance)

        instance_path = os.path.join(root_folder, name)
        out_folder = os.path.join(instance_path, "out")

        # Check if 'out' folder exists and contains exactly 4 files
        if os.path.isdir(out_folder):
            out_files = os.listdir(out_folder)
            if len(out_files) == 4:
                full_paths = [os.path.join(out_folder, f) for f in out_files]

                # Check if all files contain "=========="
                contain_marker = [instance_outcome(path) >= 0 for path in full_paths]
                all_contain_marker = all(contain_marker)

                if all_contain_marker:
                    present_indices.append(instance)

present_indices.sort()
for lst in instances_per_n.values():
    lst.sort()
for lst in instances_per_d.values():
    lst.sort()

pprint("present indices: " + str(present_indices))


# Map filenames to method names
method_map = {
    "decomposition_output.txt": "Decomposition",
    "extended-resolution_output.txt": "Extended Res",
    "regin-arc-consistent_output.txt": "Regin",
    "extended-resolution-with-regin_output.txt": "Extended Regin",
}

# Metrics regex
patterns = {
    "decisions": re.compile(r"engineStatisticsNumDecisions=(\d+)"),
    "conflicts": re.compile(r"engineStatisticsNumConflicts=(\d+)"),
    "lbd": re.compile(r"learnedClauseStatisticsAverageLbd=([\d.]+)"),
    "time": re.compile(r"engineStatisticsTimeSpentInSolver=([\d.]+)"),
}

# Initialize stats dict
stats = {method: [] for method in method_map.values()}
stats_per_instance = {}


def update_spi(method, instance, stat):
    if method not in stats_per_instance:
        stats_per_instance[method] = {}

    stats_per_instance[method][instance] = stat


# Extraction helper
def extract_stats(filepath):
    with open(filepath, "r") as f:
        content = f.read()
    outcome = instance_outcome(filepath)
    data = {"outcome": outcome}
    if outcome < 0:
        return
    else:
        for key, pattern in patterns.items():
            match = pattern.search(content)
            if match:
                value = (
                    float(match.group(1))
                    if "." in match.group(1)
                    else int(match.group(1))
                )
                data[key] = value
            else:
                data[key] = None
    return data


# Traverse each instance folder
for instance in present_indices:
    name = instance_to_str(instance)
    instance_folder = os.path.join(root_folder, name, "out")
    for filename in os.listdir(instance_folder):
        if filename in method_map:
            method = method_map[filename]
            file_path = os.path.join(instance_folder, filename)
            stat = extract_stats(file_path)
            stats[method].append(stat)
            update_spi(method, instance, stat)

pprint(stats_per_instance)

# def plot_metric_comparison_scatter(metric_name, method_x, method_y, xlabel=None, ylabel=None):
#     """
#     Plots a scatter plot comparing two methods for a given metric.
#
#     :param metric_name: Metric key (e.g., "time", "decisions", "conflicts", "lbd")
#     :param method_x: Name of method for x-axis (e.g., "Decomposition")
#     :param method_y: Name of method for y-axis (e.g., "Regin")
#     :param xlabel: Optional custom label for x-axis
#     :param ylabel: Optional custom label for y-axis
#     """
#     # Get metric values
#     x_vals = [run[metric_name] if run[metric_name] is not None else 0 for run in stats[method_x]]
#     y_vals = [run[metric_name] if run[metric_name] is not None else 0 for run in stats[method_y]]
#
#     assert len(x_vals) == len(y_vals), "Mismatched instance count"
#
#     plt.figure(figsize=(6, 6))
#     plt.scatter(x_vals, y_vals, color='blue', label='Instances')
#
#     # Plot y = x reference line
#     max_val = max(max(x_vals), max(y_vals))
#     plt.plot([0, max_val], [0, max_val], 'r--', label='y = x')
#
#     # for i, (x, y) in enumerate(zip(x_vals, y_vals)):
#     #     plt.annotate(f"{i+1}", (x, y), textcoords="offset points", xytext=(5, 5), ha='left')
#
#     # Labels and legend
#     plt.xlabel(xlabel or f"{metric_name.capitalize()} - {method_x}")
#     plt.ylabel(ylabel or f"{metric_name.capitalize()} - {method_y}")
#     plt.title(f"{metric_name.capitalize()} Comparison: {method_x} vs {method_y}")
#     plt.grid(True)
#     plt.legend()
#     plt.tight_layout()
#     plt.savefig(f"{metric_name}_comparison_{method_x.lower()}_vs_{method_y.lower()}.png")
#     plt.show()


def plot_metric_comparison_scatter(
    metric_name, method_x, method_y, xlabel=None, ylabel=None
):
    """
    Plots a scatter plot comparing two methods for a given metric.

    :param metric_name: Metric key (e.g., "time", "decisions", "conflicts", "lbd")
    :param method_x: Name of method for x-axis (e.g., "Decomposition")
    :param method_y: Name of method for y-axis (e.g., "Regin")
    :param xlabel: Optional custom label for x-axis
    :param ylabel: Optional custom label for y-axis
    """
    # Get metric values
    x_vals = [
        run[metric_name] if run[metric_name] is not None else 0.0001
        for run in stats[method_x]
    ]
    y_vals = [
        run[metric_name] if run[metric_name] is not None else 0.0001
        for run in stats[method_y]
    ]

    assert len(x_vals) == len(y_vals), "Mismatched instance count"

    plt.figure(figsize=(6, 6))
    plt.scatter(x_vals, y_vals, color="blue", label="Instances")

    # Plot y = x reference line
    min_val = min(min(x_vals), min(y_vals))
    max_val = max(max(x_vals), max(y_vals))
    plt.plot([min_val, max_val], [min_val, max_val], "r--", label="y = x")

    # Logarithmic axis if the metric is "time"
    if metric_name == "time":
        plt.xscale("log")
        plt.yscale("log")

    # Labels and legend
    plt.xlabel(xlabel or f"{metric_name.capitalize()} - {method_x}")
    plt.ylabel(ylabel or f"{metric_name.capitalize()} - {method_y}")
    plt.title(f"{metric_name.capitalize()} Comparison: {method_x} vs {method_y}")
    plt.grid(True, which="both", ls="--")
    plt.legend()
    plt.tight_layout()
    plt.savefig(
        f"{metric_name}_comparison_{method_x.lower()}_vs_{method_y.lower()}.png"
    )
    plt.show()


def plot_all_metrics_comparison_grid(
    method_x, method_y, metrics=("time", "decisions", "conflicts", "lbd"), save=True
):
    """
    Plots a 2x2 grid of scatter plots comparing method_x and method_y on multiple metrics.

    :param method_x: Name of method for x-axis (e.g., "Extended Res")
    :param method_y: Name of method for y-axis (e.g., "Decomposition")
    :param metrics: List of metric names to compare (default: all 4)
    :param save: If True, saves the figure as PNG
    """
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    axes = axes.flatten()

    for i, metric_name in enumerate(metrics):
        x_vals = [
            run[metric_name] if run[metric_name] is not None else 0.0001
            for run in stats[method_x]
        ]
        y_vals = [
            run[metric_name] if run[metric_name] is not None else 0.0001
            for run in stats[method_y]
        ]
        print("x_vals length: " + str(len(x_vals)))
        print("y_vals length: " + str(len(y_vals)))

        assert len(x_vals) == len(y_vals), "Mismatched instance count"

        ax = axes[i]
        ax.scatter(x_vals, y_vals, color="blue", label="Instances")

        # Reference line y = x
        min_val = min(min(x_vals), min(y_vals))
        max_val = max(max(x_vals), max(y_vals))
        ax.plot([min_val, max_val], [min_val, max_val], "r--", label="y = x")

        # Log scale for "time"
        if metric_name == "time":
            ax.set_xscale("log")
            ax.set_yscale("log")

        ax.set_xlabel(f"{metric_name.capitalize()} - {method_x}")
        ax.set_ylabel(f"{metric_name.capitalize()} - {method_y}")
        ax.set_title(f"{metric_name.capitalize()} Comparison")
        ax.grid(True, which="both", ls="--")
        ax.legend()

    # plt.suptitle(f"{method_x} vs {method_y} - Metric Comparison", fontsize=16)
    plt.tight_layout(rect=[0, 0.03, 1, 0.95])

    if save:
        filename = f"comparison_{method_x.lower().replace(' ', '_')}_vs_{method_y.lower().replace(' ', '_')}.png"
        plt.savefig(filename)

    plt.show()


def plot_runtime_conflicts_comparison_grid(
    method_x, method_y, metrics=("time", "conflicts"), save=True
):
    fig, axes = plt.subplots(1, 2, figsize=(12, 6))
    axes = axes.flatten()

    for i, metric_name in enumerate(metrics):
        x_vals = [
            run[metric_name] if run[metric_name] is not None else 0.0001
            for run in stats[method_x]
        ]
        y_vals = [
            run[metric_name] if run[metric_name] is not None else 0.0001
            for run in stats[method_y]
        ]
        print("x_vals length: " + str(len(x_vals)))
        print("y_vals length: " + str(len(y_vals)))

        assert len(x_vals) == len(y_vals), "Mismatched instance count"

        ax = axes[i]
        ax.scatter(x_vals, y_vals, color="blue", label="Instances")

        # Reference line y = x
        min_val = min(min(x_vals), min(y_vals))
        max_val = max(max(x_vals), max(y_vals))
        ax.plot([min_val, max_val], [min_val, max_val], "r--", label="y = x")

        # Log scale for "time"
        if metric_name == "time":
            ax.set_xscale("log")
            ax.set_yscale("log")

        ax.set_xlabel(f"{metric_name.capitalize()} - {method_x}")
        ax.set_ylabel(f"{metric_name.capitalize()} - {method_y}")
        ax.set_title(f"{metric_name.capitalize()} Comparison")
        ax.grid(True, which="both", ls="--")
        ax.legend()

    # plt.suptitle(f"{method_x} vs {method_y} - Metric Comparison", fontsize=16)
    plt.tight_layout(rect=[0, 0.03, 1, 0.95])

    if save:
        filename = f"{problem}_{method_x.lower().replace(' ', '_')}_vs_{method_y.lower().replace(' ', '_')}.png"
        save_dir = "figures"  # <-- your target directory
        os.makedirs(save_dir, exist_ok=True)  # create it if it doesn't exist
        filepath = os.path.join(save_dir, filename)
        plt.savefig(filepath)

    plt.show()


def plot_points(
    method_a, method_b, over_k="options", metrics=("time", "conflicts"), save=True
):
    instances_per_k = {"options": instances_per_d, "people": instances_per_n}[over_k]

    fig, axes = plt.subplots(1, 2, figsize=(12, 6))
    axes = axes.flatten()

    for i, metric in enumerate(metrics):
        ax = axes[i]

        # points_per_marker = {"x": [], "o": []}
        linestyles = {
            method_a: "-",
            method_b: "--",
        }
        points_per = {}
        # marker_titles = {"x": "Instances (timed out)", "o": "Instances"}
        # x = []
        # y = []
        # color = []
        # marker = []

        for k in instances_per_k:
            for instance in instances_per_k[k]:
                (n, d, is_sat) = instance
                if is_sat:
                    continue
                # if n  5?tinue

                for method_index, method in enumerate([method_a, method_b]):
                    stat = stats_per_instance[method][instance]
                    x = k
                    y = stat[metric]
                    marker = ["o", "^", "o"][method_index]
                    color = ["orange", "blue", "green", "red"][method_index]

                    if stat["outcome"] == 0:
                        continue

                    point = (
                        x,
                        y,
                        color,
                    )

                    index = (method_index, n, marker)
                    if index not in points_per:
                        points_per[index] = []

                    points_per[index].append(point)
                    # points_per_marker[marker].append(point)

        # for marker, points in points_per_marker.items():
        #     (x, y, c) = list(zip(*points))
        #     ax.scatter(x, y, color=c, marker=marker, label=marker_titles[marker])
        for (method_index, n, marker), points in points_per.items():
            method = [method_a, method_b][method_index]
            linestyle = linestyles[method]
            points.sort(key=lambda k: k[0])
            (x, y, c) = list(zip(*points))
            color = ["orange", "blue", "green", "red"][n - 5]
            ax.plot(x, y, c=f"C{n - 5}", marker=marker, linestyle=linestyle)

        ax.plot(
            [],
            [],
            c="black",
            marker="o",
            label=f"{method_a}",
            linestyle=linestyles[method_a],
        )
        ax.plot(
            [],
            [],
            c="black",
            marker="^",
            label=f"{method_b}",
            linestyle=linestyles[method_b],
        )

        # for _x, _y, _c, _m in zip(x, y, color, marker):
        #     ax.scatter(_x, _y, c=_c, marker=_m)
        ax.set_xscale("log")
        ax.set_yscale("log")

        ax.set_xlabel(f"Num of {over_k.capitalize()}")
        ax.set_ylabel(f"{metric.capitalize()}")
        ax.set_title(f"{metric.capitalize()} Comparison")
        ax.grid(True, which="both", ls="--")
        ax.legend()

    plt.tight_layout(rect=[0, 0.03, 1, 0.95])

    if save:
        filename = f"{problem}_over_{over_k}_{method_a.lower().replace(' ', '_')}_vs_{method_b.lower().replace(' ', '_')}.png"
        save_dir = "figures"  # <-- your target directory
        os.makedirs(save_dir, exist_ok=True)  # create it if it doesn't exist
        filepath = os.path.join(save_dir, filename)
        plt.savefig(filepath)

    plt.show()


plot_points("Extended Res", "Regin")

# only plot runtime and conflicts:
# plot_runtime_conflicts_comparison_grid("Extended Res", "Decomposition")
# plot_runtime_conflicts_comparison_grid("Extended Regin", "Regin")
# plot_runtime_conflicts_comparison_grid("Extended Res", "Extended Regin")


# Extended Res comparisons
# plot_all_metrics_comparison_grid("Extended Res", "Decomposition")
#
# # Extended Regin comparisons
# plot_all_metrics_comparison_grid("Extended Regin", "Regin")
#
# # Extended Res vs Extended Regin
# plot_all_metrics_comparison_grid("Extended Res", "Extended Regin")


# metrics: time, decisions, conflicts, and lbd
# methods: Decomposition (subst for decomposition), Regin, Extended Res, Extend Regin

# comparisons:

# Extended Res:
# extended res vs basic filter
# plot_metric_comparison_scatter("time", "Extended Res", "Basic Filter")
# plot_metric_comparison_scatter("decisions", "Extended Res", "Basic Filter")
# plot_metric_comparison_scatter("conflicts", "Extended Res", "Basic Filter")
# plot_metric_comparison_scatter("lbd", "Extended Res", "Basic Filter")
# # extended res vs regin
# plot_metric_comparison_scatter("time", "Extended Res", "Regin")
# plot_metric_comparison_scatter("decisions", "Extended Res", "Regin")
# plot_metric_comparison_scatter("conflicts", "Extended Res", "Regin")
# plot_metric_comparison_scatter("lbd", "Extended Res", "Regin")
#
# # Extended Regin:
# # extended regin vs basic filter
# plot_metric_comparison_scatter("time", "Extended Regin", "Basic Filter")
# plot_metric_comparison_scatter("decisions", "Extended Regin", "Basic Filter")
# plot_metric_comparison_scatter("conflicts", "Extended Regin", "Basic Filter")
# plot_metric_comparison_scatter("lbd", "Extended Regin", "Basic Filter")
# # extended regin vs regin
# plot_metric_comparison_scatter("time", "Extended Regin", "Regin")
# plot_metric_comparison_scatter("decisions", "Extended Regin", "Regin")
# plot_metric_comparison_scatter("conflicts", "Extended Regin", "Regin")
# plot_metric_comparison_scatter("lbd", "Extended Regin", "Regin")
#
# # Extended Res vs Extended Regin
# plot_metric_comparison_scatter("time", "Extended Res", "Extended Regin")
# plot_metric_comparison_scatter("decisions", "Extended Res", "Extended Regin")
# plot_metric_comparison_scatter("conflicts", "Extended Res", "Extended Regin")
# plot_metric_comparison_scatter("lbd", "Extended Res", "Extended Regin")

# random examples:
# plot_metric_comparison_scatter("time", "Basic Filter", "Regin")
# plot_metric_comparison_scatter("time", "Extended Res", "Regin")
# plot_metric_comparison_scatter("decisions", "Extended Res", "Regin")
# plot_metric_comparison_scatter("conflicts", "Extended Res", "Regin")
# plot_metric_comparison_scatter("lbd", "Extended Res", "Regin")
