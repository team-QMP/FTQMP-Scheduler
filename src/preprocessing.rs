use kiss3d::ncollide3d::shape::ConvexHull;
use rand::Rng;
use qhull::Qh;
use nalgebra::Vector3;
use std::f64;

use crate::program::Polycube;

pub struct FloatCoordinate {
    x: f64,
    y: f64,
    z: f64,
}

fn is_equal_floatcoordinates(point1: &FloatCoordinate, point2: &FloatCoordinate) -> bool {
    // Check if two points are equal
    // input: point1: FloatCoordinate, point2: FloatCoordinate
    // output: bool
    return point1.x == point2.x && point1.y == point2.y && point1.z == point2.z;
}

fn print_floatcoordinates(points: &Vec<FloatCoordinate>) {
    // Print a list of points
    // input: points: Vec<FloatCoordinate>
    for point in points {
        println!("({}, {}, {})", point.x, point.y, point.z);
    }
}

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
    return points;
}

pub struct Cuboid {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    min_z: i32,
    max_z: i32,
}

fn polycube_to_cuboid(polycube: &Polycube) -> Cuboid {
    // Convert a polycube to a cuboid
    // input: polycube: Polycube
    // output: cuboid: Cuboid
    let mut x_coordinates: Vec<i32> = Vec::new();
    let mut y_coordinates: Vec<i32> = Vec::new();
    let mut z_coordinates: Vec<i32> = Vec::new();
    for i in 0..polycube.size() {
        let coordinate = polycube.index_to_xyz(i);
        x_coordinates.push(coordinate.x as i32);
        y_coordinates.push(coordinate.y as i32);
        z_coordinates.push(coordinate.z as i32);
    }
    let min_x: i32;
    match x_coordinates.iter().min() {
        Some(n) => min_x = *n,
        None => unreachable!(),
    }
    let max_x: i32;
    match x_coordinates.iter().max() {
        Some(n) => max_x = *n,
        None => unreachable!(),
    }
    let min_y: i32;
    match y_coordinates.iter().min() {
        Some(n) => min_y = *n,
        None => unreachable!(),
    }
    let max_y: i32;
    match y_coordinates.iter().max() {
        Some(n) => max_y = *n,
        None => unreachable!(),
    }
    let min_z: i32;
    match z_coordinates.iter().min() {
        Some(n) => min_z = *n,
        None => unreachable!(),
    }
    let max_z: i32;
    match z_coordinates.iter().max() {
        Some(n) => max_z = *n,
        None => unreachable!(),
    }
    return Cuboid{min_x: min_x, max_x: max_x, min_y: min_y, max_y: max_y, min_z: min_z, max_z: max_z};
}


fn polycube_to_float_coordinates(polycube: &Polycube) -> Vec<FloatCoordinate> {
    // Convert a polycube to a list of points
    // input: polycube: Polycube
    // output: points: Vec<FloatCoordinate>
    let mut points: Vec<FloatCoordinate> = Vec::new();
    for i in 0..polycube.size() {
        let coordinate = polycube.index_to_xyz(i);
        points.push(FloatCoordinate{x: coordinate.x as f64, y: coordinate.y as f64, z: coordinate.z as f64}); // add original point
        points.push(FloatCoordinate{x: coordinate.x as f64 + 1.0, y: coordinate.y as f64, z: coordinate.z as f64}); // add point shifted in x direction
        points.push(FloatCoordinate{x: coordinate.x as f64, y: coordinate.y as f64 + 1.0, z: coordinate.z as f64}); // add point shifted in y direction
        points.push(FloatCoordinate{x: coordinate.x as f64, y: coordinate.y as f64, z: coordinate.z as f64 + 1.0}); // add point shifted in z direction
        points.push(FloatCoordinate{x: coordinate.x as f64 + 1.0, y: coordinate.y as f64 + 1.0, z: coordinate.z as f64}); // add point shifted in x and y direction
        points.push(FloatCoordinate{x: coordinate.x as f64 + 1.0, y: coordinate.y as f64, z: coordinate.z as f64 + 1.0}); // add point shifted in x and z direction
        points.push(FloatCoordinate{x: coordinate.x as f64, y: coordinate.y as f64 + 1.0, z: coordinate.z as f64 + 1.0}); // add point shifted in y and z direction
        points.push(FloatCoordinate{x: coordinate.x as f64 + 1.0, y: coordinate.y as f64 + 1.0, z: coordinate.z as f64 + 1.0}); // add point shifted in x, y and z direction
    }
    return points;
}

fn float_coordinates_to_convexhull<'a>(points: &'a Vec<FloatCoordinate>) -> Qh<'a> {
    // Compute the convex hull of a list of points
    // input: points: Vec<FloatCoordinate>
    // output: qh: Qh
    let qh = Qh::builder()
        .compute(true)
        .build_from_iter(points.iter().map(|p| [p.x, p.y, p.z].to_vec())).unwrap();
    return qh;
}

fn convex_hull_to_minimal_enclosing_box(points: &[Vector3<f64>]) -> (Vector3<f64>, Vector3<f64>, f64) {
    let qh = Qh::builder()
        .compute(true)
        .build_from_iter(points.iter().map(|p| vec![p.x, p.y, p.z]))
        .unwrap();

    let mut min_volume = f64::MAX;
    let mut best_box_center = Vector3::zeros();
    let mut best_box_dimensions = Vector3::zeros();

    for face1 in qh.all_faces() {
        if face1.is_sentinel() {
            continue;
        }
        let normal1 = Vector3::new(face1.normal()[0], face1.normal()[1], face1.normal()[2]).normalize();

        for face2 in qh.all_faces() {
            if face2.is_sentinel() || (&face1 as *const _) == (&face2 as *const _) {
                continue;
            }
            let normal2 = Vector3::new(face2.normal()[0], face2.normal()[1], face2.normal()[2]).normalize();

            if normal1.cross(&normal2).norm() < 1e-6 {
                // ベクトルが独立していない場合はスキップ
                continue;
            }

            let normal3 = normal1.cross(&normal2).normalize();
            let rotation = nalgebra::Rotation3::from_basis_unchecked(&[normal1, normal2, normal3]);

            let rotated_points: Vec<Vector3<f64>> = points
                .iter()
                .map(|p| rotation * p)
                .collect();

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

// fn convex_hull_to_minimal_enclosing_box(points: &[Vector3<f64>]) -> (Vector3<f64>, Vector3<f64>, f64) {
//     let qh = Qh::builder()
//         .compute(true)
//         .build_from_iter(points.iter().map(|p| vec![p.x, p.y, p.z]))
//         .unwrap();

//     let mut min_volume = f64::MAX;
//     let mut best_box_center = Vector3::zeros();
//     let mut best_box_dimensions = Vector3::zeros();

//     for face1 in qh.all_faces() {
//         if face1.is_sentinel() {
//             continue;
//         }
//         let normal1 = Vector3::new(face1.normal()[0], face1.normal()[1], face1.normal()[2]);

//         for face2 in qh.all_faces() {
//             if face2.is_sentinel() || (&face1 as *const _) == (&face2 as *const _) {
//                 continue;
//             }
//             let normal2 = Vector3::new(face2.normal()[0], face2.normal()[1], face2.normal()[2]);

//             if normal1.dot(&normal2).abs() > 1e-6 {
//                 continue;
//             }

//             let normal3 = normal1.cross(&normal2).normalize();
//             let rotation = nalgebra::Rotation3::from_basis_unchecked(&[normal1, normal2, normal3]);

//             let rotated_points: Vec<Vector3<f64>> = points
//                 .iter()
//                 .map(|p| rotation * p)
//                 .collect();

//             let (min_point, max_point) = rotated_points.iter().fold(
//                 (Vector3::repeat(f64::MAX), Vector3::repeat(f64::MIN)),
//                 |(min, max), point| (min.inf(point), max.sup(point)),
//             );

//             let dimensions = max_point - min_point;
//             let volume = dimensions.x * dimensions.y * dimensions.z;

//             if volume < min_volume {
//                 min_volume = volume;
//                 best_box_center = rotation.inverse() * ((min_point + max_point) / 2.0);
//                 best_box_dimensions = dimensions;
//             }
//         }
//     }

//     (best_box_center, best_box_dimensions, min_volume)
// }


#[cfg(test)]
pub mod test {
    // create random polycube
    use nalgebra::Vector3;
    use crate::ds::polycube::create_random_polycube;
    use crate::preprocessing::FloatCoordinate;
    use crate::preprocessing::Cuboid;
    use crate::preprocessing::create_random_floatcoordinates;
    use crate::preprocessing::polycube_to_cuboid;
    use crate::preprocessing::polycube_to_float_coordinates;
    use crate::preprocessing::float_coordinates_to_convexhull;
    use crate::preprocessing::print_floatcoordinates;
    use crate::preprocessing::convex_hull_to_minimal_enclosing_box;

    # [test]
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
        println!("({}, {}, {}, {}, {}, {})", cuboid.min_x, cuboid.max_x, cuboid.min_y, cuboid.max_y, cuboid.min_z, cuboid.max_z);
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

    # [test]
    fn test_floatcoordinates_to_convexhull() {
        // create a random float coordinates
        let points = create_random_floatcoordinates(10);
        println!("Random points:");
        print_floatcoordinates(&points);
        // convert float coordinates to convex hull
        let float_points: Vec<FloatCoordinate> = points.iter().map(|p| FloatCoordinate { x: p.x, y: p.y, z: p.z }).collect();
        let convexhull = float_coordinates_to_convexhull(&float_points);
        for (i, face) in convexhull.all_faces().enumerate() {
            let vertices = face.vertices();
            if !face.is_sentinel() {
                println!("\nFace {}:",i);
                for vertex in vertices{
                    // println!("\nvertex:");
                    // println!("{:?}", vertex);
                    // make a list of coordinates of vertex
                    let coordinates: Vec<_> = vertex.iter().map(|v| {
                        let point = &points[v.id() as usize];
                        [point.x, point.y, point.z]
                    }).collect();
                    println!("{:?}", coordinates);
                }
            }
        }
        assert_eq!(10, 10);
    }

    # [test]
    fn test_convexhull_to_minimal_enclosing_box(){
        let points = create_random_floatcoordinates(10);
        println!("Random points:");
        print_floatcoordinates(&points);
        // convert float coordinates to convex hull
        let float_points: Vec<FloatCoordinate> = points.iter().map(|p| FloatCoordinate { x: p.x, y: p.y, z: p.z }).collect();
        let convexhull = float_coordinates_to_convexhull(&float_points);
    // FloatCoordinateをVector3<f64>に変換
        let vector_points: Vec<Vector3<f64>> = points
            .iter()
            .map(|p| Vector3::new(p.x, p.y, p.z))
            .collect();
        // 最小内包ボックスを計算
        let (center, dimensions, volume) = convex_hull_to_minimal_enclosing_box(&vector_points);

        println!("Minimal enclosing box center: {:?}", center);
        println!("Box dimensions (W, H, D): {:?}", dimensions);
        println!("Box volume: {:?}", volume);
    }

}
