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
Please see [arXiv]() paper for more details. If you use this repository in your research or work, please cite it using the following BibTeX entry:

```
@misc{your_citation_key,
  author       = {Your Name},
  title        = {Title of the Repository},
  year         = {2025},
  publisher    = {GitHub},
  journal      = {GitHub repository},
  howpublished = {\url{https://github.com/your_username/your_repo_name}},
}```