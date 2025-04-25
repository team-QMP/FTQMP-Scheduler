# Online Scheduler for Quantum Multiprogramming
This repository contains:
- 【Python】 Toy compiler that compiles physical quantum circuit (QuantumCircuit in Qiskit and OpenQASM at the moment) into a logical program (using lattice surgery on surface codes with polycube representation) [Tutorial](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/examples/generate_dataset.ipynb)
- 【Python】Visualizer for polycubes that export SolidPython objects or OpenSCAD files [Tutorial](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_scad/solid_python_tutorial.ipynb)
- 【Rust】Preprocessors, an online scheduler, and a simulator 

![flow](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/figs/flow.pdf)

## Requirements
Required packages for python are [HERE](https://github.com/team-QMP/FTQMP-Scheduler/blob/main/python_examples/requirements.txt). You can install all with following command.
```
pip install -r requirements.txt
```

<!-- ## Installation and usage -->

<!-- ## Examples -->

## Citation

ArXiv submission is available at [HERE](). If you use this repository in your research or work, please cite it using the following BibTeX entry:

```
@misc{your_citation_key,
  author       = {Your Name},
  title        = {Title of the Repository},
  year         = {2025},
  publisher    = {GitHub},
  journal      = {GitHub repository},
  howpublished = {\url{https://github.com/your_username/your_repo_name}},
}```