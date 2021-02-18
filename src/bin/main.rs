use cgfs::*;
use png;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn save_canvas_to(canvas: &Canvas, p: &str) {
    let path = Path::new(p);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, canvas.get_width() as u32, canvas.get_height() as u32);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(canvas.data()).unwrap();
}

fn main() {
    let mut canvas = Canvas::new(600, 600, Color::black());
    let mut scene = Scene::new(1., 1., Color { r: 225., g: 230., b: 252. });
    
    let red_sphere = SphereObject {
        sphere: Sphere {
            center: Point {
                x: 0.,
                y: -1.5,
                z: 3.,
            },
            radius: 1.,
        },
        material: Material{
            color: Color::red(),
            specular: 500,
            reflective: 0.2,
        },
    };

    let blue_sphere = SphereObject {
        sphere: Sphere {
            center: Point {
                x: 2.,
                y: 1.,
                z: 4.,
            },
            radius: 1.,
        },
        material: Material {
            color: Color::blue(),
            specular: 500,
            reflective: 0.2,
        },
    };

    let green_sphere = SphereObject {
        sphere: Sphere {
            center: Point {
                x: -2.,
                y: 0.,
                z: 4.,
            },
            radius: 1.,
        },
        material: Material {
            color: Color::green(),
            specular: 10,
            reflective: 0.4,
        },
    };

    let yellow_sphere = SphereObject {
        sphere: Sphere {
            center: Point {
                x: 0.,
                y: -5001.,
                z: 0.,
            },
            radius: 5000.,
        },
        material: Material {
            color: Color {
                r: 255.,
                g: 255.,
                b: 0.,
            },
            specular: 1000,
            reflective: 0.2,
        },
    };

    scene.add_object(Box::new(red_sphere));
    scene.add_object(Box::new(blue_sphere));
    scene.add_object(Box::new(green_sphere));
    scene.add_object(Box::new(yellow_sphere));

    let subtraction = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: Point {
                x: 0.,
                y: 3.,
                z: 5.,
            },
            radius: 1.5,
        },
        operation: BooleanOperation::SUBTRACTION,
        sphere_b: Sphere {
            center: Point {
                x: 0.,
                y: 4.,
                z: 3.,
            },
            radius: 1.5,
        },
        material: Material {
            color: Color {
                r: 0.,
                g: 255.,
                b: 255.,
            },
            specular: 50,
            reflective: 0.1,
        },
    };

    scene.add_object(Box::new(subtraction));

    let intersection = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: Point {
                x: -1.0,
                y: 2.0,
                z: 2.0,
            },
            radius: 0.6,
        },
        operation: BooleanOperation::INTERSECTION,
        sphere_b: Sphere {
            center: Point {
                x: -1.4,
                y: 1.8,
                z: 1.9,
            },
            radius: 0.5,
        },
        material: Material {
            color: Color {
                r: 255.,
                g: 0.,
                b: 255.,
            },
            specular: 100,
            reflective: 0.2,
        },
    };

    scene.add_object(Box::new(intersection));

    let union = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: Point {
                x: 0.4,
                y: 1.0,
                z: 2.5,
            },
            radius: 0.6,
        },
        operation: BooleanOperation::UNION,
        sphere_b: Sphere {
            center: Point {
                x: 0.3,
                y: 0.8,
                z: 2.3,
            },
            radius: 0.5,
        },
        material: Material {
            color: Color {
                r: 128.,
                g: 128.,
                b: 128.,
            },
            specular: 20,
            reflective: 0.4,
        },
    };

    scene.add_object(Box::new(union));

    scene.add_light(Box::new(AmbientLight { intensity: 0.2 }));
    scene.add_light(Box::new(PointLight {
        intensity: 0.6,
        position: Point {
            x: 2.,
            y: 1.,
            z: 0.,
        },
    }));
    scene.add_light(Box::new(DirectionalLight {
        intensity: 0.2,
        direction: Vector {
            dx: 1.,
            dy: 4.,
            dz: 4.,
        },
    }));

    scene.set_camera(
        Point { x: 1.5, y: 1.0, z: -6.0 },
        Matrix::compose(&vec![
            Matrix::rotate_x(5.),
            Matrix::rotate_y(5.),
            Matrix::rotate_z(10.),
        ]),
        1.5,
    );

    // let t1 = std::time::SystemTime::now();
    // canvas.render(&scene, 3, 3);
    // let t2 = std::time::SystemTime::now();
    // println!("single thread render time: {:?}", t2.duration_since(t1));

    let t1 = std::time::SystemTime::now();
    canvas.render_mth(&scene, 3, 3, 3);
    let t2 = std::time::SystemTime::now();
    println!("multi thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./output.png");
}
