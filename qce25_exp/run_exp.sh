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

    local exp_dir=sim-results/${exp_name}
    echo "Preparing ${exp_name}..."
    rm -rf ${exp_dir}
    mkdir ${exp_dir}

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

    local toml_path=${exp_dir}/${toml_filename}
    printf "${toml_data}" > ${toml_path}
    echo "Output setting.toml"
    
    echo "Experiment start"
    for dataset_file in "$dataset_dir"/*.json; do
        [ -e "$dataset_file" ] || continue
        dataset_file=$(basename "$dataset_file")

        local output_file="${exp_dir}/result-${dataset_file}"
        ../target/release/qmp_scheduler --config-path ${toml_path} -o ${output_file} -d ${dataset_dir}/${dataset_file}
    done

    python3 analyze_result.py ${exp_dir}
}

run_single_class_exp() {
    local class_name=$1
    local dinterval=20000
    local time_limit=2

    run_single_exp 31 20 20 5 "lp" false ${dinterval} "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "${class_name}-LP-D=0"
    run_single_exp 31 20 20 5 "lp" true ${dinterval} "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "${class_name}-LP-D=1"
    run_single_exp 31 20 20 5 "cornergreedy" false ${dinterval} "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "${class_name}-CG-D=0"
    run_single_exp 31 20 20 5 "cornergreedy" true ${dinterval} "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "${class_name}-CG-D=1"
}

run_responsive_exp() {
    local class_name=$1
    local batch_size=$2
    local time_limit=30

    run_single_exp 31 20 20 ${batch_size} "lp" false 20000 "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "resp-${class_name}-LP-D=0_B=${batch_size}"
    #run_single_exp 31 20 20 ${batch_size} "lp" true 20000 "convert-to-cuboid" ${time_limit} "dataset/${class_name}" "resp-${class_name}-LP-D=1_B=${batch_size}"
    #run_single_exp 31 20 20 ${batch_size} "cornergreedy" false 20000 "convert-to-cuboid" 5 "dataset/${class_name}" "resp-${class_name}-CG-D=0_B=${batch_size}"
    #run_single_exp 31 20 20 ${batch_size} "cornergreedy" true 20000 "convert-to-cuboid" 5 "dataset/${class_name}" "resp-${class_name}-CG-D=1_B=${batch_size}"
}

calc_defrag_improvement() {
    class=$1
    python3 calc_defrag_effect.py sim-results/$1-LP-D=0 sim-results/$1-LP-D=1 sim-results/defrag_results/$1-LP.json
    python3 calc_defrag_effect.py sim-results/$1-CG-D=0 sim-results/$1-CG-D=1 sim-results/defrag_results/$1-CG.json
}


cd ..
cargo build --release --features with-cplex

cd qce25_exp

if [ ! -d sim-results ]; then
    mkdir sim-results
fi

# =========================================================
# throughput test
# =========================================================

run_single_class_exp "A"
#run_single_class_exp "B"
#run_single_class_exp "C"
#run_single_class_exp "D"
#run_single_class_exp "E"
#run_single_class_exp "F"
#run_single_class_exp "G"
#run_single_class_exp "H"
#run_single_class_exp "I"


# =========================================================
# Analyze improvements by defragmentation
# =========================================================

rm -rf sim-results/defrag_results
mkdir sim-results/defrag_results

calc_defrag_improvement "A"
#calc_defrag_improvement "B"
#calc_defrag_improvement "C"
#calc_defrag_improvement "D"
#calc_defrag_improvement "E"
#calc_defrag_improvement "F"
#calc_defrag_improvement "G"
#calc_defrag_improvement "H"
#calc_defrag_improvement "I"


# =========================================================
# responsiveness test
# =========================================================

echo "Start responsiveness tests..."
run_responsive_exp "G" 5
#run_responsive_exp "G" 10
#run_responsive_exp "G" 15
#run_responsive_exp "G" 20
echo "Finish responsiveness tests."


