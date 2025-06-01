import os
import subprocess

root_dir = "./dzn_files_community_and_vaccine"
community_dzn_dir = os.path.join(root_dir, "dzn_files_community")
vaccine_dzn_dir = os.path.join(root_dir, "dzn_files_vaccine")
output_dir = "./fzn_files_community_and_vaccine"
community_output_dir = os.path.join(output_dir, "community")
vaccine_output_dir = os.path.join(output_dir, "vaccine")

os.makedirs(community_output_dir, exist_ok=True)
os.makedirs(vaccine_output_dir, exist_ok=True)

minizinc_command = "minizinc -s -v -c --solver minizinc/pumpkin.msc"

for instance_number in range(1, 10):
    dzn_file = os.path.join(community_dzn_dir, f"instance_{instance_number}.dzn")
    if os.path.exists(dzn_file):
        output_fzn_file = os.path.join(community_output_dir, f"instance_{instance_number}.fzn")
        command = f"{minizinc_command} {root_dir}/community-detection.mzn {dzn_file} --output-fzn-to-file {output_fzn_file}"
        subprocess.run(command, shell=True, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


for instance_number in range(1, 10):
    dzn_file = os.path.join(vaccine_dzn_dir, f"vaccine_instance_{instance_number}.dzn")
    if os.path.exists(dzn_file):
        output_fzn_file = os.path.join(vaccine_output_dir, f"instance_{instance_number}.fzn")
        command = f"{minizinc_command} {root_dir}/vaccine.mzn {dzn_file} --output-fzn-to-file {output_fzn_file}"
        subprocess.run(command, shell=True, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

print("FZN file generation completed.")