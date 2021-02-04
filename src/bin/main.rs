use cgfs::{ Canvas, Color, Scene, Sphere, Point };
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use png;

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

    let mut canvas = Canvas::new(512, 512, Color::black());
    let mut scene = Scene::new(1., 1., 1., Color::white());
    scene.add_object(Box::new(Sphere::new(Point { x: 0., y: -1., z: 3. }, 1., Color::red())));
    scene.add_object(Box::new(Sphere::new(Point { x: 2., y: 0., z: 4. }, 1., Color::blue())));
    scene.add_object(Box::new(Sphere::new(Point { x: -2., y: 0., z: 4. }, 1., Color::green())));

    // let t1 = std::time::SystemTime::now();
    // canvas.render(&scene);
    // let t2 = std::time::SystemTime::now();
    // println!("single thread render time: {:?}", t2.duration_since(t1));

    let t1 = std::time::SystemTime::now();
    canvas.render_mth(&scene);
    let t2 = std::time::SystemTime::now();
    println!("multi thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./output.png");
}