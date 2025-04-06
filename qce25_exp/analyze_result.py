import os
import json
import glob
import sys
import math

def analyze_exp_result(input_folder):
    json_files = glob.glob(os.path.join(input_folder, "result-*.json"))
    
    result = {}

    n = len(json_files)
    throughputs = []
    sum_th = 0
    sum_sq_th = 0
    for json_file in json_files:
        with open(json_file, "r", encoding="utf-8") as f:
            data = json.load(f)
        
        th = data.get("total_cycle", [])
        throughputs.append(th)
        sum_th += th 
        sum_sq_th += th * th

    th_mean = sum_th // n
    th_stddev = math.sqrt(sum_sq_th / n - th_mean * th_mean)

    result["throughput_mean"] = th_mean
    result["throughput_stddev"] = th_stddev
    result["throughput_min"] = min(throughputs)
    result["throughput_max"] = max(throughputs)

    output_json_file = os.path.join(input_folder, "all-result.txt")

    with open(output_json_file, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent = 4)


if __name__ == "__main__":
    args = sys.argv[1:]
    dirname = args[0]
    analyze_exp_result(dirname)
