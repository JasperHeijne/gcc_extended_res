import os
import re
import matplotlib.pyplot as plt
import shutil

import numpy as np

root_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\draft_runs\runs\vaccine"

# Regex to match "instance_<number>"
instance_pattern = re.compile(r"instance_(\d+)")
present_indices = []
for name in os.listdir(root_folder):
    match = instance_pattern.fullmatch(name)
    if match:
        instance_path = os.path.join(root_folder, name)
        out_folder = os.path.join(instance_path, "out")

        # Check if 'out' folder exists and contains exactly 4 files
        if os.path.isdir(out_folder):
            out_files = os.listdir(out_folder)
            if len(out_files) == 4:
                full_paths = [os.path.join(out_folder, f) for f in out_files]

                # Check if all files contain "=========="
                all_contain_marker = all(
                    "==========" in open(path, 'r', encoding='utf-8', errors='ignore').read()
                    for path in full_paths
                )
                if all_contain_marker:
                    present_indices.append(int(match.group(1)))
                # present_indices.append(int(match.group(1)))

present_indices.sort()
print("present indices: " + str(present_indices))

# Map filenames to method names
method_map = {
    "decomposition_output.txt": "Decomposition",
    "extended-resolution_output.txt": "Extended Res",
    "regin-arc-consistent_output.txt": "Regin",
    "extended-resolution-with-regin_output.txt": "Extended Regin"
}

# Metrics regex
patterns = {
    "decisions": re.compile(r"engineStatisticsNumDecisions=(\d+)"),
    "conflicts": re.compile(r"engineStatisticsNumConflicts=(\d+)"),
    "lbd": re.compile(r"learnedClauseStatisticsAverageLbd=([\d.]+)"),
    "time": re.compile(r"engineStatisticsTimeSpentInSolver=([\d.]+)")
}

# Initialize stats dict
stats = {method: {} for method in method_map.values()}

# Extraction helper
def extract_stats(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    data = {}
    if not re.compile("==========").search(content):
        pass
    else:
        for key, pattern in patterns.items():
            match = pattern.search(content)
            if match:
                value = float(match.group(1)) if '.' in match.group(1) else int(match.group(1))
                data[key] = value
            else:
                data[key] = None
    return data

# Traverse each instance folder
for i in present_indices:
    instance_folder = os.path.join(root_folder, f"instance_{i}", "out")
    for filename in os.listdir(instance_folder):
        if filename in method_map:
            method = method_map[filename]
            file_path = os.path.join(instance_folder, filename)
            stats[method][i] = extract_stats(file_path)

def get_conflict_outliers(method_x, method_y, top_k=5):
    common_indices = list(set(stats[method_x].keys()) & set(stats[method_y].keys()))
    x_vals = [stats[method_x][i]["conflicts"] if stats[method_x][i]["conflicts"] is not None else 0.0001 for i in common_indices]
    y_vals = [stats[method_y][i]["conflicts"] if stats[method_y][i]["conflicts"] is not None else 0.0001 for i in common_indices]

    distances = [abs(x - y) for x, y in zip(x_vals, y_vals)]
    sorted_indices = sorted(range(len(distances)), key=lambda i: distances[i], reverse=True)

    # Return instance index + data
    return [(common_indices[i], x_vals[i], y_vals[i], distances[i]) for i in sorted_indices[:top_k]]


def plot_conflict_outliers(method_x, method_y, top_k=5, save=True):
    outliers = get_conflict_outliers(method_x, method_y, top_k)

    # Unpack data
    indices, x_vals, y_vals, diffs = zip(*outliers)

    fig, ax = plt.subplots(figsize=(8, 6))

    ax.scatter(x_vals, y_vals, color='red', label='Outliers')
    for i, idx in enumerate(indices):
        ax.annotate(f"#{idx}", (x_vals[i], y_vals[i]), fontsize=8, textcoords="offset points", xytext=(5, 5))

    # Reference line y = x
    min_val = min(min(x_vals), min(y_vals))
    max_val = max(max(x_vals), max(y_vals))
    ax.plot([min_val, max_val], [min_val, max_val], 'r--', label='y = x')

    ax.set_xlabel(f"Conflicts - {method_x}")
    ax.set_ylabel(f"Conflicts - {method_y}")
    ax.set_title(f"Top {top_k} Conflict Outliers ({method_x} vs {method_y})")
    # ax.set_xscale("log")
    # ax.set_yscale("log")
    ax.grid(True, which="both", ls="--")
    ax.legend()

    if save:
        filename = f"conflict_outliers_{method_x.lower().replace(' ', '_')}_vs_{method_y.lower().replace(' ', '_')}.png"
        save_dir = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\plots\figures"
        os.makedirs(save_dir, exist_ok=True)
        plt.savefig(os.path.join(save_dir, filename))

    plt.show()


outliers = get_conflict_outliers("Extended Regin", "Regin", top_k=20)
extended = []
plain = []
print("Extreme points:")
for idx, x_val, y_val, diff in outliers:
    if x_val < y_val:
        extended.append(idx)
    else:
        plain.append(idx)
    print(f"Instance {idx}: Extended Regin = {x_val}, Regin = {y_val}, |diff| = {diff}")

print(f"Extended Regin better for instances: {extended}, Plain Regin better for instances: {plain}")
# Source root where the original instance folders are
source_root = root_folder

# Destination folders
extended_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\extremes_vaccine\extended_better"
plain_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\extremes_vaccine\plain_better"

# Ensure destination folders exist
os.makedirs(extended_folder, exist_ok=True)
os.makedirs(plain_folder, exist_ok=True)

def copy_instance_folder(instance_idx, destination_folder):
    src = os.path.join(source_root, f"instance_{instance_idx}")
    dst = os.path.join(destination_folder, f"instance_{instance_idx}")
    if os.path.exists(dst):
        shutil.rmtree(dst)  # Remove if already exists to avoid duplication or merge issues
    shutil.copytree(src, dst)

# Copy folders where extended regin is better
for idx in extended:
    copy_instance_folder(idx, extended_folder)

# Copy folders where plain regin is better
for idx in plain:
    copy_instance_folder(idx, plain_folder)

print("Finished copying extreme instance folders.")


plot_conflict_outliers("Extended Regin", "Regin", top_k=20, save=False)