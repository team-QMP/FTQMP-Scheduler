from floorplan import *
from compile_qc_to_ls import *
from random_qc import *
from qiskit.circuit import QuantumCircuit
import numpy as np
import os


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

def get_param(kind):
    if kind == 1:
        return { "q": [25, 100], "w": [5, 10], "h": [5, 10], "t": [10000, 20000] }
    elif kind == 2:
        return { "q": [25, 100], "w": [5, 10], "h": [5, 10], "t": [40000, 60000] }
    elif kind == 3:
        return { "q": [25, 100], "w": [5, 10], "h": [5, 10], "t": [80000, 100000] }
    elif kind == 4:
        return { "q": [101, 200], "w": [10, 20], "h": [10, 20], "t": [10000, 20000] }
    elif kind == 5:
        return { "q": [101, 200], "w": [10, 20], "h": [10, 20], "t": [40000, 60000] }
    elif kind == 6:
        return { "q": [101, 200], "w": [10, 20], "h": [10, 20], "t": [80000, 100000] }

    assert(False)

def gen_cuboid_dataset(dinfos, req_interval, out_dir, num):
    rng = np.random.default_rng()

    os.mkdir(out_dir)

    for i in range(num):
        job_data = {"programs": []}

        num_requests = 0
        for dinfo in dinfos:
            [dtype, num] = dinfo
            num_requests += num
            params = get_param(dtype)

            [q1, q2] = params["q"]
            [w1, w2] = params["w"]
            [h1, h2] = params["h"]
            [t1, t2] = params["t"]

            q_cands = []
            table = {}
            for q in range(q1, q2 + 1):
                data = []

                for w in range(w1, w2 + 1):
                    h = q // w
                    if q % w != 0 or h < h1 or h2 < h:
                        continue
                    data.append([w, h])

                if len(data) > 0:
                    q_cands.append(q)
                    table[q] = data

            for _ in range(num):
                q_idx = int(rng.integers(0, len(q_cands)))
                q = q_cands[q_idx]
                table_idx = int(rng.integers(0, len(table[q])))
                [w, h] = table[q][table_idx]
                t = int(rng.integers(t1, t2 + 1))

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

        json_path = (out_dir + "/requests-{}.json").format(i + 1)
        f = open(json_path, "w")
        json.dump(job_data, f, ensure_ascii=False, indent = 4)
        f.close()
        print("saved as:", json_path)


os.mkdir("dataset")

gen_cuboid_dataset(dinfos=[[1, 200], [2, 20], [3, 20], [4, 20], [5, 20], [6, 20]], req_interval=0, out_dir="dataset/A", num=50)
gen_cuboid_dataset(dinfos=[[1, 20], [2, 200], [3, 20], [4, 20], [5, 20], [6, 20]], req_interval=0, out_dir="dataset/B", num=50)
gen_cuboid_dataset(dinfos=[[1, 20], [2, 20], [3, 200], [4, 20], [5, 20], [6, 20]], req_interval=0, out_dir="dataset/C", num=50)
gen_cuboid_dataset(dinfos=[[1, 20], [2, 20], [3, 20], [4, 200], [5, 20], [6, 20]], req_interval=0, out_dir="dataset/D", num=50)
gen_cuboid_dataset(dinfos=[[1, 20], [2, 20], [3, 20], [4, 20], [5, 200], [6, 20]], req_interval=0, out_dir="dataset/E", num=50)
gen_cuboid_dataset(dinfos=[[1, 20], [2, 20], [3, 20], [4, 20], [5, 20], [6, 200]], req_interval=0, out_dir="dataset/F", num=50)
gen_cuboid_dataset(dinfos=[[1, 50], [2, 50], [3, 50], [4, 50], [5, 50], [6, 50]], req_interval=0, out_dir="dataset/G", num=50)
gen_cuboid_dataset(dinfos=[[1, 90], [2, 90], [3, 90], [4, 10], [5, 10], [6, 10]], req_interval=0, out_dir="dataset/H", num=50)
gen_cuboid_dataset(dinfos=[[1, 10], [2, 10], [3, 10], [4, 90], [5, 90], [6, 90]], req_interval=0, out_dir="dataset/I", num=50)
