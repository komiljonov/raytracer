use indicatif::ProgressBar;
use rand::prelude::*;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::sync::Arc;

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
            let p =
                2.0 * Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()) - unit_vec;
            if p.squared_length() < 1.0 {
                return p;
            }
        }
    })
}

fn main() -> io::Result<()> {
    let width = 720;
    let height = 1024;
    let samples = 500;
    let max_value = 255;

    // Set the number of worker threads
    let num_threads = 8;

    let mut list: Vec<Box<dyn Hittable + Send + Sync>> = Vec::new();

    let mut rng = rand::thread_rng();

    list.push(Box::new(Sphere::sphere(
        Vec3::new(0.0, -1000.0, -1.0),
        1000.0,
        material::Material::Lambertian {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        },
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.gen::<f32>();
            let center = Vec3::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    list.push(Box::new(Sphere::sphere(
                        center,
                        0.2,
                        material::Material::Lambertian {
                            albedo: Vec3::new(
                                rng.gen::<f32>() * rng.gen::<f32>(),
                                rng.gen::<f32>() * rng.gen::<f32>(),
                                rng.gen::<f32>() * rng.gen::<f32>(),
                            ),
                        },
                    )));
                } else if choose_mat < 0.95 {
                    //metal
                    list.push(Box::new(Sphere::sphere(
                        center,
                        0.2,
                        material::Material::Metal {
                            albedo: Vec3::new(
                                0.5 * (1.0 + rng.gen::<f32>()),
                                0.5 * (1.0 + rng.gen::<f32>()),
                                0.5 * (1.0 + rng.gen::<f32>()),
                            ),
                            fuzz: (0.5 * rng.gen::<f32>()),
                        },
                    )));
                } else {
                    //glass
                    list.push(Box::new(Sphere::sphere(
                        center,
                        0.2,
                        material::Material::Dielectric { ref_idx: 1.5 },
                    )));
                }
            }
        }
    }

    list.push(Box::new(Sphere::sphere(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        material::Material::Dielectric { ref_idx: 1.5 },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        material::Material::Lambertian {
            albedo: Vec3::new(0.4, 0.2, 0.1),
        },
    )));

    list.push(Box::new(Sphere::sphere(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        material::Material::Metal {
            albedo: Vec3::new(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
    )));

    let world = Arc::new(HittableList::new(list));

    let look_from = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);

    // let dist_to_focus = (look_from - look_at).length();
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let vup = Vec3::new(0.0, 1.0, 0.0);

    let camera = Camera::new(
        look_from,
        look_at,
        vup,
        20.0,
        width as f32 / height as f32,
        aperture,
        dist_to_focus,
    );

    let bar = ProgressBar::new((height * width) as u64);
    // bar.inc(1);

    // Get the output file name from the command-line arguments or default to "res.ppm"
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 { &args[1] } else { "res.ppm" };

    let mut file = File::create(filename)?;

    writeln!(file, "P3\n{} {}\n{}", width, height, max_value)?;

    // Build a custom thread pool with the specified number of threads
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

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
                            let u = (i as f32
                                + THREAD_RNG.with(|rng| rng.borrow_mut().gen::<f32>()))
                                / width as f32;
                            let v = (j as f32
                                + THREAD_RNG.with(|rng| rng.borrow_mut().gen::<f32>()))
                                / height as f32;

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

        writeln!(file, "{} {} {}", ir, ig, ib)?;
    }

    Ok(())
}
