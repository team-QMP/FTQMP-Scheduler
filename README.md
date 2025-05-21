# Online Scheduler for Fault-Tolerant Quantum Multiprogramming (FTQMP)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

This repository contains:

0. 【Python】Quantum circuit generation in Qiskit and compilation process to polycube. It has a visualizer for a job / allocated multiple job. You can also export jobs as a JSON file or an OpenSCAD files. There is a jupyter notebook [tutorial](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_examples/circuit_generation_and_compilation.ipynb).
1. 【Rust】Preprocessor: Approximate polycube with a bounding boxes.
2. 【Rust】Scheduler: Allocate the job request to a space in the quantum processor
3. 【Rust】Quantum Processor Simulator
4. 【Rust】Defragmentation: Relocate job during execution to make a space for next job.


![flow](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/figs/QMP_flow.jpg)


## How to build

### Docker image

You can build our schedulers by the following commands:

```
$ docker build -t ftqmp:latest .
$ docker run -it --name ftqmp ftqmp:latest
```

This will start bash with the executable `qmp_scheduler` enabled in the container.


## 【Python】 Requirements
Required packages for python are [HERE](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_examples/requirements.txt). You can install all with following command.
```
pip install -r requirements.txt
```

## How to use

The current FTQMP scheduler (`qmp_scheduler`) takes three options as input:

```
qmp_scheduler -d <dataset-file> --config-path <config-file> -o <output-path>
```

- `dataset-file` is a JSON file containing program (polycube or cuboid) and request data,
- `config-file` is a TOML file containing the parameters for the simulation, and
- `output-path` specifies the path where the result JSON file will be output.

Please see `examples/` for details of the structure of dataset JSON files and config TOML files.

### The data format of JSON for datasets

```
{
    "programs": [
        <program1>,
        ...
    ],
    "job_requests": [
        [program_id1, tm],
        ...,
        [program_idm, tm]
    ]
}
```

- A job request `[program_id, t]` means the i-th program will be requested at time `t`.
- `program_id` and `t` must be integer values.

Currently, either the polycube or k-cuboid representation is available as program data.

```
{
    "Polycube": {
        "blocks": [
            [x1, y1, z1],
            ...,
            [xn, yn, zn]
        ]
    }
}
```

- `[xi, yi, zi]` represents the i-th block of the polycube
- `xi`, `yi`, `zi` are must be integer values

```
{
    "Cuboid": [
        {
            "pos": [x1, y1, z1],
            "size_x": a1,
            "size_y": b1,
            "size_z": c1,
        },
        ...
    ]
}
```

- Each cuboid consists of `pos`, `size_x`, `size_y`, `size_z`
    - `pos` is the base point of the cuboid, which has three integer values `xi`, `yi`, `zi`
    - `size_x`, `size_y`, `size_z` are the sizes of the cuboid
    - that is, each cuboid is `[xi, xi + size_xi) * [yi, yi + size_yi) * [zi, zi + size_zi)`

We have to note that some features are not supported for the cuboid representation with k >= 2.


## Citation
Please see [arXiv](https://arxiv.org/abs/2505.06741) paper for more details. If you use this repository in your research or work, please cite it using the following BibTeX entry:

```
@misc{nishio2025onlinejobschedulerfaulttolerant,
      title={Online Job Scheduler for Fault-tolerant Quantum Multiprogramming}, 
      author={Shin Nishio and Ryo Wakizaka and Daisuke Sakuma and Yosuke Ueno and Yasunari Suzuki},
      year={2025},
      eprint={2505.06741},
      archivePrefix={arXiv},
      primaryClass={quant-ph},
      url={https://arxiv.org/abs/2505.06741}, 
}```
