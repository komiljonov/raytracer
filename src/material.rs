use crate::{hittable::HitRecord, random_in_unit_sphere, ray::Ray, vec3::Vec3};
use rand::Rng;

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
    attenuation: &mut Vec3,
    scattered: &mut Ray,
) -> bool {
    match material {
        Material::Lambertian { albedo } => {
            let target = rec.p + rec.normal + random_in_unit_sphere();
            *scattered = Ray::new(rec.p, target - rec.p);
            *attenuation = *albedo;
            true
        }
        Material::Metal { albedo, fuzz } => {
            let fuzz = fuzz.min(1.0);
            let reflected = reflect(&Vec3::unit_vector(&ray_in.direction()), &rec.normal);
            *scattered = Ray::new(rec.p, reflected + fuzz * random_in_unit_sphere());
            *attenuation = *albedo;
            Vec3::dot(&scattered.direction(), &rec.normal) > 0.0
        }
        Material::Dielectric { ref_idx } => {
            let (outward_normal, ni_over_nt, cosine) =
                if Vec3::dot(&ray_in.direction(), &rec.normal) > 0.0 {
                    (
                        -rec.normal,
                        *ref_idx,
                        ref_idx * Vec3::dot(&ray_in.direction(), &rec.normal)
                            / ray_in.direction().length(),
                    )
                } else {
                    (
                        rec.normal,
                        1.0 / *ref_idx,
                        -Vec3::dot(&ray_in.direction(), &rec.normal) / ray_in.direction().length(),
                    )
                };

            *attenuation = Vec3::new(1.0, 1.0, 1.0);

            let refracted = refract(&ray_in.direction(), &outward_normal, ni_over_nt);
            let reflect_prob = refracted.map(|_| schlick(cosine, *ref_idx)).unwrap_or(1.0);

            let mut rng = rand::thread_rng();
            *scattered = if rng.gen::<f32>() < reflect_prob {
                Ray::new(rec.p, reflect(&ray_in.direction(), &rec.normal))
            } else {
                Ray::new(rec.p, refracted.unwrap())
            };

            true
        }
    }
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

fn refract(v: &Vec3, n: &Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let uv = Vec3::unit_vector(v);
    let dt = Vec3::dot(&uv, n);
    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);

    if discriminant > 0.0 {
        Some(ni_over_nt * (uv - *n * dt) - *n * discriminant.sqrt())
    } else {
        None
    }
}

#[inline]
pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    *v - 2.0 * Vec3::dot(v, n) * *n
}
