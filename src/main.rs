// extern crate kiss3d;

use qmp_scheduler::visualizer;
use qmp_scheduler::ds::polycube::Polycube;
use qmp_scheduler::ds::polycube::Coordinate;
use qmp_scheduler::ds::schedule::Schedule;

fn create_basis_polyblock() -> Polycube{
    let mut poly_cube = Polycube::new(vec![Coordinate::new(0, 0, 0)]);
    let pos_candidate_list: Vec<Coordinate> = vec![
        Coordinate{x: 1, y: 0, z: 0},
        Coordinate{x: 0, y: 1, z: 0},
        Coordinate{x: 0, y: 2, z: 0},
        Coordinate{x: 0, y: 0, z: 1},
        Coordinate{x: 0, y: 0, z: 2},
        Coordinate{x: 0, y: 0, z: 3},
    ];
    for i in 0..pos_candidate_list.len(){
        let pos = &pos_candidate_list[i];
        println!("{:?}", pos);
        poly_cube.add_block(pos.clone());
    }
    println!("{:?}", poly_cube);
    return poly_cube;
}

fn create_test_polyblock() -> (Polycube, Vec<Schedule>) {
    let mut poly_cube = Polycube::new(vec![Coordinate::new(0, 0, 0)]);
    let pos_candidate_list: Vec<Coordinate> = vec![
        Coordinate{x: 0, y: 1, z: 0},
        Coordinate{x: 0, y: 2, z: 0},
        Coordinate{x: 0, y: 3, z: 0},
        Coordinate{x: 0, y: 4, z: 0},
        Coordinate{x: 0, y: 5, z: 0},
        Coordinate{x: 0, y: 6, z: 0},
        Coordinate{x: 0, y: 7, z: 0},
        Coordinate{x: 0, y: 0, z: 1},
        Coordinate{x: 0, y: 2, z: 1},
        Coordinate{x: 0, y: 4, z: 1},
        Coordinate{x: 0, y: 4, z: 2},
        Coordinate{x: 0, y: 6, z: 1},
        Coordinate{x: 0, y: 6, z: 2},
        Coordinate{x: 0, y: 6, z: 3},
    ];
    // 移動量、回転量、反転を定義する
    let cube_settings: Vec<Schedule> = vec![
        Schedule{x: 3, y: 2, z: 1, rotate: 2, flip: false},
    ];
    for i in 0..pos_candidate_list.len(){
        let pos = &pos_candidate_list[i];
        println!("{:?}", pos);
        poly_cube.add_block(pos.clone());
    }
    println!("{:?}", poly_cube);
    return (poly_cube, cube_settings);
}

fn main() {
    println!("Hello, world!");

    // まずは複数のブロックを組み合わせてポリキューブを作る
    // ランダムだとテトリスできなくて困るので、固定でポリキューブのセットを作る
    // 原点もよくわからないので、原点用のポリキューブをまず作る。
    let basis_polycube = create_basis_polyblock();
    // 任意の形状のポリキューブを作る
    // ポリキューブを回転したり移動させる
    let (test_polycube, test_cube_settings) = create_test_polyblock();
    let polycubes: Vec<Polycube> = vec![basis_polycube, test_polycube];
    visualizer::render_cubes(&polycubes, &test_cube_settings);



    // 任意の形状のポリキューブを複数作る

    // 複数のポリキューブの位置と回転を定義して表示する
}
