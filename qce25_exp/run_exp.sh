#!/bin/bash

run_single_exp() {
    local IFS=',' # separated by ','

    local micro_per_cycle=$1
    local sx=$2
    local sy=$3
    local batch_size=$4
    local scheduler=$5
    local enable_defrag=$6
    local defrag_interval=$7
    local pp=($8) # array (give like "A,B,C") 
    local time_limit=$9
    local dataset_dir=${10}
    local exp_name=${11}

    if $enable_defrag; then
        local enable_defrag="true"
    else
        local enable_defrag="false"
    fi

    echo "Preparing ${exp_name}..."
    rm -rf ${exp_name}
    mkdir ${exp_name}

    local toml_filename="setting.toml"
    local toml_data="
size_x = ${sx}
size_y = ${sy}
micro_sec_per_cycle = ${micro_per_cycle}
no_output_program = true

enable_defrag = ${enable_defrag}
defrag_interval = ${defrag_interval}

[preprocessor]
processes = [\"${pp[*]}\"]

[scheduler]
kind = \"${scheduler}\"
time_limit = ${time_limit}
batch_size = ${batch_size}"

    printf "${toml_data}" > ${exp_name}/${toml_filename}
    echo "Output setting.toml"
    
    echo "Experiment start"
    for dataset_file in "$dataset_dir"/*.json; do
        [ -e "$dataset_file" ] || continue
        dataset_file=$(basename "$dataset_file")

        local output_file="${exp_name}/result-${dataset_file}"
        ../target/release/qmp_scheduler --config-path ${exp_name}/${toml_filename} -o ${output_file} -d ${dataset_dir}/${dataset_file}
    done

    python3 analyze_result.py ${exp_name}
}


cd ..
cargo build --release --features with-cplex

cd qce25_exp

# =========================================================
# responsiveness test
# =========================================================

# TODO...

# =========================================================
# throughput test
# =========================================================

run_single_exp 100 21 21 5 "cornergreedy" false 10000 "convert-to-cuboid" 120 "dataset/A" "A-CG-D=0"
run_single_exp 100 21 21 5 "cornergreedy" true 10000 "convert-to-cuboid" 120 "dataset/A" "A-CG-D=1"

