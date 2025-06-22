import os
import re
import matplotlib.pyplot as plt
import numpy as np

problem = "community"
# problem = "vaccine"
# problem = "nsite"

if problem == "community":
    root_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\draft_runs\runs\community"
elif problem == "vaccine":
    root_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\draft_runs\runs\vaccine"
elif problem == "nsite":
    root_folder = r"C:\Users\jaspe\Documents\TuDelft\Reasearch_Intelligent_Decision_Making\pumpkin-gcc_extended_res\gcc_extended_res\experiments\draft_runs\runs\nsite"
else:
    print("ERROR: Incorrect problem class")

decomp_over_extended_Res = []
extendedRes_over_decomp = []
reg_over_exReg = []
exReg_over_reg = []

def find_difference(out_folder):
    instance = out_folder.split("\\")[-2].split("_")[1]
    decomp_str = open(os.path.join(out_folder, "decomposition_output.txt"), 'r', encoding='utf-8', errors='ignore').read()
    extendedReg_str = open(os.path.join(out_folder, "extended-resolution-with-regin_output.txt"), 'r', encoding='utf-8', errors='ignore').read()
    extendedRes_str = open(os.path.join(out_folder, "extended-resolution_output.txt"), 'r', encoding='utf-8', errors='ignore').read()
    reg_str = open(os.path.join(out_folder, "regin-arc-consistent_output.txt"), 'r', encoding='utf-8', errors='ignore').read()

    if "==========" in decomp_str and "==========" not in extendedRes_str:
        decomp_over_extended_Res.append(instance)
    elif "==========" not in decomp_str and "==========" in extendedRes_str:
        extendedRes_over_decomp.append(instance)
    if "==========" in reg_str and "==========" not in extendedReg_str:
        reg_over_exReg.append(instance)
    elif "==========" not in reg_str and "==========" in extendedReg_str:
        exReg_over_reg.append(instance)
    # return decomp_over_extended_Res, extendedRes_over_decomp, reg_over_exReg, exReg_over_reg


# Regex to match "instance_<number>"
instance_pattern = re.compile(r"instance_(\d+)")
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
                    continue
                find_difference(out_folder)

print(f"Decomposition did not timeout, while extended Res did, for {len(decomp_over_extended_Res)} instances: {decomp_over_extended_Res}")
print(f"Extended Res did not timeout, while decomposition did, for {len(extendedRes_over_decomp)} instances: {extendedRes_over_decomp}")
print(f"Plain Regin did not timeout, while extended Regin did, for {len(reg_over_exReg)} instances: {reg_over_exReg}")
print(f"Extende Regin did not timeout, while plain Regin did, for {len(exReg_over_reg)} instances: {exReg_over_reg}")
