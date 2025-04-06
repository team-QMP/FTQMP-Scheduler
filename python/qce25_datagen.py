from floorplan import *
from compile_qc_to_ls import *
from random_qc import *
from qiskit.circuit import QuantumCircuit
import numpy as np


def gen_polycube_dataset():
    num_ref_qc = 100
    num_req_per_qc = 5
    num_request = num_ref_qc * num_req_per_qc
    interval = 100

    job_data = {"programs": []}

    for i in range(num_ref_qc):
        print("{}-th program generated.".format(i))
        polycube = gen_random_ls_polycubes(qwidth_range=[3, 10], qheight_range=[3, 10], layer_range=[100, 1000], num_programs=1)[0];
        job_data["programs"].append({"Polycube": {"blocks": polycube}})

    request_arr = np.zeros(num_request, dtype=int)
    for id in range(num_ref_qc):
        l = id * num_req_per_qc
        r = (id + 1) * num_req_per_qc
        request_arr[l:r] = id
    np.random.shuffle(request_arr)
    request_arr = request_arr.tolist()

    requests = []
    requests_t0 = []
    for job_id in range(num_request):
        ref_id = request_arr[job_id]
        requests.append([(job_id + 1) * interval, ref_id])
        requests_t0.append([0, ref_id])

    job_data["job_requests"] = requests
    json_file_name = "3-10x3-10_N={}_random.json".format(num_request)
    f = open(json_file_name, "w")
    json.dump(job_data, f, ensure_ascii=False, indent = 4)
    f.close()
    print("saved as:", json_file_name)

    # for responsiveness test
    #job_data["job_requests"] = requests_t0
    #json_file_name = "3-10x3-10_N={}_t=0_random.json".format(num_request)
    #f = open(json_file_name, "w")
    #json.dump(job_data, f, ensure_ascii=False, indent = 4)
    #f.close()
    #print("saved as:", json_file_name)

def get_param(kind):
    if kind == 1:
        return { "w": [5, 10], "h": [5, 10], "t": [2000, 4000] }
    elif kind == 2:
        return { "w": [6, 12], "h": [6, 12], "t": [6000, 8000] }
    elif kind == 3:
        return { "w": [7, 14], "h": [7, 14], "t": [10000, 20000] }
    # for test
    elif kind == 4:
        return { "w": [5, 9], "h": [5, 9], "t": [4000, 8000] }
    elif kind == 5:
        return { "w": [5, 11], "h": [5, 11], "t": [12000, 20000] }
    elif kind == 6:
        return { "w": [5, 13], "h": [5, 13], "t": [30000, 40000] }

    assert(False)

def gen_cuboid_dataset(dinfos, req_interval, json_name_prefix, num):
    rng = np.random.default_rng()

    for i in range(num):
        job_data = {"programs": []}

        num_requests = 0
        for dinfo in dinfos:
            [dtype, num] = dinfo
            num_requests += num
            params = get_param(dtype)

            for _ in range(num):
                [w1, w2] = params["w"]
                [h1, h2] = params["h"]
                [t1, t2] = params["t"]

                w = int(rng.integers(w1, w2))
                h = int(rng.integers(h1, h2))
                t = int(rng.integers(t1, t2))
                if w > h:
                    w, h = h, w

                job_data["programs"].append({"Cuboid": [{"pos": [0,0,0], "size_x": w, "size_y": h, "size_z": t}]})

        request_arr = np.arange(num_requests, dtype=int)
        np.random.shuffle(request_arr)
        request_arr = request_arr.tolist()
        requests = []
        for job_id in range(num_requests):
            req_time = req_interval * job_id
            ref_id = request_arr[job_id]
            requests.append([req_time, ref_id])

        job_data["job_requests"] = requests
        json_filename = (json_name_prefix + "-{}.json").format(i + 1)
        f = open(json_filename, "w")
        json.dump(job_data, f, ensure_ascii=False, indent = 4)
        f.close()
        print("saved as:", json_filename)


gen_cuboid_dataset(dinfos=[[4, 333], [5, 333], [6, 334]], req_interval=2000, json_name_prefix="A", num=100)


