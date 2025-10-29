from floorplan import build_graph_from_qubit_floorplan
import networkx as nx
from solid import union, translate, cube, color, scad_render

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
    input: floorplan
    output: dict{index: coordinate} of data_qubits, frame_qubits and ancilla_qubits
    Frame qubits are included in the dict of ancilla_qubits
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

def _neighbors4(coord):
    x, y = coord
    return [(x+1,y), (x-1,y), (x,y+1), (x,y-1)]

def _ancilla_coord_set(ancilla_qubits):
    # ancilla_qubits: dict[idx] -> (x,y)
    return set(ancilla_qubits.values())

def _pick_adjacent_ancilla(data_xy, ancilla_qubits, occupied_now, dead_set):
    """
    データタイル data_xy の4近傍から使える補助タイルを1つ選ぶ。
    近傍に無ければ最短距離の ancilla を1つ選ぶ。
    """
    anc_set = _ancilla_coord_set(ancilla_qubits)
    # まず4近傍
    for nb in _neighbors4(data_xy):
        if nb in anc_set and (nb not in occupied_now) and (nb not in dead_set):
            return nb
    # 無ければ最近傍
    # 距離順 + 未使用優先
    candidates = [a for a in anc_set if (a not in occupied_now) and (a not in dead_set)]
    if not candidates:
        # 最悪、データタイル自身を返して衝突を誘発（後でアイドル挿入される）
        return data_xy
    candidates.sort(key=lambda a: abs(a[0]-data_xy[0]) + abs(a[1]-data_xy[1]))
    return candidates[0]

def collision(occupied_coordinates, dead_coordinates, new_candidates):
    """occupied_coordinates: list[(x,y)]
       dead_coordinates: set[(x,y)]
       new_candidates: list[(x,y)]
    """
    occ_set = set(occupied_coordinates)
    for c in new_candidates:
        if (c in occ_set) or (c in dead_coordinates):
            return True
    return False

def add_idling_op(polycube, time, dead_coordinates, occupied_coordinates, data_qubits):
    """
    このタイムスライスで未占有,未デッドの全データタイルにIを置く
    """
    occ_set = set(occupied_coordinates)
    for _, xy in data_qubits.items():
        if (xy not in occ_set) and (xy not in dead_coordinates):
            polycube.append([xy[0], xy[1], time])
    return polycube


# QASM→LS

def qc_to_LS_T(qc, floorplan, connect_factory=False, magic_interface=None):
    """
    return:
      - LS: list of [opcode, coord_list, qubits, duration]
      - t_2d_coordinates: list of 2D coords where magic-T is injected (ancilla tile used)
    """
    # convert qiksit circuit to a list of instructions
    instructions = []
    for instruction in qc.data:
        instructions.append({
            "operation": instruction.operation.name,
            "qubits": [q.index for q in instruction.qubits],
            "params": instruction.operation.params
        })

    qubit_dict = make_dict_of_qubits(floorplan)
    data_qubits = qubit_dict[0]      # dict[qid] -> (x,y)
    ancilla_qubits = qubit_dict[1]   # dict[aid] -> (x,y)

    # Supported gates
    single_qubit_gate = ["x", "y", "z", "h", "t", "tdg", "s", "sdg", "id"]
    two_qubit_gate = ["cx", "cz", "swap", "iswap"]

    LS = []
    t_2d_coordinates = []

    # graph for surgery pathfinding (constructed only if needed)
    G = None
    if connect_factory:
        if magic_interface is None:
            raise ValueError("connect_factory=True ですが magic_interface が指定されていません。")
        G = build_graph_from_qubit_floorplan(floorplan)

    for inst in instructions:
        op = inst["operation"]
        qs = inst["qubits"]

        # Single qubit gate
        if op in single_qubit_gate:
            q = qs[0]
            data_xy = data_qubits[q]

            if op in ["x", "y", "z", "id"]:
                LS.append(["PG", [data_xy], [q], 1])

            elif op in ["s", "sdg"]:
                anc_xy = _pick_adjacent_ancilla(data_xy, ancilla_qubits, occupied_now=[], dead_set=set())
                LS.append(["SG", [data_xy, anc_xy], [q], 2])

            elif op == "h":
                anc_xy = _pick_adjacent_ancilla(data_xy, ancilla_qubits, occupied_now=[], dead_set=set())
                LS.append(["HG", [data_xy, anc_xy], [q], 3])

            elif op in ["t", "tdg"]:
                # data + ancilla + magic
                anc_xy = _pick_adjacent_ancilla(data_xy, ancilla_qubits, occupied_now=[], dead_set=set())
                magic_xy = anc_xy  # injection point (as before)

                # Supply magic state via factory connection if requested
                if connect_factory:
                    # magic state preparation: shortest path from interface to magic_xy
                    path = nx.shortest_path(G, source=magic_interface, target=magic_xy)
                    # duration is path length
                    LS.append(["MP", path, [], len(path)])

                if op == "t":
                    LS.append(["TG", [data_xy, anc_xy, magic_xy], [q], 4])
                else:
                    LS.append(["TG_DAG", [data_xy, anc_xy, magic_xy], [q], 4])

                t_2d_coordinates.append(magic_xy)

            else:
                LS.append(["Undefined1Q", [data_xy], [q], 1])

        # 2 qubit gate
        elif op in two_qubit_gate:
            G2 = build_graph_from_qubit_floorplan(floorplan)
            start = floorplan['data_qubits'][qs[0]]
            goal  = floorplan['data_qubits'][qs[1]]
            path = nx.shortest_path(G2, source=start, target=goal)

            for i in range(len(path) - 1):
                e = [path[i], path[i+1]]
                LS.append(["SURGERY", e, qs, 3])

        # measurement
        elif op == "measure":
            q = qs[0]
            data_xy = floorplan['data_qubits'][q]
            LS.append(["M", [data_xy], [q], 1])

        else:
            pass

    return LS, t_2d_coordinates

# ---- LS → 3D polycube ----

def LS_T_to_polycube(LS, floorplan):
    polycube = []
    time = 0
    occupied_coordinates = []     # 現スライスで占有している2D座標
    dead_coordinates = set()      # 2D座標で管理（重要！）

    data_qubits, ancilla_qubits = make_dict_of_qubits(floorplan)
    t_3d_coordinates = []

    def finalize_slice():
        nonlocal polycube, time, occupied_coordinates
        # まだ確定していなければ、このスライスで未占有データにIを付与
        add_idling_op(polycube, time, dead_coordinates, occupied_coordinates, data_qubits)
        time += 1
        occupied_coordinates = []

    for op, coords, qubits, duration in LS:
        need = list(coords)

        # 衝突している間はスライス確定（=Iを埋める）して次へ進める
        while collision(occupied_coordinates, dead_coordinates, need):
            finalize_slice()

        # duration 回、同じ占有座標でステップを回す
        for k in range(duration):
            # 命令ボクセルを書き込み
            for (x, y) in need:
                polycube.append([x, y, time])
                # T注入点の3D記録（ここでは need[-1] をマジック注入座標として扱う）
                if op in ["TG", "TG_DAG"] and (x, y) == need[-1]:
                    t_3d_coordinates.append([x, y, time])

            # このステップで占有していた座標を記録
            occupied_coordinates = need.copy()

            # スライスを確定してIを付与し、次の時間へ
            finalize_slice()

        # 測定はデータ座標を dead に追加（qubit index ではない！）
        if op == "M":
            dead_coordinates.add(need[0])

    # 末尾で占有が残っていても finalize 済みなので何もしない
    return polycube, t_3d_coordinates

def create_polycube_colored(polycube, t_3d_coordinates, voxel=1.0):
    """
    polycube: [[x,y,t], ...]
    t_3d_coordinates: [[x,y,t], ...] 
    voxel: ボクセル1つの立方体サイズ
    """
    tset = set(map(tuple, t_3d_coordinates))
    normal_cubes = []
    t_cubes = []

    for x, y, z in polycube:
        c = translate([x * voxel, y * voxel, z * voxel])(cube(voxel, center=True))
        if (x, y, z) in tset:
            t_cubes.append(c)
        else:
            normal_cubes.append(c)

    model = union()(
        color([0.8, 0.8, 0.8])(union()(*normal_cubes)) if normal_cubes else None,
        color([1.0, 0.0, 0.0])(union()(*t_cubes)) if t_cubes else None
    )
    return model