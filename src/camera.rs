use crate::{ray::Ray, vec3::Vec3};
use rand::prelude::*;

#[derive(Clone, Copy)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lens_radius: f32,
    u: Vec3,
    v: Vec3,
    // w: Vec3,
}

impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        vup: Vec3,
        fvof: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Camera {
        // let mut u = Vec3::default();
        // let mut v = Vec3::default();
        // let mut w = Vec3::default();

        let lens_radius = aperture / 2.0;

        let theta = fvof * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();

        let half_width = aspect * half_height;

        let origin = look_from;
        let w = Vec3::unit_vector(&(look_from - look_at));
        let u = Vec3::unit_vector(&Vec3::cross(&vup, &w));
        let v = Vec3::cross(&w, &u);

        Camera {
            lower_left_corner: origin
                - half_width * focus_dist * u
                - half_height * focus_dist * v
                - focus_dist * w,
            horizontal: 2.0 * half_width * focus_dist * u,
            vertical: 2.0 * half_height * focus_dist * v,
            origin,
            lens_radius,
            u,
            v,
            // w,
        }
    }

    pub fn get_ray(self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.u * rd.x() + self.v * rd.y();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

fn random_in_unit_disk() -> Vec3 {
    let mut rng = rand::thread_rng();

    loop {
        let p = 2.0 * Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), 0.0) - Vec3::new(1.0, 1.0, 0.0);

        // The dot product should be less than 1.0 to ensure the point is inside the unit disk
        if Vec3::dot(&p, &p) < 1.0 {
            return p;
        }
    }
}