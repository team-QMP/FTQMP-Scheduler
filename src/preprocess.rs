pub mod convert_to_cuboid;

pub use convert_to_cuboid::ConvertToCuboid;

use nalgebra::Vector3;
use qhull::Qh;
use rand::Rng;
use std::f64;

use serde::{Deserialize, Serialize};

use crate::program::{Polycube, Program};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreprocessKind {
    #[serde(rename = "convert-to-cuboid")]
    ConvertToCuboid,
}

pub trait Preprocessor {
    fn process(&self, program: Program) -> Program;
}

pub struct FloatCoordinate {
    x: f64,
    y: f64,
    z: f64,
}

#[allow(dead_code)]
fn is_equal_floatcoordinates(point1: &FloatCoordinate, point2: &FloatCoordinate) -> bool {
    // Check if two points are equal
    // input: point1: FloatCoordinate, point2: FloatCoordinate
    // output: bool
    point1.x == point2.x && point1.y == point2.y && point1.z == point2.z
}

#[allow(dead_code)]
fn print_floatcoordinates(points: &Vec<FloatCoordinate>) {
    // Print a list of points
    // input: points: Vec<FloatCoordinate>
    for point in points {
        println!("({}, {}, {})", point.x, point.y, point.z);
    }
}

#[allow(dead_code)]
fn create_random_floatcoordinates(n: i32) -> Vec<FloatCoordinate> {
    // Create a list of random points
    // input: n: i32 the number of points
    // output: points: Vec<FloatCoordinate>
    let mut rng = rand::thread_rng();
    let mut points: Vec<FloatCoordinate> = Vec::new();
    for _ in 0..n {
        points.push(FloatCoordinate {
            x: rng.gen_range(0.0..100.0),
            y: rng.gen_range(0.0..100.0),
            z: rng.gen_range(0.0..100.0),
        });
    }
    points
}

#[allow(dead_code)]
pub struct Cuboid {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    min_z: i32,
    max_z: i32,
}

#[allow(dead_code)]
fn polycube_to_cuboid(polycube: &Polycube) -> Cuboid {
    assert!(polycube.size() > 0);
    // Convert a polycube to a cuboid
    // input: polycube: Polycube
    // output: cuboid: Cuboid
    let (min_x, max_x, min_y, max_y, min_z, max_z) = polycube.blocks().iter().fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y, min_z, max_z), pos| {
            (
                i32::min(min_x, pos.x),
                i32::max(max_x, pos.x),
                i32::min(min_y, pos.y),
                i32::max(max_y, pos.y),
                i32::min(min_z, pos.z),
                i32::max(max_z, pos.z),
            )
        },
    );

    Cuboid {
        min_x,
        max_x,
        min_y,
        max_y,
        min_z,
        max_z,
    }
}

#[allow(dead_code)]
fn polycube_to_float_coordinates(polycube: &Polycube) -> Vec<FloatCoordinate> {
    // Convert a polycube to a list of points
    // input: polycube: Polycube
    // output: points: Vec<FloatCoordinate>
    let mut points: Vec<FloatCoordinate> = Vec::new();
    for i in 0..polycube.size() {
        let coordinate = polycube.index_to_xyz(i);
        points.push(FloatCoordinate {
            x: coordinate.x as f64,
            y: coordinate.y as f64,
            z: coordinate.z as f64,
        }); // add original point
        points.push(FloatCoordinate {
            x: coordinate.x as f64 + 1.0,
            y: coordinate.y as f64,
            z: coordinate.z as f64,
        }); // add point shifted in x direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64,
            y: coordinate.y as f64 + 1.0,
            z: coordinate.z as f64,
        }); // add point shifted in y direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64,
            y: coordinate.y as f64,
            z: coordinate.z as f64 + 1.0,
        }); // add point shifted in z direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64 + 1.0,
            y: coordinate.y as f64 + 1.0,
            z: coordinate.z as f64,
        }); // add point shifted in x and y direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64 + 1.0,
            y: coordinate.y as f64,
            z: coordinate.z as f64 + 1.0,
        }); // add point shifted in x and z direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64,
            y: coordinate.y as f64 + 1.0,
            z: coordinate.z as f64 + 1.0,
        }); // add point shifted in y and z direction
        points.push(FloatCoordinate {
            x: coordinate.x as f64 + 1.0,
            y: coordinate.y as f64 + 1.0,
            z: coordinate.z as f64 + 1.0,
        }); // add point shifted in x, y and z direction
    }
    points
}

#[allow(dead_code)]
fn float_coordinates_to_convexhull(points: &[FloatCoordinate]) -> Qh {
    // Compute the convex hull of a list of points
    // input: points: Vec<FloatCoordinate>
    // output: qh: Qh
    let qh = Qh::builder()
        .compute(true)
        .build_from_iter(points.iter().map(|p| [p.x, p.y, p.z].to_vec()))
        .unwrap();
    qh
}

#[allow(dead_code)]
fn convex_hull_to_minimal_enclosing_box(
    points: &[Vector3<f64>],
) -> (Vector3<f64>, Vector3<f64>, f64) {
    let qh = Qh::builder()
        .compute(true)
        .build_from_iter(points.iter().map(|p| vec![p.x, p.y, p.z]))
        .unwrap();

    let mut min_volume = f64::MAX;
    let mut best_box_center = Vector3::zeros();
    let mut best_box_dimensions = Vector3::zeros();

    // Note: `Qh::facets` returns sentinal faces only.
    for face1 in qh.facets() {
        let normal1 = face1.normal().unwrap();
        let normal1 = Vector3::new(normal1[0], normal1[1], normal1[2]).normalize();

        for face2 in qh.facets() {
            if std::ptr::eq(&face1, &face2) {
                continue;
            }
            let normal2 = face2.normal().unwrap();
            let normal2 = Vector3::new(normal2[0], normal2[1], normal2[2]).normalize();

            if normal1.cross(&normal2).norm() < 1e-6 {
                // Skip if vectors are not independent
                continue;
            }

            let normal3 = normal1.cross(&normal2).normalize();
            let rotation = nalgebra::Rotation3::from_basis_unchecked(&[normal1, normal2, normal3]);

            let rotated_points: Vec<Vector3<f64>> = points.iter().map(|p| rotation * p).collect();

            let (min_point, max_point) = rotated_points.iter().fold(
                (Vector3::repeat(f64::MAX), Vector3::repeat(f64::MIN)),
                |(min, max), point| (min.inf(point), max.sup(point)),
            );

            let dimensions = max_point - min_point;
            let volume = dimensions.x * dimensions.y * dimensions.z;

            if volume.is_nan() || volume <= 0.0 {
                continue;
            }

            if volume < min_volume {
                min_volume = volume;
                best_box_center = rotation.inverse() * ((min_point + max_point) / 2.0);
                best_box_dimensions = dimensions;
            }
        }
    }

    (best_box_center, best_box_dimensions, min_volume)
}

#[cfg(test)]
pub mod test {
    // create random polycube
    use crate::preprocess::convex_hull_to_minimal_enclosing_box;
    use crate::preprocess::create_random_floatcoordinates;
    use crate::preprocess::float_coordinates_to_convexhull;
    use crate::preprocess::polycube_to_cuboid;
    use crate::preprocess::polycube_to_float_coordinates;
    use crate::preprocess::print_floatcoordinates;
    use crate::program::polycube::create_random_polycube;
    use nalgebra::Vector3;

    #[test]
    fn test_create_random_floatcoordinates() {
        let points = create_random_floatcoordinates(10); // create random points
        println!("Random points:");
        print_floatcoordinates(&points); // print points
                                         // check that the number of points is equal to the input
        assert_eq!(points.len(), 10);
    }

    #[test]
    fn test_polycube_to_cuboid() {
        // create a random polycube
        let randompolycube = create_random_polycube(10);
        println!("Random polycube:");
        randompolycube.print();

        let cuboid = polycube_to_cuboid(&randompolycube); // convert polycube to Cuboid
        println!("Cuboid:");
        println!(
            "({}, {}, {}, {}, {}, {})",
            cuboid.min_x, cuboid.max_x, cuboid.min_y, cuboid.max_y, cuboid.min_z, cuboid.max_z
        );
        // check that the cuboid contains all the points in the polycube
        for i in 0..randompolycube.size() {
            let coordinate = randompolycube.index_to_xyz(i);
            assert!(coordinate.x >= cuboid.min_x && coordinate.x <= cuboid.max_x);
            assert!(coordinate.y >= cuboid.min_y && coordinate.y <= cuboid.max_y);
            assert!(coordinate.z >= cuboid.min_z && coordinate.z <= cuboid.max_z);
        }
    }

    #[test]
    fn test_polycube_to_floatcoordinates() {
        // create a random polycube
        let randompolycube = create_random_polycube(10);
        println!("Random polycube:");
        randompolycube.print();

        let points = polycube_to_float_coordinates(&randompolycube); // convert polycube to FloatCoordinates
        println!("Float coordinates:");
        print_floatcoordinates(&points); // print points
                                         // check that the number of points is equal to the number of blocks in the polycube
        assert_eq!(points.len(), (randompolycube.size() as usize) * 8);
    }

    #[test]
    fn test_floatcoordinates_to_convexhull() {
        // create a random float coordinates
        let points = create_random_floatcoordinates(10);
        println!("Random points:");
        print_floatcoordinates(&points);
        // convert float coordinates to convex hull
        let qh = float_coordinates_to_convexhull(&points);
        for (i, face) in qh.facets().enumerate() {
            println!("\nFace {}:", i);
            let vertices = face.vertices().unwrap();
            println!("vertices:\n{:?}", vertices);
            // make a list of coordinates of vertex
            let coordinates: Vec<_> = vertices
                .iter()
                .map(|v| {
                    let point = &points[v.index_unchecked(&qh)];
                    [point.x, point.y, point.z]
                })
                .collect();
            println!("{:?}", coordinates);
        }
    }

    #[test]
    fn test_convexhull_to_minimal_enclosing_box() {
        let points = create_random_floatcoordinates(10);
        println!("Random points:");
        print_floatcoordinates(&points);
        // Convert `FloatCoordinate` to `Vector3`
        let vector_points: Vec<Vector3<f64>> =
            points.iter().map(|p| Vector3::new(p.x, p.y, p.z)).collect();
        // Calculate the minimal enclosing box
        let (center, dimensions, volume) = convex_hull_to_minimal_enclosing_box(&vector_points);

        println!("Minimal enclosing box center: {:?}", center);
        println!("Box dimensions (W, H, D): {:?}", dimensions);
        println!("Box volume: {:?}", volume);
    }
}
