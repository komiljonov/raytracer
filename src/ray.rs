use crate::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    a: Vec3,
    b: Vec3,
}

impl Ray {
    // Constructor method
    pub fn new(a: Vec3, b: Vec3) -> Ray {
        Ray { a, b }
    }

    // Returns the origin of the ray
    pub fn origin(self) -> Vec3 {
        self.a
    }

    // Returns the direction of the ray
    pub fn direction(self) -> Vec3 {
        self.b
    }

    // Computes the point at a given parameter t
    pub fn point_at_parameter(self, t: f32) -> Vec3 {
        self.a + self.b * t
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_ray_origin() {}

    #[test]
    fn test_ray_direction() {}

    #[test]
    fn test_ray_point_at_parameter() {}
}
