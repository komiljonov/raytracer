use std::mem::Discriminant;

use crate::{hittable::HitRecord, random_in_unit_sphere, ray::Ray, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { ref_idx: f32 },
}

impl Default for Material {
    #[inline]
    fn default() -> Self {
        Material::Lambertian {
            albedo: Vec3::default(),
        }
    }
}

pub fn scatter(
    material: &Material,
    ray_in: &Ray,
    rec: &HitRecord,
    attentuation: &mut Vec3,
    scattered: &mut Ray,
) -> bool {
    match material {
        &Material::Lambertian { albedo } => {
            let target = rec.p + rec.normal + random_in_unit_sphere();

            *scattered = Ray::new(rec.p, target - rec.p);

            *attentuation = albedo;
            return true;
        }
        Material::Metal { albedo, fuzz } => {
            let mut f = 1.0;

            if *fuzz < 1.0 {
                f = *fuzz;
            }

            let reflected = reflect(&Vec3::unit_vector(&ray_in.direction()), &rec.normal);
            *scattered = Ray::new(rec.p, reflected + f * random_in_unit_sphere());
            *attentuation = *albedo;

            return Vec3::dot(&scattered.direction(), &rec.normal) > 0.0;
        }
        Material::Dielectric { ref_idx } => {
            let mut outward_normal = Vec3::default();

            let _reflected = reflect(&ray_in.direction(), &rec.normal);

            let mut ni_over_nt = 0.0;
            let _attentuation = Vec3::new(1.0, 1.0, 0.0);
            let mut refracted = Vec3::default();

            if Vec3::dot(&ray_in.direction(), &rec.normal) > 0f32 {
                outward_normal = -rec.normal;

                ni_over_nt = *ref_idx;
            } else {
                outward_normal = rec.normal;
                ni_over_nt = 1.0 / ref_idx;
            }

            if refract(
                &ray_in.direction(),
                &outward_normal,
                ni_over_nt,
                &mut refracted,
            ) {
                *scattered = Ray::new(rec.p, refracted);
            } else {
                *scattered = Ray::new(rec.p, refracted);
                return false;
            }

            return true;
        }
    }
}

fn refract(v: &Vec3, n: &Vec3, ni_over_nt: f32, refracted: &mut Vec3) -> bool {
    let uv = Vec3::unit_vector(v);

    let dt = Vec3::dot(&uv, n);

    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);

    if discriminant > 0.0 {
        *refracted = ni_over_nt * (uv - *n * dt) - *n * discriminant.sqrt();
        return true;
    } else {
        return false;
    }
}

#[inline]
pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    *v - 2.0 * Vec3::dot(v, n) * *n
}
