# Online Scheduler for Quantum Multiprogramming
This repository contains:

0. 【Python】Quantum circuit generation in Qiskit and compilation process to polycube. It has a visualizer for a job / allocated multiple job. You can also export jobs as a JSON file or an OpenSCAD files. There is a jupyter notebook [tutorial](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_examples/circuit_generation_and_compilation.ipynb).
1. 【Rust】Preprocessor: Approximate polycube with a bounding boxes.
2. 【Rust】Scheduler: Allocate the job request to a space in the quantum processor
3. 【Rust】Quantum Processor Simulator
4. 【Rust】Defragmentation: Relocate job during execution to make a space for next job.


![flow](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/figs/QMP_flow.jpg)

## 【Python】 Requirements
Required packages for python are [HERE](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_examples/requirements.txt). You can install all with following command.
```
pip install -r requirements.txt
```

<!-- ## Installation and usage -->

<!-- ## Examples -->

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