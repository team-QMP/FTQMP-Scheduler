"""Utility functions for generating random circuits.
Original version: https://github.com/Qiskit/qiskit/blob/main/qiskit/circuit/random/utils.py
"""

import numpy as np
import json
import random
import math
from floorplan import place_surface_code_qubits, place_surface_code_qubits_with_fixed_width, width_to_height_for_num_qubit, visualize_qubit_layout
from compile_qc_to_ls import qc_to_LS
from compile_qc_to_ls import LS_to_polycube

from qiskit.circuit import ClassicalRegister, QuantumCircuit, CircuitInstruction
from qiskit.circuit import Reset
from qiskit.circuit.library import standard_gates
from qiskit.circuit.exceptions import CircuitError

def random_circuit_modified(
    num_qubits, depth, max_operands=4, measure=False, conditional=False, reset=False, seed=None
):
    """Generate random circuit of arbitrary size and form.

    This function will generate a random circuit by randomly selecting gates
    from the set of standard gates in :mod:`qiskit.extensions`. For example:

    .. plot::
       :include-source:

       from qiskit.circuit.random import random_circuit

       circ = random_circuit(2, 2, measure=True)
       circ.draw(output='mpl')

    Args:
        num_qubits (int): number of quantum wires
        depth (int): layers of operations (i.e. critical path length)
        max_operands (int): maximum qubit operands of each gate (between 1 and 4)
        measure (bool): if True, measure all qubits at the end
        conditional (bool): if True, insert middle measurements and conditionals
        reset (bool): if True, insert middle resets
        seed (int): sets random seed (optional)

    Returns:
        QuantumCircuit: constructed circuit

    Raises:
        CircuitError: when invalid options given
    """
    if num_qubits == 0:
        return QuantumCircuit()
    if max_operands < 1 or max_operands > 4:
        raise CircuitError("max_operands must be between 1 and 4")
    max_operands = max_operands if num_qubits > max_operands else num_qubits

    gates_1q = [
        # (Gate class, number of qubits, number of parameters)
        (standard_gates.IGate, 1, 0),
        #(standard_gates.SXGate, 1, 0),
        (standard_gates.XGate, 1, 0),
        #(standard_gates.RZGate, 1, 1),
        #(standard_gates.RGate, 1, 2),
        (standard_gates.HGate, 1, 0),
        #(standard_gates.PhaseGate, 1, 1),
        #(standard_gates.RXGate, 1, 1),
        #(standard_gates.RYGate, 1, 1),
        (standard_gates.SGate, 1, 0),
        (standard_gates.SdgGate, 1, 0),
        #(standard_gates.SXdgGate, 1, 0),
        (standard_gates.TGate, 1, 0),
        (standard_gates.TdgGate, 1, 0),
        #(standard_gates.UGate, 1, 3),
        #(standard_gates.U1Gate, 1, 1),
        #(standard_gates.U2Gate, 1, 2),
        #(standard_gates.U3Gate, 1, 3),
        (standard_gates.YGate, 1, 0),
        (standard_gates.ZGate, 1, 0),
    ]
    if reset:
        gates_1q.append((Reset, 1, 0))
    gates_2q = [
        (standard_gates.CXGate, 2, 0),
        #(standard_gates.DCXGate, 2, 0),
        #(standard_gates.CHGate, 2, 0),
        #(standard_gates.CPhaseGate, 2, 1),
        #(standard_gates.CRXGate, 2, 1),
        #(standard_gates.CRYGate, 2, 1),
        #(standard_gates.CRZGate, 2, 1),
        #(standard_gates.CSXGate, 2, 0),
        #(standard_gates.CUGate, 2, 4),
        #(standard_gates.CU1Gate, 2, 1),
        #(standard_gates.CU3Gate, 2, 3),
        (standard_gates.CYGate, 2, 0),
        (standard_gates.CZGate, 2, 0),
        #(standard_gates.RXXGate, 2, 1),
        #(standard_gates.RYYGate, 2, 1),
        #(standard_gates.RZZGate, 2, 1),
        #(standard_gates.RZXGate, 2, 1),
        #(standard_gates.XXMinusYYGate, 2, 2),
        #(standard_gates.XXPlusYYGate, 2, 2),
        #(standard_gates.ECRGate, 2, 0),
        #(standard_gates.CSGate, 2, 0),
        #(standard_gates.CSdgGate, 2, 0),
        #(standard_gates.SwapGate, 2, 0),
        #(standard_gates.iSwapGate, 2, 0),
    ]
    gates_3q = [
        #(standard_gates.CCXGate, 3, 0),
        #(standard_gates.CSwapGate, 3, 0),
        #(standard_gates.CCZGate, 3, 0),
        #(standard_gates.RCCXGate, 3, 0),
    ]
    gates_4q = [
        #(standard_gates.C3SXGate, 4, 0),
        #(standard_gates.RC3XGate, 4, 0),
    ]

    gates = gates_1q.copy()
    if max_operands >= 2:
        gates.extend(gates_2q)
    if max_operands >= 3:
        gates.extend(gates_3q)
    if max_operands >= 4:
        gates.extend(gates_4q)
    gates = np.array(
        gates, dtype=[("class", object), ("num_qubits", np.int64), ("num_params", np.int64)]
    )
    gates_1q = np.array(gates_1q, dtype=gates.dtype)

    qc = QuantumCircuit(num_qubits)

    if measure or conditional:
        cr = ClassicalRegister(num_qubits, "c")
        qc.add_register(cr)

    if seed is None:
        seed = np.random.randint(0, np.iinfo(np.int32).max)
    rng = np.random.default_rng(seed)

    qubits = np.array(qc.qubits, dtype=object, copy=True)

    # Apply arbitrary random operations in layers across all qubits.
    for layer_number in range(depth):
        # We generate all the randomness for the layer in one go, to avoid many separate calls to
        # the randomisation routines, which can be fairly slow.

        # This reliably draws too much randomness, but it's less expensive than looping over more
        # calls to the rng. After, trim it down by finding the point when we've used all the qubits.
        gate_specs = rng.choice(gates, size=len(qubits))
        cumulative_qubits = np.cumsum(gate_specs["num_qubits"], dtype=np.int64)
        # Efficiently find the point in the list where the total gates would use as many as
        # possible of, but not more than, the number of qubits in the layer.  If there's slack, fill
        # it with 1q gates.
        max_index = np.searchsorted(cumulative_qubits, num_qubits, side="right")
        gate_specs = gate_specs[:max_index]
        slack = num_qubits - cumulative_qubits[max_index - 1]
        if slack:
            gate_specs = np.hstack((gate_specs, rng.choice(gates_1q, size=slack)))

        # For efficiency in the Python loop, this uses Numpy vectorisation to pre-calculate the
        # indices into the lists of qubits and parameters for every gate, and then suitably
        # randomises those lists.
        q_indices = np.empty(len(gate_specs) + 1, dtype=np.int64)
        p_indices = np.empty(len(gate_specs) + 1, dtype=np.int64)
        q_indices[0] = p_indices[0] = 0
        np.cumsum(gate_specs["num_qubits"], out=q_indices[1:])
        np.cumsum(gate_specs["num_params"], out=p_indices[1:])
        parameters = rng.uniform(0, 2 * np.pi, size=p_indices[-1])
        rng.shuffle(qubits)

        # We've now generated everything we're going to need.  Now just to add everything.  The
        # conditional check is outside the two loops to make the more common case of no conditionals
        # faster, since in Python we don't have a compiler to do this for us.
        if conditional and layer_number != 0:
            is_conditional = rng.random(size=len(gate_specs)) < 0.1
            condition_values = rng.integers(
                0, 1 << min(num_qubits, 63), size=np.count_nonzero(is_conditional)
            )
            c_ptr = 0
            for gate, q_start, q_end, p_start, p_end, is_cond in zip(
                gate_specs["class"],
                q_indices[:-1],
                q_indices[1:],
                p_indices[:-1],
                p_indices[1:],
                is_conditional,
            ):
                operation = gate(*parameters[p_start:p_end])
                if is_cond:
                    qc.measure(qc.qubits, cr)
                    # The condition values are required to be bigints, not Numpy's fixed-width type.
                    operation.condition = (cr, int(condition_values[c_ptr]))
                    c_ptr += 1
                qc._append(CircuitInstruction(operation=operation, qubits=qubits[q_start:q_end]))
        else:
            for gate, q_start, q_end, p_start, p_end in zip(
                gate_specs["class"], q_indices[:-1], q_indices[1:], p_indices[:-1], p_indices[1:]
            ):
                operation = gate(*parameters[p_start:p_end])
                qc._append(CircuitInstruction(operation=operation, qubits=qubits[q_start:q_end]))

    if measure:
        qc.measure(qc.qubits, cr)

    return qc

def random_circuit_variable_T_ratio(
    num_qubits, depth, T_gate_ratio, max_operands=4, measure=False, conditional=False, reset=False, seed=None
):
    """Generate random circuit of arbitrary size and form.

    This function will generate a random circuit by randomly selecting gates
    from the set of standard gates in :mod:`qiskit.extensions`. For example:

    .. plot::
       :include-source:

       from qiskit.circuit.random import random_circuit

       circ = random_circuit(2, 2, measure=True)
       circ.draw(output='mpl')

    Args:
        num_qubits (int): number of quantum wires
        depth (int): layers of operations (i.e. critical path length)
        T_gate_ratio(float): ratio of T gates to all gates
        max_operands (int): maximum qubit operands of each gate (between 1 and 4)
        measure (bool): if True, measure all qubits at the end
        conditional (bool): if True, insert middle measurements and conditionals
        reset (bool): if True, insert middle resets
        seed (int): sets random seed (optional)

    Returns:
        QuantumCircuit: constructed circuit

    Raises:
        CircuitError: when invalid options given
    """
    if num_qubits == 0:
        return QuantumCircuit()
    if max_operands < 1 or max_operands > 4:
        raise CircuitError("max_operands must be between 1 and 4")
    max_operands = max_operands if num_qubits > max_operands else num_qubits

    gates_1q = [
        # (Gate class, number of qubits, number of parameters)
        #(standard_gates.IGate, 1, 0),
        #(standard_gates.SXGate, 1, 0),
        #(standard_gates.XGate, 1, 0),
        #(standard_gates.RZGate, 1, 1),
        #(standard_gates.RGate, 1, 2),
        (standard_gates.HGate, 1, 0),
        #(standard_gates.PhaseGate, 1, 1),
        #(standard_gates.RXGate, 1, 1),
        #(standard_gates.RYGate, 1, 1),
        (standard_gates.SGate, 1, 0),
        #(standard_gates.SdgGate, 1, 0),
        #(standard_gates.SXdgGate, 1, 0),
        #(standard_gates.TGate, 1, 0),
        #(standard_gates.TdgGate, 1, 0),
        #(standard_gates.UGate, 1, 3),
        #(standard_gates.U1Gate, 1, 1),
        #(standard_gates.U2Gate, 1, 2),
        #(standard_gates.U3Gate, 1, 3),
        #(standard_gates.YGate, 1, 0),
        #(standard_gates.ZGate, 1, 0),
    ]
    if reset:
        gates_1q.append((Reset, 1, 0))
    gates_2q = [
        (standard_gates.CXGate, 2, 0),
        #(standard_gates.DCXGate, 2, 0),
        #(standard_gates.CHGate, 2, 0),
        #(standard_gates.CPhaseGate, 2, 1),
        #(standard_gates.CRXGate, 2, 1),
        #(standard_gates.CRYGate, 2, 1),
        #(standard_gates.CRZGate, 2, 1),
        #(standard_gates.CSXGate, 2, 0),
        #(standard_gates.CUGate, 2, 4),
        #(standard_gates.CU1Gate, 2, 1),
        #(standard_gates.CU3Gate, 2, 3),
        #(standard_gates.CYGate, 2, 0),
        #(standard_gates.CZGate, 2, 0),
        #(standard_gates.RXXGate, 2, 1),
        #(standard_gates.RYYGate, 2, 1),
        #(standard_gates.RZZGate, 2, 1),
        #(standard_gates.RZXGate, 2, 1),
        #(standard_gates.XXMinusYYGate, 2, 2),
        #(standard_gates.XXPlusYYGate, 2, 2),
        #(standard_gates.ECRGate, 2, 0),
        #(standard_gates.CSGate, 2, 0),
        #(standard_gates.CSdgGate, 2, 0),
        #(standard_gates.SwapGate, 2, 0),
        #(standard_gates.iSwapGate, 2, 0),
    ]
    gates_3q = [
        #(standard_gates.CCXGate, 3, 0),
        #(standard_gates.CSwapGate, 3, 0),
        #(standard_gates.CCZGate, 3, 0),
        #(standard_gates.RCCXGate, 3, 0),
    ]
    gates_4q = [
        #(standard_gates.C3SXGate, 4, 0),
        #(standard_gates.RC3XGate, 4, 0),
    ]

    gates = gates_1q.copy()
    if max_operands >= 2:
        gates.extend(gates_2q)
    if max_operands >= 3:
        gates.extend(gates_3q)
    if max_operands >= 4:
        gates.extend(gates_4q)
    gates = np.array(
        gates, dtype=[("class", object), ("num_qubits", np.int64), ("num_params", np.int64)]
    )
    gates_1q = np.array(gates_1q, dtype=gates.dtype)
    T_gates = [(standard_gates.TGate, 1, 0)]
    T_gates = np.array(
        T_gates, dtype=[("class", object), ("num_qubits", np.int64), ("num_params", np.int64)]
    )

    qc = QuantumCircuit(num_qubits)

    if measure or conditional:
        cr = ClassicalRegister(num_qubits, "c")
        qc.add_register(cr)

    if seed is None:
        seed = np.random.randint(0, np.iinfo(np.int32).max)
    rng = np.random.default_rng(seed)

    qubits = np.array(qc.qubits, dtype=object, copy=True)

    # Apply arbitrary random operations in layers across all qubits.
    for layer_number in range(depth):
        # We generate all the randomness for the layer in one go, to avoid many separate calls to
        # the randomisation routines, which can be fairly slow.

        # This reliably draws too much randomness, but it's less expensive than looping over more
        # calls to the rng. After, trim it down by finding the point when we've used all the qubits.
        gate_specs = rng.choice(gates, size=len(qubits))

        # swap gates with T gates
        num_swaps = int(len(qubits) * T_gate_ratio)
        swap_indices = rng.choice(len(qubits), size=num_swaps, replace=False)
        for i in swap_indices:
            gate_specs[i] = rng.choice(T_gates)

        cumulative_qubits = np.cumsum(gate_specs["num_qubits"], dtype=np.int64)

        # Efficiently find the point in the list where the total gates would use as many as
        # possible of, but not more than, the number of qubits in the layer.  If there's slack, fill
        # it with 1q gates.
        max_index = np.searchsorted(cumulative_qubits, num_qubits, side="right")
        gate_specs = gate_specs[:max_index]
        slack = num_qubits - cumulative_qubits[max_index - 1]
        if slack:
            gate_specs = np.hstack((gate_specs, rng.choice(gates_1q, size=slack)))

        # For efficiency in the Python loop, this uses Numpy vectorisation to pre-calculate the
        # indices into the lists of qubits and parameters for every gate, and then suitably
        # randomises those lists.
        q_indices = np.empty(len(gate_specs) + 1, dtype=np.int64)
        p_indices = np.empty(len(gate_specs) + 1, dtype=np.int64)
        q_indices[0] = p_indices[0] = 0
        np.cumsum(gate_specs["num_qubits"], out=q_indices[1:])
        np.cumsum(gate_specs["num_params"], out=p_indices[1:])
        parameters = rng.uniform(0, 2 * np.pi, size=p_indices[-1])
        rng.shuffle(qubits)

        # We've now generated everything we're going to need.  Now just to add everything.  The
        # conditional check is outside the two loops to make the more common case of no conditionals
        # faster, since in Python we don't have a compiler to do this for us.
        if conditional and layer_number != 0:
            is_conditional = rng.random(size=len(gate_specs)) < 0.1
            condition_values = rng.integers(
                0, 1 << min(num_qubits, 63), size=np.count_nonzero(is_conditional)
            )
            c_ptr = 0
            for gate, q_start, q_end, p_start, p_end, is_cond in zip(
                gate_specs["class"],
                q_indices[:-1],
                q_indices[1:],
                p_indices[:-1],
                p_indices[1:],
                is_conditional,
            ):
                operation = gate(*parameters[p_start:p_end])
                if is_cond:
                    qc.measure(qc.qubits, cr)
                    # The condition values are required to be bigints, not Numpy's fixed-width type.
                    operation.condition = (cr, int(condition_values[c_ptr]))
                    c_ptr += 1
                qc._append(CircuitInstruction(operation=operation, qubits=qubits[q_start:q_end]))
        else:
            for gate, q_start, q_end, p_start, p_end in zip(
                gate_specs["class"], q_indices[:-1], q_indices[1:], p_indices[:-1], p_indices[1:]
            ):
                operation = gate(*parameters[p_start:p_end])
                qc._append(CircuitInstruction(operation=operation, qubits=qubits[q_start:q_end]))

    if measure:
        qc.measure(qc.qubits, cr)

    return qc

def random_circuit_to_json(unit_time = 50, num_qc = 10, json_file_name = "random_program.json"):
    """
    Generate random quantum circuits and save them as a json file for the polycube program.
    Args:
        unit_time (int): unit time for interval
        num_qc (int): number of quantum circuits
        json_file_name (str): name of the json file
    """

    # initial time
    t = 0
    jobid = 0
    program = {"programs": []}

    jr = []

    for i in range(num_qc):
        t += random.randint(1, 4) * unit_time

        random_num_qubits = random.randint(1, 100)
        random_layers = random.randint(10, 200)
        random_T_ratio = random.uniform(0, 1)
        qc = random_circuit_variable_T_ratio(num_qubits=random_num_qubits, depth=random_layers, seed=None, T_gate_ratio=random_T_ratio)
        qc.draw(output = 'mpl')    
        t
        # floorplan =  place_surface_code_qubits_without_size_const(num_data_qubits = qc.num_qubits,  frame = ["bottom","right"], pattern = "block25")
        width = math.floor(np.sqrt(qc.num_qubits)) * 4 + 1
        height = math.floor(np.sqrt(qc.num_qubits)) * 4 + 1
        floorplan = place_surface_code_qubits(width=width,height=height, num_data_qubits = qc.num_qubits, frame = ["bottom","right"], pattern = "block25")
        # visualize_qubit_layout(floorplan, show_data_indices=True)
        LS =  qc_to_LS(qc, floorplan)
        polycube = LS_to_polycube(LS, floorplan)
        program["programs"].append({"Polycube": {"blocks": polycube}})
        jr.append([t, jobid])
        jobid += 1
        
    program["job_requests"] = jr

    f = open(json_file_name, "w")
    json.dump(program, f, ensure_ascii=False, indent = 4)
    f.close()
    print("saved as:", json_file_name)
    return True

def gen_random_polycube_with_block25(num_qubits_x, num_qubits_y, num_layer):
    width = 2 * num_qubits_x + 1
    height = 2 * num_qubits_y + 1
    num_qubits = num_qubits_x * num_qubits_y
    t_gate_ratio = random.uniform(0, 1)
    qc = random_circuit_variable_T_ratio(num_qubits=num_qubits, depth=num_layer, seed=None, T_gate_ratio=t_gate_ratio)
    floorplan = place_surface_code_qubits_with_fixed_width(width=width, frame=["bottom", "right"], num_data_qubits=qc.num_qubits, pattern="block25")
    #visualize_qubit_layout(floorplan, show_data_indices=True)
    LS = qc_to_LS(qc, floorplan)
    return LS_to_polycube(LS, floorplan)

def gen_random_ls_polycubes(qwidth_range, qheight_range, layer_range, num_programs):
    polycubes = []
    for _ in range(num_programs):
        w = random.randint(qwidth_range[0], qwidth_range[1])
        h = random.randint(qheight_range[0], qheight_range[1])
        l = random.randint(layer_range[0], layer_range[1])
        polycubes.append(gen_random_polycube_with_block25(w, h, l))

    return polycubes

def random_circuit_to_json_with_fixed_width(width, unit_time = 50, num_qc = 100, num_qubits = [5,100], layers = [10,200],json_file_name = "random_program_with_fixed_witdth.json"):
    """
    Generate random quantum circuits and save them as a json file for the polycube program.
    Args:
        unit_time (int): unit time for interval
        num_qc (int): number of quantum circuits
        json_file_name (str): name of the json file
    """

    # initial time
    t = 0
    jobid = 0
    program = {"programs": []}

    jr = []

    for i in range(num_qc):
        t += random.randint(1, 4) * unit_time  # interval

        random_num_qubits = random.randint(num_qubits[0], num_qubits[1])
        random_layers = random.randint(layers[0], layers[1])
        random_T_ratio = random.uniform(0, 1)
        qc = random_circuit_variable_T_ratio(num_qubits=random_num_qubits, depth=random_layers, seed=None, T_gate_ratio=random_T_ratio)
        floorplan = place_surface_code_qubits_with_fixed_width(width=width, frame = ["bottom","right"], num_data_qubits=qc.num_qubits, pattern = "block25")
        # visualize_qubit_layout(floorplan, show_data_indices=True)
        LS =  qc_to_LS(qc, floorplan)
        polycube = LS_to_polycube(LS, floorplan)
        program["programs"].append({"Polycube": {"blocks": polycube}})
        jr.append([t, jobid])
        jobid += 1
        
    program["job_requests"] = jr

    f = open(json_file_name, "w")
    json.dump(program, f, ensure_ascii=False, indent = 4)
    f.close()
    print("saved as:", json_file_name)
    return True



### Example with variable T ratio
# num_qubits = 10
# num_layer = 3
# T_ratio = 0.1
# seed = 1

# qc = random_circuit_variable_T_ratio(num_qubits=num_qubits, depth=num_layer, seed=seed, T_gate_ratio=T_ratio)
# qc.draw()
