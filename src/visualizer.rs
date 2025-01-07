use kiss3d::light::Light;
use kiss3d::nalgebra::Translation3;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::window::Window;

use crate::program::{Polycube, Program, ProgramFormat};
use crate::scheduler::Schedule;

struct RandomColorGenerator {
    rng: rand::rngs::StdRng,
}

impl RandomColorGenerator {
    fn new() -> Self {
        let seed = 42;
        Self {
            rng: rand::SeedableRng::seed_from_u64(seed),
        }
    }

    fn gen(&mut self) -> (f32, f32, f32) {
        use rand::prelude::*;
        (self.rng.gen(), self.rng.gen(), self.rng.gen())
    }
}

#[allow(dead_code)]
pub fn visualize(polycube: &Polycube) {
    let mut window = Window::new("Block Visualize");
    let scale: f32 = 0.1;
    let margin: f32 = 0.1;
    for pos in polycube.blocks() {
        let mut c = window.add_cube(
            scale * (1. - margin),
            scale * (1. - margin),
            scale * (1. - margin),
        );
        c.append_translation(&Translation3::new(
            (pos.x as f32) * scale,
            (pos.y as f32) * scale,
            (pos.z as f32) * scale,
        ));
        if pos.x == 0 && pos.y == 0 && pos.z == 0 {
            c.set_color(1., 0., 0.);
        }
    }
    window.set_light(Light::StickToCamera);
    while window.render() {}
}

#[allow(dead_code)]
pub fn render_program(programs: &[Program]) {
    let mut window = Window::new("Result Visualization");
    let scale = 0.1f32;
    let margin = 0.1f32;
    let mut rng_color = RandomColorGenerator::new();

    for program in programs {
        match program.format() {
            ProgramFormat::Polycube(p) => {
                let (r, g, b) = rng_color.gen();
                for block in p.blocks() {
                    let mut c = window.add_cube(
                        scale * (1. - margin),
                        scale * (1. - margin),
                        scale * (1. - margin),
                    );
                    let trans = Translation3::new(
                        (block.x as f32) * scale,
                        (block.y as f32) * scale,
                        (block.z as f32) * scale,
                    );
                    c.append_translation(&trans);
                    c.set_color(r, g, b);
                }
            }
        }
    }
    window.set_light(Light::StickToCamera);
    while window.render() {} // TOOD: draw in another thread?
}

#[allow(dead_code)]
pub fn render_cubes(polycubes: &[Polycube], cube_settings: &[Schedule]) {
    println!("render cubes");
    let mut window = Window::new("Block Visualize");
    let scale: f32 = 0.1;
    let margin: f32 = 0.1;
    let basis_polycube = polycubes[0].clone();
    for pos in basis_polycube.blocks() {
        let mut c = window.add_cube(
            scale * (1. - margin),
            scale * (1. - margin),
            scale * (1. - margin),
        );
        c.append_translation(&Translation3::new(
            (pos.x as f32) * scale,
            (pos.y as f32) * scale,
            (pos.z as f32) * scale,
        ));
        if pos.x == 0 && pos.y == 0 && pos.z == 0 {
            c.set_color(1., 0., 0.);
        }
    }

    for i in 1..polycubes.len() {
        let polycube = &polycubes[i];
        let schedule = &cube_settings[i - 1];
        for pos in polycube.blocks() {
            let mut c = window.add_cube(
                scale * (1. - margin),
                scale * (1. - margin),
                scale * (1. - margin),
            );
            let angle = (schedule.rotate as f32) * std::f32::consts::PI / 2.;
            let axis = Vector3::y_axis();
            let rotation = UnitQuaternion::from_axis_angle(&axis, angle);
            // XZ平面というか、ここではYZ平面で鏡面対称
            if schedule.flip {
                c.append_translation(&Translation3::new(
                    (pos.x as f32) * scale * (-1.),
                    (pos.y as f32) * scale,
                    (pos.z as f32) * scale,
                ));
            } else {
                c.append_translation(&Translation3::new(
                    (pos.x as f32) * scale,
                    (pos.y as f32) * scale,
                    (pos.z as f32) * scale,
                ));
            }
            c.append_rotation(&rotation);
            c.append_translation(&Translation3::new(
                (schedule.x as f32) * scale,
                (schedule.y as f32) * scale,
                (schedule.z as f32) * scale,
            ));
            c.set_color(0., 1., 0.);
        }
    }
    window.set_light(Light::StickToCamera);
    while window.render() {}
}

//fn create_basis_polyblock() -> Polycube{
//    let mut poly_cube = Polycube::new(vec![Coordinate::new(0, 0, 0)]);
//    let pos_candidate_list: Vec<Coordinate> = vec![
//        Coordinate{x: 1, y: 0, z: 0},
//        Coordinate{x: 0, y: 1, z: 0},
//        Coordinate{x: 0, y: 2, z: 0},
//        Coordinate{x: 0, y: 0, z: 1},
//        Coordinate{x: 0, y: 0, z: 2},
//        Coordinate{x: 0, y: 0, z: 3},
//    ];
//    for i in 0..pos_candidate_list.len(){
//        let pos = &pos_candidate_list[i];
//        println!("{:?}", pos);
//        poly_cube.add_block(pos.clone());
//    }
//    println!("{:?}", poly_cube);
//    return poly_cube;
//}
//
//fn create_test_polyblock() -> (Polycube, Vec<Schedule>) {
//    let mut poly_cube = Polycube::new(vec![Coordinate::new(0, 0, 0)]);
//    let pos_candidate_list: Vec<Coordinate> = vec![
//        Coordinate{x: 0, y: 1, z: 0},
//        Coordinate{x: 0, y: 2, z: 0},
//        Coordinate{x: 0, y: 3, z: 0},
//        Coordinate{x: 0, y: 4, z: 0},
//        Coordinate{x: 0, y: 5, z: 0},
//        Coordinate{x: 0, y: 6, z: 0},
//        Coordinate{x: 0, y: 7, z: 0},
//        Coordinate{x: 0, y: 0, z: 1},
//        Coordinate{x: 0, y: 2, z: 1},
//        Coordinate{x: 0, y: 4, z: 1},
//        Coordinate{x: 0, y: 4, z: 2},
//        Coordinate{x: 0, y: 6, z: 1},
//        Coordinate{x: 0, y: 6, z: 2},
//        Coordinate{x: 0, y: 6, z: 3},
//        Coordinate{x: 1, y: 4, z: 0},
//    ];
//    // 移動量、回転量、反転を定義する
//    let cube_settings: Vec<Schedule> = vec![
//        Schedule{x: 3, y: 2, z: 1, rotate: 2, flip: true},
//    ];
//    for i in 0..pos_candidate_list.len(){
//        let pos = &pos_candidate_list[i];
//        println!("{:?}", pos);
//        poly_cube.add_block(pos.clone());
//    }
//    println!("{:?}", poly_cube);
//    return (poly_cube, cube_settings);
//}
