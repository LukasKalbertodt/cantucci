use std::ops::Range;
use core::math::*;
use cgmath;

/// This camera implementation always uses (0, 0, 1) as up-vector. Because the
/// direction vector must never be linear dependent to the up-vector, we have
/// limit the possible values theta can take. This is the "safe zone" that theta
/// must never be in; specifically '0...epsilon' and '(pi - epsilon)...pi'
const THETA_SAFE_EPSILON: Rad<f64> = Rad(0.02);


/// Saves the camera position and look direction as well as projection
/// parameters.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Camera {
    pub position: Point3<f64>,
    direction: Vector3<f64>,
    pub projection: Projection,
}

impl Camera {
    /// Creates a new instance.
    ///
    /// `dir` mustn't be zero.
    pub fn new(pos: Point3<f64>, dir: Vector3<f64>, proj: Projection) -> Self {
        assert!(!dir.is_zero());

        Camera {
            position: pos,
            direction: dir.normalize(),
            projection: proj,
        }
    }

    /// Returns the spherical coordinates of the direction vector as
    /// `(theta, phi)`.
    pub fn spherical_coords(&self) -> (Rad<f64>, Rad<f64>) {
        let d = self.direction;
        (Rad(d.z.acos()), Rad(f64::atan2(d.y, d.x)))
    }

    /// Sets the normalized, given vector as new direction vector.
    pub fn look_in(&mut self, dir: Vector3<f64>) {
        self.direction = dir.normalize();

        let (theta, phi) = self.spherical_coords();
        let theta = Self::clamp_theta(theta);
        self.look_at_sphere(theta, phi);
    }

    /// Clamps theta into the allowed range
    fn clamp_theta(theta: Rad<f64>) -> Rad<f64> {
        use std::f64::consts::PI;

        clamp(theta, THETA_SAFE_EPSILON, Rad(PI) - THETA_SAFE_EPSILON)
    }

    /// Sets the direction vector from the given spherical coordinates
    pub fn look_at_sphere(&mut self, theta: Rad<f64>, phi: Rad<f64>) {
        let theta = Self::clamp_theta(theta);

        self.direction = Vector3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
    }

    /// Returns the current direction vector. It's guaranteed to be normalized.
    pub fn direction(&self) -> Vector3<f64> {
        self.direction
    }

    /// Returns the matrix representing the transformation into view space.
    pub fn view_transform(&self) -> Matrix4<f64> {
        Matrix4::look_at(
            self.position,
            self.position + self.direction,
            Vector3::new(0.0, 0.0, 1.0),
        )
    }

    /// Returns the matrix representing the transformation into projection
    /// space.
    pub fn proj_transform(&self) -> Matrix4<f64> {
        self.projection.transformation_matrix()
    }
}

/// Represents a specific projection that can be transformed by the selected
/// rendering method.
#[derive(Clone, Copy, PartialEq, Debug)]
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

    /// Calculates and sets the aspect ratio from the screen dimension. The
    /// dimension has to be non-zero in both directions.
    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        assert!(
            width > 0 && height > 0,
            "given screen dimension {:?} musn't be zero",
            (width, height)
        );

        self.aspect_ratio = (width as f64) / (height as f64);
    }

    /// Sets the aspect ratio to the aspect ratio of `other`
    pub fn set_aspect_ratio_from(&mut self, other: &Self) {
        self.aspect_ratio = other.aspect_ratio;
    }

    /// Returns the matrix representing the projection transformation specified
    /// by the parameters in this struct.
    ///
    /// The field of view needs to be in between 0 and π/2. Both, near and far
    /// plane have to be greater than zero and the near plane has to be less
    /// than the far plane.
    pub fn transformation_matrix(&self) -> Matrix4<f64> {
        use std::f64::consts::FRAC_PI_2;

        assert!(self.fov > Rad(0.0));
        assert!(self.fov < Rad(FRAC_PI_2));
        assert!(self.far_plane > 0.0);
        assert!(self.near_plane > 0.0);
        assert!(self.far_plane > self.near_plane);

        cgmath::perspective(
            self.fov,
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }
}
