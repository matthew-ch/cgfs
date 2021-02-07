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
    let mut scene = Scene::new(1., 1., Color::black());
    scene.add_object(Box::new(Sphere {
        center: Point {
            x: 0.,
            y: -1.5,
            z: 3.,
        },
        radius: 1.,
        color: Color::red(),
        specular: 500,
        reflective: 0.2,
    }));
    scene.add_object(Box::new(Sphere {
        center: Point {
            x: 2.,
            y: 1.,
            z: 4.,
        },
        radius: 1.,
        color: Color::blue(),
        specular: 500,
        reflective: 0.5,
    }));
    scene.add_object(Box::new(Sphere {
        center: Point {
            x: -2.,
            y: 0.,
            z: 4.,
        },
        radius: 1.,
        color: Color::green(),
        specular: 10,
        reflective: 0.4,
    }));
    scene.add_object(Box::new(Sphere {
        center: Point {
            x: 0.,
            y: -5001.,
            z: 0.,
        },
        radius: 5000.,
        color: Color {
            r: 255.,
            g: 255.,
            b: 0.,
        },
        specular: 1000,
        reflective: 0.2,
    }));

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
        Point { x: 3.0, y: 1.5, z: -6.0 },
        Matrix::compose(&vec![
            Matrix::rotate_x(-10.),
            Matrix::rotate_y(15.),
            Matrix::rotate_z(10.),
        ]),
        2.0,
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
