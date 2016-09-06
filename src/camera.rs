use errors::*;
// use cgmath::*;
use core::math::*;
use std::error::Error as StdError;

pub struct Camera {
    pub position: Point3f,
    pub direction: Vector3f,
    projection: Projection,
}

impl Camera {
    pub fn new(pos: Point3f, dir: Vector3f, proj: Projection)
        -> Result<Self, Box<StdError>>
    {
        // if params.fov.0 >= ::std::f64::consts::FRAC_PI_2 {
        //     return Err("field of view has to be less than 180° = π/2".into());
        //     // return Err(ParamError::InvalidFov(params.fov).into());
        // }

        // TODO: parameter checking

        Ok(Camera {
            position: pos,
            direction: dir.normalize(),
            projection: proj,
        })
    }

    /// Returns the spherical coordinates of the direction vector as
    /// `(theta, phi)`.
    pub fn spherical_coords(&self) -> (Rad<f64>, Rad<f64>) {
        let d = self.direction;
        (Rad(d.z.acos()), Rad(f64::atan2(d.y, d.x)))
    }

    pub fn look_in(&mut self, dir: Vector3f) {
        self.direction = dir.normalize();
    }

    pub fn view_transform(&self) -> Matrix4f {
        Matrix4::look_at(
            self.position,
            self.position + self.direction,
            Vector3f::new(0.0, 0.0, 1.0),
        )
    }

    pub fn proj_transform(&self) -> Matrix4f {
        perspective(
            self.projection.fov,
            self.projection.aspect_ratio,
            self.projection.near_plane,
            self.projection.far_plane,
        )
    }
}

pub struct Projection {
    pub fov: Rad<f64>,
    pub aspect_ratio: f64,
    pub near_plane: f64,
    pub far_plane: f64,
}


// pub struct CameraParams {
//     pub position: Point3f,
//     pub direction: Vector3f,
//     pub fov: Rad<f64>,
//     pub aspect_ratio: f64,
//     pub near_plane: f64,
//     pub far_plane: f64,
// }

// quick_error! {
//     #[derive(Debug)]
//     pub enum ParamError {
//         InvalidFov(fov: Rad<f64>) {
//             description("fov has to be less than 180° = π/2 and greater than 0")
//             display("given fov ({}) is greater than 180° = π/2 or less than 0", fov.0)
//         }
//     }
// }
