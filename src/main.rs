// A simple ray tracer built by translating the tutorial
// https://raytracing.github.io/books/RayTracingInOneWeekend.html
// to Rust

// TODO: pull these mod declarations out into a separate lib.rs
mod camera;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod sphere;
mod vec3;

use camera::Camera;
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use material::scatter;
use ray::Ray;
use sphere::Sphere;
use vec3::Vec3;

use rand::prelude::*;

fn color(r: &Ray, world: &HittableList, depth: i32) -> Vec3 {
    let _rec = HitRecord::default();

    if let Some(rec) = world.hit(&r, 0.001, std::f32::MAX) {
        let mut scattered = Ray::new(Vec3::default(), Vec3::default());
        let mut attenuation = Vec3::default();

        if depth < 50 && scatter(&rec.material, r, &rec, &mut attenuation, &mut scattered) {
            return attenuation * color(&scattered, world, depth + 1);
        } else {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        // return 0.5
        //     * Vec3::new(
        //         rec.normal().x() + 1.0,
        //         rec.normal().y() + 1.0,
        //         rec.normal().z() + 1.0,
        //     );
    } else {
        let unit_direction = Vec3::unit_vector(&r.direction());
        let t = 0.5 * (unit_direction.y() + 1.0);

        Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t
    }
}

fn random_in_unit_sphere() -> Vec3 {
    let mut p = Vec3::default();
    let mut rng = rand::thread_rng();

    loop {
        p = 2.0 * Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>())
            - Vec3::new(1.0, 1.0, 1.0);

        if p.squared_length() < 1.0 {
            return p;
        }
    }
}

fn main() {
    //println!("A raytracer in Rust!");

    let width = 800;
    let height = 400;
    let samples = 100;
    let max_value = 255;

    let mut list: Vec<Box<dyn Hittable>> = Vec::new();

    list.push(Box::new(Sphere::sphere(
        Vec3::new(0.0, 0.0, -1.0),
        0.5,
        material::Material::Lambertian {
            albedo: Vec3::new(0.1, 0.2, 0.5),
        },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(0.0, -100.5, -1.0),
        100f32,
        material::Material::Lambertian {
            albedo: Vec3::new(0.8, 0.8, 0.0),
        },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        material::Material::Metal {
            albedo: Vec3::new(0.8, 0.6, 0.2),
            fuzz: 0.0
        },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        material::Material::Dielectric { ref_idx: 1.5 },
    )));

    let world = HittableList::new(list);

    let camera = Camera::new();

    let mut rng = rand::thread_rng();

    // we use a plan txt ppm to start building images
    println!("P3\n{} {}\n{}", width, height, max_value);

    for j in (0..height).rev() {
        for i in 0..width {
            let mut col = Vec3::default();

            for _ in 0..samples {
                let u = (i as f32 + rng.gen::<f32>()) / width as f32;
                let v = (j as f32 + rng.gen::<f32>()) / height as f32;

                let r = &camera.get_ray(u, v);

                col = col + color(&r, &world, 0);
            }

            col = col / samples as f32;

            col = Vec3::new(col.r().sqrt(), col.g().sqrt(), col.b().sqrt());

            let ir = (255.99 * col.r()) as i32;
            let ig = (255.99 * col.g()) as i32;
            let ib = (255.99 * col.b()) as i32;

            println!("{} {} {}", ir, ig, ib);
        }
    }
}
