import math
import networkx as nx
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

# Make a floorplan of a quantum processor

def place_surface_code_qubits(
    random_dataset
    width, height,
    frame,
    num_data_qubits,
    pattern="block25"
):
    プロセッサ格子(width x height)上に量子ビットを配置する。

    Parameters
    ----------
    width : int
        格子の横幅
    height : int
        格子の縦幅
    frame : list of str
        ["top", "bottom", "left", "right"] の中からフレームにしたい辺を指定
        例: ["bottom", "right"] など
    num_data_qubits : int
        配置したい論理データ量子ビット数
    pattern : str, optional
        データ配置パターンを表す文字列。例:
          - "block25" : 内部を2x2部屋で敷き詰め、各部屋の右下1セルをデータ候補(25%)
          - "block44" : 3x3部屋で敷き詰め、各部屋の右下2x2セルをデータ候補(約44%)
          - "stripe50": 縦ストライプ(交互1列データ,1列アンシラ)
          - "stripe66": 縦ストライプ(2列データ,1列アンシラ)

    Returns
    -------
    dict
        {
            "width": int,
            "height": int,
            "frame_qubits": [(x,y), ...],    # フレームとして固定した量子ビット
            "data_qubits": [(x,y), ...],     # データ量子ビット
            "ancilla_qubits": [(x,y), ...],  # 残りのアンシラ量子ビット(内部)
            "final_fill_rate": float         # フレームも含めた全体に対するデータ比(0～1)
        }
    """
    # 1) まず全セルをアンシラ候補に
    all_coords = [(x, y) for y in range(height) for x in range(width)]
    ancilla_qubits = set(all_coords)
    data_qubits = set()
    frame_qubits = set()

    # 2) フレーム(端)を確定
    def is_in_frame(x, y):
        if "left" in frame and x == 0:
            return True
        if "right" in frame and x == width - 1:
            return True
        if "bottom" in frame and y == 0:
            return True
        if "top" in frame and y == height - 1:
            return True
        return False

    for (x, y) in all_coords:
        if is_in_frame(x, y):
            frame_qubits.add((x, y))

    # フレーム部分はアンシラ候補から除外(専用アンシラとする)
    ancilla_qubits -= frame_qubits

    # 内部領域のリスト
    interior_coords = sorted(ancilla_qubits, key=lambda c: (c[1], c[0]))
    # y昇順(下->上), x昇順(左->右)でソート

    # 3) パターンごとに データ候補セル を作成
    candidate_data_positions = []
    if pattern in ("block25", "block44"):
        # タイル(部屋)方式
        if pattern == "block25":
            block_w, block_h = 2, 2
            # 2x2のうち 右下1セルをデータ候補 => 25%
            local_data_offsets = [(1, 0)]
        else:
            block_w, block_h = 3, 3
            # 3x3のうち 右下2x2 => 4セル => ~44%
            local_data_offsets = [(1,0), (2,0), (1,1), (2,1)]

        xs = [c[0] for c in interior_coords]
        ys = [c[1] for c in interior_coords]
        if xs and ys:
            min_x, max_x = min(xs), max(xs)
            min_y, max_y = min(ys), max(ys)

            tile_candidates = []
            # 下端から block_h 刻み、左端から block_w 刻みで部屋を走査
            y_blk = min_y
            while y_blk <= max_y:
                x_blk = min_x
                while x_blk <= max_x:
                    # 1つの部屋の bottom-left = (x_blk, y_blk)
                    for (dx, dy) in local_data_offsets:
                        cx = x_blk + dx
                        cy = y_blk + dy
                        if (cx, cy) in interior_coords:
                            tile_candidates.append((cx, cy))
                    x_blk += block_w
                y_blk += block_h

            tile_candidates.sort(key=lambda c: (c[1], c[0]))
            candidate_data_positions = tile_candidates

    elif pattern == "stripe50":
        # 縦ストライプ => x%2==0 がデータ候補
        candidate_data_positions = [
            (x, y) for (x, y) in interior_coords if (x % 2 == 0)
        ]

    elif pattern == "stripe66":
        # 縦ストライプ => x%3 in [0,1] がデータ候補
        candidate_data_positions = [
            (x, y) for (x, y) in interior_coords if (x % 3 in [0,1])
        ]

    else:
        raise NotImplementedError(f"Unknown pattern '{pattern}'")

    # 4) 候補数チェック
    if len(candidate_data_positions) < num_data_qubits:
        raise ValueError(
            f"Not enough candidate cells for {num_data_qubits} data qubits. "
            f"Pattern='{pattern}', have={len(candidate_data_positions)}."
        )

    # 5) 必要数だけデータに割り当て
    chosen_data = candidate_data_positions[:num_data_qubits]
    data_qubits = set(chosen_data)

    # 6) アンシラから除外
    ancilla_qubits -= data_qubits

    # 7) 充填率 (フレーム込み)
    total_cells = width * height
    final_fill_rate = len(data_qubits) / total_cells

    # 8) 戻り値
    return {
        "width": width,
        "height": height,
        "frame_qubits": sorted(frame_qubits),
        "data_qubits": sorted(data_qubits),
        "ancilla_qubits": sorted(ancilla_qubits),
        "final_fill_rate": final_fill_rate
    }


def place_surface_code_qubits_without_size_const(
    frame,
    num_data_qubits,
    pattern="block25"
):
    """
    プロセッサのサイズ (width, height) を指定せず、
    データ量子ビット数とパターンから「必要最小限の内部領域」を自動で決め、
    frame=(["top","bottom","left","right"] のリスト) に従って
    フレームを取り付けたうえで、量子ビット配置を返す。

    Parameters
    ----------
    frame : list of str
        ["top","bottom","left","right"] のうち必要な端を列挙。
        例: ["bottom","right"] なら下端と右端をフレームに。
    num_data_qubits : int
        配置したいデータ量子ビット数。
        パターンによっては余計にスペースを取ることがあるが、
        これを満たせないほど小さくはしない。
    pattern : str
        "block25" => 2x2ブロック(各部屋1データ=25%)
        "block44" => 3x3ブロック(各部屋4データ=約44%)
        "stripe50" => 縦ストライプ(1列おきデータ=50%)
        "stripe66" => 縦ストライプ(2列データ,1列アンシラ=66%)

    Returns
    -------
    dict
        {
          "width": int,  # フレームを含む全体幅
          "height": int,  # フレームを含む全体高さ
          "frame_qubits": [(x,y), ...],
          "data_qubits": [(x,y), ...],
          "ancilla_qubits": [(x,y), ...],
          "final_fill_rate": float  # 0.0～1.0
        }

    Raises
    ------
    ValueError
        データ数(num_data_qubits)を配置できない場合。
    """

    # -----------------------------------------------------
    # 1) まず、"内部領域" の幅と高さを決定
    # -----------------------------------------------------
    if pattern == "block25":
        # 各2x2ブロックで 1 個のデータを置く
        # => num_data_qubits ブロック必要 => 内部は (2 * num_data_qubits) x 2
        #    (横にブロックを並べる想定)
        internal_w = 2 * num_data_qubits
        internal_h = 2

    elif pattern == "block44":
        # 3x3ブロック1つにつき 4 個のデータを置ける
        # => 必要ブロック数 = ceil(num_data_qubits / 4)
        # => 内部は (3 * Nblocks) x 3
        nblocks = math.ceil(num_data_qubits / 4)
        internal_w = 3 * nblocks
        internal_h = 3

    elif pattern == "stripe50":
        # 1 行で、(x % 2 == 0) がデータ => ちょうど半分がデータ
        # => num_data_qubits 個を置きたい => 内部に 2 * num_data_qubits カラムあれば足りる
        internal_w = 2 * num_data_qubits
        internal_h = 1

    elif pattern == "stripe66":
        # (x % 3 in [0,1]) がデータ => 3 列中 2 列がデータ => 2/3 ~= 66%
        # => 何列あれば num_data_qubits 個置けるか探る
        #    capacity(width) = 実際に x=0..(width-1) のうち x%3 in [0,1] の数
        #    これが >= num_data_qubits となる最小の width を探す
        def capacity_stripe66(width):
            c = 0
            for x in range(width):
                if x % 3 in [0,1]:
                    c += 1
            return c

        w_candidate = 0
        while True:
            if capacity_stripe66(w_candidate) >= num_data_qubits:
                internal_w = w_candidate
                break
            w_candidate += 1
        internal_h = 1

    else:
        raise NotImplementedError(f"Unknown pattern='{pattern}'")

    # -----------------------------------------------------
    # 2) フレームによる拡張サイズ
    #    frame=['left','right','top','bottom'] の有無に応じて
    #    +1列/行 する
    # -----------------------------------------------------
    offset_x = 0
    offset_y = 0
    total_w = internal_w
    total_h = internal_h

    # 左フレーム => x=0 がフレーム => 内部は x=1～(1+internal_w-1)
    if "left" in frame:
        offset_x = 1
        total_w += 1
    # 下フレーム => y=0 がフレーム => 内部は y=1～(1+internal_h-1)
    if "bottom" in frame:
        offset_y = 1
        total_h += 1
    # 右フレーム => 内部の右端のさらに +1
    if "right" in frame:
        total_w += 1
    # 上フレーム => 内部の上端のさらに +1
    if "top" in frame:
        total_h += 1

    # -----------------------------------------------------
    # 3) 全セル座標を一旦「アンシラ候補」にし、フレーム領域を除外
    #    その後 内部領域にパターン通りのデータを配置
    # -----------------------------------------------------
    all_coords = [(x, y) for y in range(total_h) for x in range(total_w)]
    ancilla_qubits = set(all_coords)
    frame_qubits = set()
    data_qubits = set()

    def is_in_frame(x, y):
        # 左端
        if ("left" in frame) and (x == 0):
            return True
        # 右端
        if ("right" in frame) and (x == total_w - 1):
            return True
        # 下端
        if ("bottom" in frame) and (y == 0):
            return True
        # 上端
        if ("top" in frame) and (y == total_h - 1):
            return True
        return False

    # フレーム領域を frame_qubits に
    for (x, y) in all_coords:
        if is_in_frame(x, y):
            frame_qubits.add((x, y))

    # フレームをアンシラ候補から除く
    ancilla_qubits -= frame_qubits

    # -----------------------------------------------------
    # 4) 内部領域でデータを配置
    #    具体的には offset_x..(offset_x+internal_w-1),
    #               offset_y..(offset_y+internal_h-1) が内部
    # -----------------------------------------------------

    # パターン別に "候補" を生成してから、
    # そこから num_data_qubits 個を置く。
    # 既に内部サイズが決まっているので、必ず余裕がある想定。

    if pattern == "block25":
        # internal_w = 2 * num_data_qubits, internal_h=2
        # => ブロック数 = num_data_qubits
        # => 各ブロック: 幅2,高さ2
        # => 右下1セル(x+1, y+0) をデータ候補
        candidate_positions = []
        for b in range(num_data_qubits):
            # ブロックの左下 = (offset_x + 2*b, offset_y + 0)
            blk_x0 = offset_x + 2*b
            blk_y0 = offset_y
            # 部屋内の右下 (1,0)
            data_x = blk_x0 + 1
            data_y = blk_y0 + 0
            # このセルが存在するはず
            candidate_positions.append((data_x, data_y))

        # 念のためチェック
        if len(candidate_positions) < num_data_qubits:
            raise ValueError("block25: internally not enough data positions. Something's wrong.")

        # 上記 candidate_positions と num_data_qubits は同数なのでそのまま
        chosen_data = candidate_positions[:num_data_qubits]

        # 残り3セルはアンシラになる

    elif pattern == "block44":
        # internal_w = 3*Nblocks, internal_h=3
        # 1ブロック(3x3)に 4個データ置ける => candidate はブロックごとに4セル
        # => 例えば 右下2x2 => (1,0),(2,0),(1,1),(2,1)
        nblocks = math.ceil(num_data_qubits / 4)
        # 各ブロックの左下 => (offset_x + 3*b, offset_y)
        candidate_positions = []
        for b in range(nblocks):
            blk_x0 = offset_x + 3*b
            blk_y0 = offset_y
            # ブロック内の4セル
            block_cells = [
                (blk_x0 + 1, blk_y0 + 0),
                (blk_x0 + 2, blk_y0 + 0),
                (blk_x0 + 1, blk_y0 + 1),
                (blk_x0 + 2, blk_y0 + 1),
            ]
            candidate_positions.extend(block_cells)

        # candidate_positions >= 4*nblocks
        # そこから num_data_qubits 個だけ使う
        # 余りが出る場合あり
        if len(candidate_positions) < num_data_qubits:
            raise ValueError("block44: internally not enough data positions. Something's wrong.")

        chosen_data = candidate_positions[:num_data_qubits]

    elif pattern == "stripe50":
        # internal_w=2*num_data_qubits,internal_h=1
        # => x=0,2,4,... がデータ候補
        candidate_positions = []
        for x in range(offset_x, offset_x + internal_w):
            # 1行のみ => y=offset_y
            y = offset_y
            if (x - offset_x) % 2 == 0:
                candidate_positions.append((x, y))

        if len(candidate_positions) < num_data_qubits:
            raise ValueError("stripe50: Not enough data capacity. Something's wrong.")

        chosen_data = candidate_positions[:num_data_qubits]

    elif pattern == "stripe66":
        # internal_w は先ほど計算した最小幅
        # => x%3 in [0,1] がデータ候補
        candidate_positions = []
        for x in range(offset_x, offset_x + internal_w):
            y = offset_y  # 高さ1
            if (x - offset_x) % 3 in [0,1]:
                candidate_positions.append((x, y))

        if len(candidate_positions) < num_data_qubits:
            raise ValueError("stripe66: Not enough data capacity. Something's wrong.")

        chosen_data = candidate_positions[:num_data_qubits]

    else:
        raise NotImplementedError("pattern not recognized in final placement")

    # データセットに追加
    data_qubits.update(chosen_data)

    # データにしたセルをアンシラ集合から外す
    ancilla_qubits -= data_qubits

    # -----------------------------------------------------
    # 5) 充填率
    # -----------------------------------------------------
    total_cells = total_w * total_h
    final_fill_rate = len(data_qubits) / total_cells

    return {
        "width": total_w,
        "height": total_h,
        "frame_qubits": sorted(frame_qubits),
        "data_qubits": sorted(data_qubits),
        "ancilla_qubits": sorted(ancilla_qubits),
        "final_fill_rate": final_fill_rate
    }

def visualize_qubit_layout(qubit_dict, show_data_indices=False):
    """
    配置結果をmatplotlibで可視化。
      - frame_qubits: lime
      - data_qubits : 10色サイクル
      - ancilla_qubits: 白
      - 外枠: 黒太線
      - show_data_indices=True でデータ量子ビットだけインデックス表示
        (0-based; sorted順)
    """
    width = qubit_dict["width"]
    height = qubit_dict["height"]
    frame_qubits = qubit_dict["frame_qubits"]
    data_qubits_list = qubit_dict["data_qubits"]
    ancilla_qubits = qubit_dict["ancilla_qubits"]
    final_rate = qubit_dict["final_fill_rate"]

    fig, ax = plt.subplots(figsize=(6, 6))
    ax.set_aspect("equal", adjustable="box")

    cmap = plt.get_cmap("tab10")

    # 1) 内部アンシラ
    for (x, y) in ancilla_qubits:
        rect = Rectangle((x, y), 1, 1,
                         edgecolor='lightgray', facecolor='white', linewidth=0.5)
        ax.add_patch(rect)

    # 2) フレーム
    for (x, y) in frame_qubits:
        rect = Rectangle((x, y), 1, 1,
                         edgecolor='black', facecolor='lime', linewidth=1.0)
        ax.add_patch(rect)

    # 3) データ
    for i, (x, y) in enumerate(data_qubits_list):
        color = cmap(i % 10)
        rect = Rectangle((x, y), 1, 1,
                         edgecolor='black', facecolor=color, linewidth=1.0)
        ax.add_patch(rect)

        if show_data_indices:
            ax.text(x+0.5, y+0.5, str(i),
                    ha='center', va='center', fontsize=8, color='black')

    # 全体枠 (太線)
    border = Rectangle((0, 0), width, height,
                       edgecolor='black', facecolor='none', linewidth=2.0)
    ax.add_patch(border)

    ax.set_xlim(0, width)
    ax.set_ylim(0, height)
    ax.set_xticks(range(width+1))
    ax.set_yticks(range(height+1))

    plt.title(f"Layout: {width}x{height}, data={len(data_qubits_list)}, fill={final_rate*100:.1f}%")
    plt.show()


def build_graph_from_qubit_floorplan(layout_dict):
    """
    place_surface_code_qubits系の戻り値(layout_dict)から、
    NetworkX の無向グラフを生成する。

    ノード:
      - "data_qubits", "ancilla_qubits", "frame_qubits" に含まれる
        (x,y) タプルの全量子ビット。
    ノード属性 "type":
      - data / ancilla / frame
    エッジ:
      - (x,y) と (x+1,y) が共に存在すれば辺
      - (x,y) と (x,y+1) が共に存在すれば辺

    Parameters
    ----------
    layout_dict : dict
        {
          "width": int,
          "height": int,
          "frame_qubits": [(x1,y1), ...],
          "data_qubits": [(x2,y2), ...],
          "ancilla_qubits": [(x3,y3), ...],
          "final_fill_rate": float
        }

    Returns
    -------
    G : networkx.Graph
        ノードは (x,y)。
        ノード属性 "type" は "data"/"ancilla"/"frame"。
        隣接セル間に無向辺が張られる。
    """

    G = nx.Graph()

    data_qubits = set(layout_dict["data_qubits"])
    ancilla_qubits = set(layout_dict["ancilla_qubits"])
    frame_qubits = set(layout_dict["frame_qubits"])

    # --- 1) 全ノードを追加 (ノード属性 "type" 付き) ---
    all_nodes = data_qubits | ancilla_qubits | frame_qubits
    for node in all_nodes:
        if node in data_qubits:
            node_type = "data"
        elif node in frame_qubits:
            node_type = "frame"
        else:
            node_type = "ancilla"
        G.add_node(node, type=node_type)

    # --- 2) 隣接セルに無向エッジを追加 ---
    for (x, y) in all_nodes:
        right = (x+1, y)
        if right in all_nodes:
            G.add_edge((x, y), right)
        up = (x, y+1)
        if up in all_nodes:
            G.add_edge((x, y), up)

    return G

def visualize_qubit_graph(G, show_labels=False):
    """
    build_graph_from_qubit_floorplan() で生成したNetworkXグラフ G を可視化する関数。
      - ノードは (x,y) をそのまま座標に配置
      - ノード属性 "type" に応じて色分け
      - エッジは灰色
      - show_labels=True ならノードに (x,y) をラベル表示
    """
    # 1) ノードを座標 (x,y) にマッピング
    pos = {}
    for node in G.nodes():
        x, y = node  # nodeは(x,y)
        pos[node] = (x, y)

    # 2) ノードタイプ -> カラー の辞書
    #    好みに応じて変更可能
    TYPE_COLORS = {
        "data":    "tab:blue",  # データ量子ビット
        "ancilla": "white",     # 内部アンシラ
        "frame":   "lime",      # フレーム
    }

    # 3) ノードをタイプ別に分けて描画
    #    それぞれ別の draw_networkx_nodes 呼び出しで色を変える
    node_types = ["frame", "data", "ancilla"]  # 描画順(フレームが目立つよう先に描画でもOK)
    for t in node_types:
        # typeが t のノード一覧
        nodes_of_type = [n for n in G.nodes() if G.nodes[n]["type"] == t]
        if not nodes_of_type:
            continue
        nx.draw_networkx_nodes(
            G, pos,
            nodelist=nodes_of_type,
            node_color=TYPE_COLORS[t],
            edgecolors="black",    # ノード枠線色
            node_size=300
        )

    # 4) エッジ描画
    nx.draw_networkx_edges(
        G, pos,
        edge_color="gray"
    )

    # 5) ノードラベル (オプション)
    if show_labels:
        labels = {n: f"{n}" for n in G.nodes()}
        nx.draw_networkx_labels(G, pos, labels=labels, font_size=8, font_color="black")

    # 軸の可視化設定
    plt.gca().set_aspect("equal", adjustable="box")

    # 軸範囲を、ノードの座標に合わせて多少余裕をもたせる
    all_x = [n[0] for n in G.nodes()]
    all_y = [n[1] for n in G.nodes()]
    plt.xlim(min(all_x) - 1, max(all_x) + 1)
    plt.ylim(min(all_y) - 1, max(all_y) + 1)

    # (好みで座標軸を表示/非表示にしたりtickを消したり可)
    plt.xticks(range(min(all_x) - 1, max(all_x) + 2))
    plt.yticks(range(min(all_y) - 1, max(all_y) + 2))

    plt.title(f"Qubit Graph (nodes={G.number_of_nodes()}, edges={G.number_of_edges()})")
    plt.show()

def width_to_height_for_num_qubit(num_qubit , width, frame = ["bottom","right"]):
    # num_qubit量子ビットのプログラムを横幅widthのプロセッサに配置する際の縦幅heightを求める
    # 1/4の間取り
    # print("num_qubit:",num_qubit)
    # print("width:",width)
    if "left" in frame:
        width -= 1
    if "right" in frame:
        width -= 1

    height = 2 * (num_qubit / math.ceil((width - 1) / 2))

    if "bottom" in frame:
        height += 1
    if "top" in frame:
        height += 1

    return math.ceil(height)

def place_surface_code_qubits_with_fixed_width(
    width,
    frame,
    num_data_qubits,
    pattern="block25"
):
    height = width_to_height_for_num_qubit(num_qubit=num_data_qubits, width=width, frame=frame)
    return place_surface_code_qubits(width, height, frame, num_data_qubits, pattern)

def auto_floorplan(num_data_qubits, width, pattern = "block25"):
    height = width_to_height_for_num_qubit(num_data_qubits, width, frame = ["bottom","right"])
    floorplan =  place_surface_code_qubits(width = width, height = height, num_data_qubits = num_data_qubits,  frame = ["bottom","right"], pattern = pattern)
    visualize_qubit_layout(floorplan, show_data_indices=True)
    plt.show()