import os
import subprocess
from concurrent.futures import ThreadPoolExecutor


num_cores = os.cpu_count()
print(f"Number of CPU cores: {num_cores}")
timeout = "20000"  # Timeout in milliseconds

methods = ['extended-resolution-with-regin',
           'basic-filter',
           'regin-arc-consistent',
           'extended-resolution']

root_dir = "./fzn_files_community_and_vaccine"
community_fzn_dir = os.path.join(root_dir, "community")
vaccine_fzn_dir = os.path.join(root_dir, "vaccine")
runs_dir = "./runs"

community_runs_dir = os.path.join(runs_dir, "community")
vaccine_runs_dir = os.path.join(runs_dir, "vaccine")
os.makedirs(community_runs_dir, exist_ok=True)
os.makedirs(vaccine_runs_dir, exist_ok=True)

cargo_command = "./target/release/pumpkin-solver -s -v --gcc-propagation-method"

def run_command(method, fzn_path, out_dir, err_dir):
    output_file = os.path.join(out_dir, f"{method}_output.txt")
    error_file = os.path.join(err_dir, f"{method}_error.txt")
    with open(output_file, "w") as outfile, open(error_file, "w") as errfile:
        command = f"{cargo_command} {method} {fzn_path} -t {timeout}"
        print('running', command)
        subprocess.run(command, shell=True, check=True, stdout=outfile, stderr=errfile)

def process_files(fzn_dir, runs_dir):
    tasks = []
    with ThreadPoolExecutor() as executor:
        for fzn_file in os.listdir(fzn_dir):
            if fzn_file.endswith(".fzn"):
                instance_name = os.path.splitext(fzn_file)[0]
                instance_dir = os.path.join(runs_dir, instance_name)
                os.makedirs(instance_dir, exist_ok=True)
                out_dir = os.path.join(instance_dir, "out")
                err_dir = os.path.join(instance_dir, "err")
                os.makedirs(out_dir, exist_ok=True)
                os.makedirs(err_dir, exist_ok=True)

                fzn_path = os.path.join(fzn_dir, fzn_file)
                for method in methods:
                    tasks.append(executor.submit(run_command, method, fzn_path, out_dir, err_dir))

        for task in tasks:
            task.result()

process_files(community_fzn_dir, community_runs_dir)
process_files(vaccine_fzn_dir, vaccine_runs_dir)

print("Execution of FZN files completed. Outputs saved to 'runs' folder.")