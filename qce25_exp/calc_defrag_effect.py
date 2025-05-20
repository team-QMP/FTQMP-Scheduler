import os
import json
import glob
import sys
import math

def calc_defrag_effect(input_folder1, input_folder2, output_file):
    json_files1 = glob.glob(os.path.join(input_folder1, "result-*.json"))
    
    n = len(json_files1)
    improve_rates = []
    for json_file1 in json_files1:
        with open(json_file1, "r", encoding="utf-8") as f1:
            data1 = json.load(f1)

        basename = os.path.basename(json_file1)
        json_file2 = os.path.join(input_folder2, basename)
        with open(json_file2, "r", encoding="utf-8") as f2:
            data2 = json.load(f2)
        
        th1 = data1.get("total_cycle", [])
        th2 = data2.get("total_cycle", [])
        improve_rates.append((th1 - th2) / th1 * 100)

    ir_sq_sum = sum(map(lambda v: v * v, improve_rates))
    ir_mean = sum(improve_rates) / n
    ir_stddev = math.sqrt(ir_sq_sum / n - ir_mean * ir_mean)

    result = {}
    result["improve_rate_mean"] = ir_mean
    result["improve_rate_min"] = min(improve_rates)
    result["improve_rate_max"] = max(improve_rates)
    result["improve_rate_stddev"] = ir_stddev

    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent = 4)


if __name__ == "__main__":
    args = sys.argv[1:]
    dir1 = args[0]
    dir2 = args[1]
    out = args[2]
    calc_defrag_effect(dir1, dir2, out)
