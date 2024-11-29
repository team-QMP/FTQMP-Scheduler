extern crate kiss3d;

use kiss3d::nalgebra::Translation3;
use kiss3d::window::Window;
use kiss3d::light::Light;

use crate::ds::polycube::Polycube;
use crate::ds::schedule::Schedule;

#[allow(dead_code)]
pub fn visualize(polycube: &Polycube) {
    let mut window = Window::new("Block Visualize");
    let scale: f32 = 0.1;
    let margin: f32 = 0.1;
    for pos in polycube.blocks() {
        let mut c = window.add_cube(scale*(1.-margin),scale*(1.-margin),scale*(1.-margin));
        c.append_translation(&Translation3::new((pos.x as f32)*scale, (pos.y as f32)*scale,(pos.z as f32)*scale));
        if pos.x == 0 && pos.y == 0 && pos.z == 0 {
            c.set_color(1., 0., 0., );
        }
    }
    window.set_light(Light::StickToCamera);
    while window.render(){ }
}


#[allow(dead_code)]
pub fn render_cubes(polycubes: &Vec<Polycube>, cube_settings: &Vec<Schedule>) {
    println!("render cubes");
    let mut window = Window::new("Block Visualize");
    let scale: f32 = 0.1;
    let margin: f32 = 0.1;
    let basis_polycube = polycubes[0].clone();
    for pos in basis_polycube.blocks() {
        let mut c = window.add_cube(scale*(1.-margin),scale*(1.-margin),scale*(1.-margin));
        c.append_translation(&Translation3::new((pos.x as f32)*scale, (pos.y as f32)*scale,(pos.z as f32)*scale));
        if pos.x == 0 && pos.y == 0 && pos.z == 0 {
            c.set_color(1., 0., 0., );
        }
    }

    for i in 1..polycubes.len() {
        let polycube = &polycubes[i];
        let schedule = &cube_settings[i-1];
        for pos in polycube.blocks() {
            let mut c = window.add_cube(scale*(1.-margin),scale*(1.-margin),scale*(1.-margin));
            c.append_translation(&Translation3::new((pos.x as f32)*scale+(schedule.x as f32)*scale, (pos.y as f32)*scale+(schedule.y as f32)*scale,(pos.z as f32)*scale+(schedule.z as f32)*scale));
            c.set_color(0., 1., 0., );
        }
    }
    window.set_light(Light::StickToCamera);
    while window.render(){ }
}
