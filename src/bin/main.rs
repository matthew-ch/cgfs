use cgfs::*;
use png;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn save_canvas_to(canvas: &Canvas, p: &str) {
    let path = Path::new(p);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, canvas.get_width(), canvas.get_height());
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(canvas.data()).unwrap();
}

fn ray_tracing() {
    let mut canvas = Canvas::new(600, 600, Color::black());
    let mut scene = Scene::new(1., 1., Color { r: 225., g: 230., b: 252. });
    
    let red_sphere = SphereObject {
        sphere: Sphere {
            center: (0., -1.5, 3.).into(),
            radius: 1.,
        },
        material: Material{
            color: Color::red(),
            specular: 500,
            reflective: 0.2,
            transparency: None,
        },
    };

    let blue_sphere = SphereObject {
        sphere: Sphere {
            center: (2., 1., 4.).into(),
            radius: 1.,
        },
        material: Material {
            color: Color::blue(),
            specular: 500,
            reflective: 0.2,
            transparency: None,
        },
    };

    let green_sphere = SphereObject {
        sphere: Sphere {
            center: (-2., 0., 4.).into(),
            radius: 1.,
        },
        material: Material {
            color: Color::green(),
            specular: 10,
            reflective: 0.4,
            transparency: None,
        },
    };

    let yellow_sphere = SphereObject {
        sphere: Sphere {
            center: (0., -5001., 0.).into(),
            radius: 5000.,
        },
        material: Material {
            color: Color::yellow(),
            specular: 1000,
            reflective: 0.2,
            transparency: None,
        },
    };

    scene.add_object(Box::new(red_sphere));
    scene.add_object(Box::new(blue_sphere));
    scene.add_object(Box::new(green_sphere));
    scene.add_object(Box::new(yellow_sphere));

    let subtraction = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: (0., 3., 5.).into(),
            radius: 1.5,
        },
        operation: BooleanOperation::SUBTRACTION,
        sphere_b: Sphere {
            center: (0., 4., 3.).into(),
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
            transparency: None,
        },
    };

    scene.add_object(Box::new(subtraction));

    let intersection = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: (-1., 2., 2.).into(),
            radius: 0.6,
        },
        operation: BooleanOperation::INTERSECTION,
        sphere_b: Sphere {
            center: (-1.4, 1.8, 1.9).into(),
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
            transparency: None,
        },
    };

    scene.add_object(Box::new(intersection));

    let union = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: (0.4, 1., 2.5).into(),
            radius: 0.6,
        },
        operation: BooleanOperation::UNION,
        sphere_b: Sphere {
            center: (0.3, 0.8, 2.3).into(),
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
            transparency: None,
        },
    };

    scene.add_object(Box::new(union));

    scene.add_light(Box::new(AmbientLight { intensity: 0.2 }));
    scene.add_light(Box::new(PointLight {
        intensity: 0.6,
        position: (2., 1., 0.).into(),
    }));
    scene.add_light(Box::new(DirectionalLight {
        intensity: 0.2,
        direction: (1., 4., 4., 0.).into(),
    }));

    scene.set_camera(
        (1.5, 1.0, -6.0).into(),
        Matrix::compose(vec![
            Matrix::rotation_x(-5.),
            Matrix::rotation_y(-5.),
            Matrix::rotation_z(-10.),
        ]),
        1.5,
    );

    let vertices = [
        (0., 2., 0.).into(),
        (3., 5., 5.).into(),
        (0., 3.5, -1.).into(),
        (-3., 5., 5.).into(),
    ];

    let polyhedron = PolyhedronObject {
        triangles: vec![
            Triangle::new(vertices[0], vertices[2], vertices[1],),
            Triangle::new(vertices[0], vertices[3], vertices[2],),
            Triangle::new(vertices[0], vertices[1], vertices[3],),
            Triangle::new(vertices[1], vertices[2], vertices[3],),
        ],
        material: Material {
            color: Color::black(),
            reflective: 0.8,
            specular: 300,
            transparency: Some(1.02),
        },
    };

    scene.add_object(Box::new(polyhedron));

    let lense = BooleanOperationSpheresObject {
        sphere_a: Sphere {
            center: (0.5, 1.0, -0.5).into(),
            radius: 1.,
        },
        operation: BooleanOperation::INTERSECTION,
        sphere_b: Sphere {
            center: (0.5, 1.5, -1.5).into(),
            radius: 1.,
        },
        material: Material {
            color: Color::black(),
            transparency: Some(1.01),
            reflective: 0.5,
            specular: 100,
        },
    };

    scene.add_object(Box::new(lense));

    // let t1 = std::time::SystemTime::now();
    // canvas.render(&scene, 3, 3);
    // let t2 = std::time::SystemTime::now();
    // println!("single thread render time: {:?}", t2.duration_since(t1));

    let t1 = std::time::SystemTime::now();
    canvas.render_mth(&scene, 3, 5, 3);
    let t2 = std::time::SystemTime::now();
    println!("multi thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./output.png");
}

fn rasterization() {
    let mut canvas = Canvas::new(600, 600, Color::white() * 0.9);
    let mut scene = Scene::new(1., 1., Color::black());

    scene.add_model(SceneModel::new(
        "cube".into(),
        vec![
            [1., 1., 1.],
            [-1., 1., 1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., -1.],
            [1., -1., -1. ],
        ].into_iter().map(|v| v.into()).collect(),
        vec![
            ([0, 1, 2], Color::red(), 50).into(),
            ([0, 2, 3], Color::red(), 50).into(),
            ([4, 0, 3], Color::green(), 50).into(),
            ([4, 3, 7], Color::green(), 50).into(),
            ([5, 4, 7], Color::blue(), 50).into(),
            ([5, 7, 6], Color::blue(), 50).into(),
            ([1, 5, 6], Color::yellow(), 50).into(),
            ([1, 6, 2], Color::yellow(), 50).into(),
            ([4, 5, 1], Color::purple(), 50).into(),
            ([4, 1, 0], Color::purple(), 50).into(),
            ([2, 6, 7], Color::cyan(), 50).into(),
            ([2, 7, 3], Color::cyan(), 50).into(),
        ],
    ));

    scene.add_instance(SceneModelInstance {
        model_name: "cube".into(),
        transform: Matrix::compose(vec![
            Matrix::translation(-1.5, 0., 7.),
            Matrix::scale(0.75),
        ]),
    });

    scene.add_instance(SceneModelInstance {
        model_name: "cube".into(),
        transform: Matrix::compose(vec![
            Matrix::translation(1.25, 2.5, 7.5),
            Matrix::rotation_y(-195.),
        ]),
    });

    scene.add_model(SceneModel::create_sphere_model("sphere".into(), 12, Color::green(), 50));

    scene.add_instance(SceneModelInstance {
        model_name: "sphere".into(),
        transform: Matrix::compose(vec![
            Matrix::translation(1.75, -0.5, 7.),
            Matrix::scale(1.5),
        ]),
    });

    scene.add_light(Box::new(AmbientLight {
        intensity: 0.2,
    }));
    scene.add_light(Box::new(DirectionalLight {
        intensity: 0.2,
        direction: (-1., 0., 1., 0.).into(),
    }));
    scene.add_light(Box::new(PointLight {
        intensity: 0.6,
        position: (-3., 2., -10.).into(),
    }));

    scene.set_camera(
        (-3., 1., 2.).into(), 
        Matrix::rotation_y(30.),
        1.5,
    );

    let t1 = std::time::SystemTime::now();
    canvas.rasterize(&scene, Shading::PHONG, false);
    let t2 = std::time::SystemTime::now();
    println!("single thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./output.png");
}

fn compare() {
    let mut canvas = Canvas::new(600, 600, Color::white());
    let mut scene = Scene::new(1., 1., Color::white());
    
    let red_sphere = SphereObject {
        sphere: Sphere {
            center: (0., -1., 3.).into(),
            radius: 1.,
        },
        material: Material{
            color: Color::red(),
            specular: 50,
            reflective: 0.,
            transparency: None,
        },
    };

    let blue_sphere = SphereObject {
        sphere: Sphere {
            center: (2., 0., 4.).into(),
            radius: 1.,
        },
        material: Material {
            color: Color::blue(),
            specular: 50,
            reflective: 0.,
            transparency: None,
        },
    };

    let green_sphere = SphereObject {
        sphere: Sphere {
            center: (-2., 0., 4.).into(),
            radius: 1.,
        },
        material: Material {
            color: Color::green(),
            specular: 50,
            reflective: 0.,
            transparency: None,
        },
    };

    let yellow_sphere = SphereObject {
        sphere: Sphere {
            center: (0., -5001., 0.).into(),
            radius: 5000.,
        },
        material: Material {
            color: Color::yellow(),
            specular: 50,
            reflective: 0.,
            transparency: None,
        },
    };

    scene.add_object(Box::new(red_sphere));
    scene.add_object(Box::new(blue_sphere));
    scene.add_object(Box::new(green_sphere));
    scene.add_object(Box::new(yellow_sphere));

    scene.add_model(SceneModel::create_sphere_model("red_sphere".into(), 16, Color::red(), 50));
    scene.add_model(SceneModel::create_sphere_model("green_sphere".into(), 16, Color::green(), 50));
    scene.add_model(SceneModel::create_sphere_model("blue_sphere".into(), 16, Color::blue(), 50));
    scene.add_model(SceneModel::create_sphere_model("yellow_sphere".into(), 128, Color::yellow(), 50));
    scene.add_instance(SceneModelInstance {
        model_name: "red_sphere".into(),
        transform: Matrix::translation(0., -1., 3.),
    });
    scene.add_instance(SceneModelInstance {
        model_name: "green_sphere".into(),
        transform: Matrix::translation(-2., 0., 4.),
    });
    scene.add_instance(SceneModelInstance {
        model_name: "blue_sphere".into(),
        transform: Matrix::translation(2., 0., 4.),
    });
    scene.add_instance(SceneModelInstance {
        model_name: "yellow_sphere".into(),
        transform: Matrix::compose(vec![
            Matrix::translation(0., -5001., 0.),
            Matrix::scale(5000.),
        ]),
    });

    scene.add_light(Box::new(AmbientLight { intensity: 0.2 }));
    scene.add_light(Box::new(PointLight {
        intensity: 0.6,
        position: (2., 1., 0.).into(),
    }));
    scene.add_light(Box::new(DirectionalLight {
        intensity: 0.2,
        direction: (1., 4., 4., 0.).into(),
    }));

    scene.set_camera(
        (-1.5, 0.5, -0.5).into(),
        Matrix::rotation_y(10.),
        1.2
    );

    let t1 = std::time::SystemTime::now();
    canvas.render(&scene, 0, 1);
    let t2 = std::time::SystemTime::now();
    println!("single thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./comp-ray.png");

    canvas.clear(Color::white());

    let t1 = std::time::SystemTime::now();
    canvas.rasterize(&scene, Shading::PHONG, false);
    let t2 = std::time::SystemTime::now();
    println!("single thread render time: {:?}", t2.duration_since(t1));
    save_canvas_to(&canvas, r"./comp-ras.png");
}

fn main() {
    let mode = 3;
    match mode {
        1 => ray_tracing(),
        2 => rasterization(),
        3 => compare(),
        _ => (),
    };
}