use std::ops::Range;
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
        self.projection.transformation_matrix()
    }
}

/// Represents a specific projection that can be transformed by the selected
/// rendering method.
pub struct Projection {
    /// Field of view in the y direction (in range [0, π/2]).
    pub fov: Rad<f64>,

    /// Ratio between the width and the height. The field of view in the x
    /// direction is `self.fov * aspect_ratio`.
    aspect_ratio: f64,

    /// Everything closer to the camera than this won't be rendered.
    pub near_plane: f64,

    /// Everything farther away from the camera than this won't be rendered.
    pub far_plane: f64,
}

impl Projection {
    /// Creates a new projection from the given parameters.
    ///
    /// The `proj_range` reflects the near and far plane for projection. The
    /// aspect ratio is calculated from the given screen dimension, which has
    /// to be non-zero in both axes.
    pub fn new(fov: Rad<f64>, proj_range: Range<f64>, (w, h): (u32, u32)) -> Self {
        assert!(w > 0 && h > 0, "given screen dimension {:?} musn't be zero", (w, h));

        Projection {
            fov: fov,
            aspect_ratio: (w as f64) / (h as f64),
            near_plane: proj_range.start,
            far_plane: proj_range.end,
        }
    }

    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        assert!(
            width > 0 && height > 0,
            "given screen dimension {:?} musn't be zero",
            (width, height)
        );

        self.aspect_ratio = (width as f64) / (height as f64);
    }

    /// Returns the matrix representing the projection transformation specified
    /// by the parameters in this struct.
    ///
    /// The field of view needs to be in between 0 and π/2. Both, near and far
    /// plane have to be greater than zero and the near plane has to be less
    /// than the far plane.
    pub fn transformation_matrix(&self) -> Matrix4f {
        use std::f64::consts::FRAC_PI_2;

        assert!(self.fov > Rad(0.0));
        assert!(self.fov < Rad(FRAC_PI_2));
        assert!(self.far_plane > 0.0);
        assert!(self.near_plane > 0.0);
        assert!(self.far_plane > self.near_plane);

        perspective(
            self.fov,
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }
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
