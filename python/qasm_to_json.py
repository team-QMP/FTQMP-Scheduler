import math
import json
import networkx as nx
from networkx import grid_graph
from qiskit.circuit import QuantumCircuit

def qubit_assignment(num_qubit, qubit_index):
    n = math.ceil(math.sqrt(num_qubit))
    m = math.ceil(math.sqrt(num_qubit))
    return (2 * (qubit_index % n), 2 * (qubit_index // n))

def add_idling_op(polycube, time, num_qubits, dead_qubits, occupied_coordinates):
    for i in range(num_qubits):
        if i not in dead_qubits: # and i not in used_qubits
            if not collision_for_a_coordinate(occupied_coordinates, qubit_assignment(num_qubits, i)):
                coordinate = qubit_assignment(num_qubits, i)
                polycube.append([coordinate[0], coordinate[1], time])
    return polycube

def collision_for_a_coordinate(occupied_coordinates, new_coordinate):
    for coordinate in occupied_coordinates:
        if coordinate == new_coordinate:
            return True
    return False

def collision_for_coordinates(occupied_coordinates, new_coordinates):
    for new_coordinate in new_coordinates:
        if collision_for_a_coordinate(occupied_coordinates, new_coordinate):
            return True
    return False

def qc_to_polycube(qc):
    polycube = [] # list of 3d coordinates
    instructions = []
    for instruction in qc.data:
        instructions.append({
            "operation": instruction.operation.name,
            "qubits": [q.index for q in instruction.qubits],
            "params": instruction.operation.params
        })
    num_qubits = qc.num_qubits
    array_size = math.ceil(math.sqrt(num_qubits)) * math.ceil(math.sqrt(num_qubits))     
    # convert qasm instructions to Lattice Surgery operations
    LS = []
    single_qubit_gate = ["x", "y", "z", "h", "t", "tdg", "s", "sdg", "u1", "u2", "u3", "id", "rx", "ry", "rz", "sx"]
    two_qubit_gate = ["cx", "cz", "swap", "iswap"]
    for instruction in instructions:
        if instruction["operation"] in single_qubit_gate:
            # single qubit gate
            coordinate = qubit_assignment(num_qubits, instruction["qubits"][0])
            LS.append(["1Q",[coordinate], instruction["qubits"]])
        elif instruction["operation"] in two_qubit_gate:
            # two qubit gate
            # Shortest path on a 2d grid graph 
            G = grid_graph(dim=(array_size, array_size))
            all_qubits_except_start_goal = [i for i in range(num_qubits)]
            all_qubits_except_start_goal.remove(instruction["qubits"][0])
            all_qubits_except_start_goal.remove(instruction["qubits"][1])
            for qubit in all_qubits_except_start_goal:
                G.remove_node(qubit_assignment(num_qubits, qubit))
            start = qubit_assignment(num_qubits, instruction["qubits"][0])
            goal = qubit_assignment(num_qubits, instruction["qubits"][1])
            path = nx.shortest_path(G, source=start, target=goal)
            coordinate = []
            for i in path:
                coordinate.append(i)
            LS.append(["2Q",coordinate, instruction["qubits"]])
        elif instruction["operation"] == "measure":
            coordinate = qubit_assignment(num_qubits, instruction["qubits"][0])
            LS.append(["M",[coordinate], instruction["qubits"]])

    time = 0 # time slice
    occupied_coordinates = [] # occupied coordinates at a time slice
    dead_qubits = [] # qubits that are measured
    # convert LS orders to 3D coordinates
    # if there is a collision, add idling operations and move to the next time slice
    for instruction in LS:
        if instruction[0] == "1Q":
            if collision_for_a_coordinate(occupied_coordinates, instruction[1][0]):
                polycube = add_idling_op(polycube, time, num_qubits, dead_qubits, occupied_coordinates)
                time += 1
                occupied_coordinates = []
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
            else:
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
        elif instruction[0] == "2Q":
            if collision_for_coordinates(occupied_coordinates, instruction[1]):
                polycube = add_idling_op(polycube, time, num_qubits, dead_qubits, occupied_coordinates)
                time += 1
                occupied_coordinates = []
                for coordinate in instruction[1]:
                    occupied_coordinates.append(coordinate)
                    polycube.append([coordinate[0], coordinate[1], time])
            else:
                for coordinate in instruction[1]:
                    occupied_coordinates.append(coordinate)
                    polycube.append([coordinate[0], coordinate[1], time])
        elif instruction[0] == "M":
            dead_qubits.append(instruction[2])
            if collision_for_a_coordinate(occupied_coordinates, instruction[1][0]):
                polycube = add_idling_op(polycube, time, num_qubits, dead_qubits, occupied_coordinates)
                time += 1
                occupied_coordinates = []
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
            else:
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
    return polycube

def qasms_to_json(qasm_files, json_file_name = "output.json", time_interval = 0):
    jobid = 0
    program = {"programs": []}
    polycubes = []
    for qasm_file in qasm_files:
        print("loading qasm_file:", qasm_file)
        qc = QuantumCircuit.from_qasm_file(qasm_file)
        polycube = qc_to_polycube(qc)
        program["programs"].append({"Polycube": {"blocks": polycube}})
    time_interval = 100
    jr = []
    for i in range(len(qasm_files)):
        time = i * time_interval
        id = i 
        jr.append([time, id])
    program["job_requests"] = jr

    f = open(json_file_name, "w")
    json.dump(program, f, ensure_ascii=False, indent = 4)
    f.close()
    print("saved as:", json_file_name)
    return True