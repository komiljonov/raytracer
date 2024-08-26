use std::sync::Arc;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use indicatif::ProgressBar;
use std::cell::RefCell;
use rand::prelude::*;

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

// Create a thread-local RNG
thread_local! {
    static THREAD_RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
}

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
    } else {
        let unit_direction = Vec3::unit_vector(&r.direction());
        let t = 0.5 * (unit_direction.y() + 1.0);

        Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t
    }
}

fn random_in_unit_sphere() -> Vec3 {
    THREAD_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let unit_vec = Vec3::new(1.0, 1.0, 1.0);

        loop {
            let p = 2.0 * Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()) - unit_vec;
            if p.squared_length() < 1.0 {
                return p;
            }
        }
    })
}

fn main() {
    let width = 800;
    let height = 400;
    let samples = 100;
    let max_value = 255;
    
    // Set the number of worker threads
    let num_threads = 16;

    let mut list: Vec<Box<dyn Hittable + Send + Sync>> = Vec::new();

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
            fuzz: 0.3,
        },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        material::Material::Dielectric { ref_idx: 1.5 },
    )));
    list.push(Box::new(Sphere::sphere(
        Vec3::new(-1.0, 0.0, -1.0),
        -0.45,
        material::Material::Dielectric { ref_idx: 1.5 },
    )));

    let world = Arc::new(HittableList::new(list));

    let look_from = Vec3::new(3.0, 3.0, 2.0);
    let look_at = Vec3::new(0.0, 0.0, -1.0);

    let dist_to_focus = (look_from - look_at).length();
    let aperture = 2.0;

    let camera = Camera::new(
        look_from,
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        width as f32 / height as f32,
        aperture,
        dist_to_focus,
    );

    let bar = ProgressBar::new((height * width + 1) as u64);
    bar.inc(1);
    println!("P3\n{} {}\n{}", width, height, max_value);

    // Build a custom thread pool with the specified number of threads
    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();

    let pixels: Vec<Vec3> = pool.install(|| {
        (0..height)
            .into_par_iter()
            .rev()
            .flat_map(|j| {
                (0..width)
                    .into_par_iter()
                    .map(|i| {
                        let mut col = Vec3::default();

                        for _ in 0..samples {
                            let u = (i as f32 + THREAD_RNG.with(|rng| rng.borrow_mut().gen::<f32>())) / width as f32;
                            let v = (j as f32 + THREAD_RNG.with(|rng| rng.borrow_mut().gen::<f32>())) / height as f32;

                            let r = camera.get_ray(u, v);
                            col = col + color(&r, &world, 0);
                        }

                        col = col / samples as f32;
                        col = Vec3::new(col.r().sqrt(), col.g().sqrt(), col.b().sqrt());
                        bar.inc(1);
                        col
                    })
                    .collect::<Vec<Vec3>>()
            })
            .collect()
    });

    bar.finish();

    for pixel in pixels {
        let ir = (255.99 * pixel.r()) as i32;
        let ig = (255.99 * pixel.g()) as i32;
        let ib = (255.99 * pixel.b()) as i32;

        println!("{} {} {}", ir, ig, ib);
    }
}