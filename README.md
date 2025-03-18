# Online Scheduler for Quantum Multiprogramming
This repository contains:
- 【Python】 Toy compiler that compiles physical quantum circuit (QuantumCircuit in Qiskit and OpenQASM at the moment) into a logical program (using lattice surgery on surface codes with polycube representation)
- 【Python】Visualizer for polycubes that export SolidPython objects or OpenSCAD files
- 【Rust】Preprocessors, an online scheduler, and a simulator 

# Requirements for Python
- Visualization
  - solidpython                   1.1.3
  - viewscad                      0.2.0
  - matplotlib                    3.7.2
- compilation
  - networkx                      3.1
  - numpy                         1.24.3
- Quantum Circuits
  - qiskit                        0.46