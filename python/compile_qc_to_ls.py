from floorplan import build_graph_from_qubit_floorplan
import networkx as nx


def qc_to_LS(qc, floorplan):
    # return lattice for the given quantum circuit and floorplan
    
    # convert qiksit circuit to a list of instructions
    instructions = []
    for instruction in qc.data:
        instructions.append({
            "operation": instruction.operation.name,
            "qubits": [q.index for q in instruction.qubits],
            "params": instruction.operation.params
        })
    num_qubits = qc.num_qubits

    # convert qasm instructions to Lattice Surgery operations
    LS = [] # Lattice surgery operations

    qubit_dict = make_dict_of_qubits(floorplan)
    data_qubits = qubit_dict[0]
    ancilla_qubits = qubit_dict[1]

    # list of supported quasntum gates
    single_qubit_gate = ["x", "y", "z", "h", "t", "tdg", "s", "sdg", "u1", "u2", "u3", "id", "rx", "ry", "rz", "sx"]
    two_qubit_gate = ["cx", "cz", "swap", "iswap"]
    
    for instruction in instructions:
        if instruction["operation"] in single_qubit_gate:
            # single qubit gate
            coordinate = data_qubits[instruction["qubits"][0]]
            LS.append(["1Q",[coordinate], instruction["qubits"]])
        elif instruction["operation"] in two_qubit_gate:
            # two qubit gate
            # Shortest path on a 2d grid graph 
            G = build_graph_from_qubit_floorplan(floorplan)

            # all_qubits_except_start_goal = [i for i in range(num_qubits)]
            # all_qubits_except_start_goal.remove(data_qubits[instruction["qubits"][0]])
            # all_qubits_except_start_goal.remove(data_qubits[instruction["qubits"][1]])
            # for qubit in all_qubits_except_start_goal:
            #     G.remove_node(qubit_assignment(num_qubits, qubit))

            start = floorplan['data_qubits'][instruction["qubits"][0]] # position of start qubit
            goal = floorplan['data_qubits'][instruction["qubits"][1]]  # position of goal qubit

            path = nx.shortest_path(G, source=start, target=goal)
            coordinate = []
            for i in path:
                coordinate.append(i)
            LS.append(["2Q",coordinate, instruction["qubits"]])
        elif instruction["operation"] == "measure":
            coordinate = floorplan['data_qubits'][instruction["qubits"][0]]
            LS.append(["M",[coordinate], instruction["qubits"]])

    return LS

def make_dict_of_qubits(floorplan):
    """
    データ量子ビット、フレーム量子ビット、アンシラ量子ビットからなるfloorplanを受け取り、データ量子ビットとアンシラ量子ビットのdict{インデックス: 座標}を返す。
    この時フレーム量子ビットはアンシラ量子ビットのdictに含める
    """
    data_qubits = {}
    ancilla_qubits = {}

    i = 0
    for qubit in floorplan['data_qubits']:
        data_qubits[i] = qubit
        i+=1

    i = 0
    for qubit in floorplan['frame_qubits']:
        ancilla_qubits[i] = qubit
        i+=1
    for qubit in floorplan['ancilla_qubits']:
        ancilla_qubits[i] = qubit
        i+=1
    return data_qubits, ancilla_qubits
    

def collision(occupied_coordinates, dead_coordinates, operation):
    # check if there is a collision in the Lattice Surgery operations
    # all_qubit_coordinates: list of all qubit coordinates
    # occupied_coordinates: list of occupied coordinates
    # dead_coordinates: list of dead coordinates(measured qubits)
    # operation: Lattice Surgery operation (path, list of 2D coordinates)
    collision = False

    for qubit in operation:
        if qubit in occupied_coordinates:
            collision = True
            break

    for qubit in operation:
        if qubit in dead_coordinates:
            collision = True
            break

    return collision

def add_idling_op(polycube, time, dead_coordinates, occupied_coordinates,  data_qubits):
    # add identity operations for all data qubits exept occupied_coordinates, dead_coordinates
    data_key =  data_qubits.keys()
    # print('dead_coordinates', dead_coordinates)
    # print('occupied_coordinates', occupied_coordinates)
    # print('data_qubits', data_qubits)
    for i in data_key:
        apply_identity = True
        # !deadかつ !occupiedならidentityをつける
        if data_qubits[i] in occupied_coordinates:
            apply_identity = False
        if data_qubits[i] in dead_coordinates:
            apply_identity = False

        if apply_identity:
            polycube.append([data_qubits[i][0], data_qubits[i][1], time])
            # print('add' + str([data_qubits[i][0], data_qubits[i][1], time]))
    return polycube

def LS_to_polycube(LS, floorplan):
    # return polycube (list of 3D coordinates) for the given Lattice Surgery operations and floorplan
    polycube = [] # list of 3d coordinates
    time = 0 # time slice
    occupied_coordinates = [] # occupied coordinates at a time slice
    dead_coordinates = [] # qubits that are measured

    data_qubits, ancilla_qubits = make_dict_of_qubits(floorplan)

    # convert LS operations to 3D coordinates
    # if there is a collision, add idling operations and move to the next time slice, then apply the operation
    for instruction in LS:
        if instruction[0] == "1Q":
            if collision(occupied_coordinates, dead_coordinates, [instruction[1][0]]):
                polycube = add_idling_op(polycube, time, dead_coordinates, occupied_coordinates,  data_qubits)
                time += 1
                occupied_coordinates = []
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
            else:
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
        elif instruction[0] == "2Q":
            if collision(occupied_coordinates, dead_coordinates, instruction[1]):
                polycube = add_idling_op(polycube, time, dead_coordinates, occupied_coordinates,  data_qubits)
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
            dead_coordinates.append(instruction[2])
            if collision(occupied_coordinates, dead_coordinates, instruction[1][0]):
                polycube = add_idling_op(polycube, time, dead_coordinates, occupied_coordinates, data_qubits)
                time += 1
                occupied_coordinates = []
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])
            else:
                occupied_coordinates.append(instruction[1][0])
                polycube.append([instruction[1][0][0], instruction[1][0][1], time])

    return polycube