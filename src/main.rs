extern crate kiss3d;

use kiss3d::nalgebra::Translation3;
use kiss3d::window::Window;
use kiss3d::light::Light;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug)]
struct PolyBlock{
    pos_list: Vec<Coordinate>,
}


fn visualize(poly_block: &PolyBlock) {
    let mut window = Window::new("Block Visualize");
    let scale: f32 = 0.1; // マージン含めた1グリッド分のサイズ(立方体のサイズはその1-margin(%)分になる)
    let margin: f32 = 0.1; // 立方体同士の間隔
    for pos in poly_block.pos_list.iter(){
        // 1個あたり1x1x1の立方体
        // これはまず原点に配置するということ
        let mut c = window.add_cube(scale*(1.-margin),scale*(1.-margin),scale*(1.-margin));

        // ブロックの位置を指定
        c.append_translation(&Translation3::new((pos.x as f32)*scale, (pos.y as f32)*scale,(pos.z as f32)*scale));
        if pos.x == 0 && pos.y == 0 && pos.z == 0 {
            c.set_color(1., 0., 0., );
        }
    }
    window.set_light(Light::StickToCamera);
    // z軸が奥行きになっている
    while window.render(){ }
}

fn create_basis_polyblock() -> PolyBlock{
    let mut poly_block: PolyBlock = PolyBlock{pos_list: Vec::new()};
    let pos_candidate_list: Vec<Coordinate> = vec![
        Coordinate{x: 0, y: 0, z: 0},
        Coordinate{x: 1, y: 0, z: 0},
        Coordinate{x: 0, y: 1, z: 0},
        Coordinate{x: 0, y: 2, z: 0},
        Coordinate{x: 0, y: 0, z: 1},
        Coordinate{x: 0, y: 0, z: 2},
        Coordinate{x: 0, y: 0, z: 3},
    ];
    for i in 0..pos_candidate_list.len(){
        let pos = pos_candidate_list[i];
        println!("{:?}", pos);
        poly_block.pos_list.push(pos);
    }
    println!("{:?}", poly_block);
    return poly_block;
}

fn main() {
    println!("Hello, world!");

    // まずは複数のブロックを組み合わせてポリキューブを作る
    // ランダムだとテトリスできなくて困るので、固定でポリキューブのセットを作る
    // 原点もよくわからないので、原点用のポリキューブをまず作る。

    // 脇坂さんのでキューブを定義し直す
    let basis_blocks = create_basis_polyblock();
    visualize(&basis_blocks);

    // ポリキューブを可視化する


    // ポリキューブを回転したり移動させる

}
