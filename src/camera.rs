use std::ops::Range;
use cgmath;

use core::math::*;

/// This camera implementation always uses (0, 0, 1) as up-vector. Because the
/// direction vector must never be linear dependent to the up-vector, we have
/// limit the possible values theta can take. This is the "safe zone" that theta
/// must never be in; specifically '0...epsilon' and '(pi - epsilon)...pi'
const THETA_SAFE_EPSILON: Rad<f32> = Rad(0.02);


/// Saves the camera position and look direction as well as projection
/// parameters.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    direction: Vector3<f32>,
    pub projection: Projection,
}

impl Camera {
    /// Creates a new instance.
    ///
    /// `dir` mustn't be zero.
    pub fn new(pos: Point3<f32>, dir: Vector3<f32>, proj: Projection) -> Self {
        assert!(!dir.is_zero());

        Camera {
            position: pos,
            direction: dir.normalize(),
            projection: proj,
        }
    }

    /// Returns the spherical coordinates of the direction vector as
    /// `(theta, phi)`.
    pub fn spherical_coords(&self) -> (Rad<f32>, Rad<f32>) {
        let d = self.direction;
        (Rad(d.z.acos()), Rad(f32::atan2(d.y, d.x)))
    }

    /// Sets the normalized, given vector as new direction vector.
    pub fn look_in(&mut self, dir: Vector3<f32>) {
        self.direction = dir.normalize();

        let (theta, phi) = self.spherical_coords();
        let theta = Self::clamp_theta(theta);
        self.look_at_sphere(theta, phi);
    }

    /// Clamps theta into the allowed range
    fn clamp_theta(theta: Rad<f32>) -> Rad<f32> {
        use std::f32::consts::PI;

        clamp(theta, THETA_SAFE_EPSILON, Rad(PI) - THETA_SAFE_EPSILON)
    }

    /// Sets the direction vector from the given spherical coordinates
    pub fn look_at_sphere(&mut self, theta: Rad<f32>, phi: Rad<f32>) {
        let theta = Self::clamp_theta(theta);

        self.direction = Vector3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
    }

    /// Returns the current direction vector. It's guaranteed to be normalized.
    pub fn direction(&self) -> Vector3<f32> {
        self.direction
    }

    /// Returns the matrix representing the transformation into view space.
    pub fn view_transform(&self) -> Matrix4<f32> {
        Matrix4::look_at(
            self.position,
            self.position + self.direction,
            Vector3::new(0.0, 0.0, 1.0),
        )
    }

    /// Returns the matrix representing the transformation into projection
    /// space.
    pub fn proj_transform(&self) -> Matrix4<f32> {
        self.projection.transformation_matrix()
    }

    /// returns the top left and bottom right bounding box points of the 
    /// (projected) near plane
    // TODO: give new name that says what this thing actually does
    pub fn get_near_plane_bb(&self) -> Vec<Point3<f32>> {

        let mut vec = Vec::new();

        let frustum_height = self.projection.get_height_of_near_plane();
        let frustum_width = self.projection.get_width_of_near_plane();
        let inv_view_transform = self.view_transform().invert().unwrap();

        // top left
        // let top_left = Point3::new(-frustum_width/2.0, frustum_height/2.0, -self.projection.near_plane);
        // let proj_top_left = inv_view_transform * top_left.to_homogeneous();
        // vec.push(Point3::from_homogeneous(proj_top_left));

        // println!("Erstellter   Top-Left: {:?}", top_left);
        // println!("Projizierter Top-Left: {:?}", proj_top_left);
        // println!("Camera Position:       {:?}", self.position);
        // println!("Camera Direction:      {:?}", self.direction);
        // let proj_2_top_left = Point3::from_homogeneous(self.projection.transformation_matrix() * top_left.to_homogeneous());
        // println!("Projizierter2 Top-Left: {:?}", proj_2_top_left);
        // println!("frustum_width:  {}", frustum_width);
        // println!("frustum_height: {}", frustum_height);

        // bottom right
        // let bottom_right = Point3::new( frustum_width/2.0,  -frustum_height/2.0, -self.projection.near_plane);
        // let proj_bottom_right = inv_view_transform * bottom_right.to_homogeneous();
        // vec.push(Point3::from_homogeneous(proj_bottom_right));

        // 0
        let point = inv_view_transform * Point3::new(-2.0/6.0*frustum_width, 2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 1
        let point = inv_view_transform * Point3::new(0.0, 2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 2
        let point = inv_view_transform * Point3::new(2.0/6.0*frustum_width, 2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 3
        let point = inv_view_transform * Point3::new(-2.0/6.0*frustum_width, 0.0, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 4
        let point = inv_view_transform * Point3::new(0.0, 0.0, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 5
        let point = inv_view_transform * Point3::new(2.0/6.0*frustum_width, 0.0, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 6
        let point = inv_view_transform * Point3::new(-2.0/6.0*frustum_width, -2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 7
        let point = inv_view_transform * Point3::new(0.0, -2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // 8
        let point = inv_view_transform * Point3::new(2.0/6.0*frustum_width, -2.0/6.0*frustum_height, -self.projection.near_plane).to_homogeneous();
        vec.push(Point3::from_homogeneous(point));
        // let proj_2_bottom_right = Point3::from_homogeneous(self.projection.transformation_matrix() * bottom_right.to_homogeneous());
        // println!("Projizierter2 Bottom-right: {:?}", proj_2_bottom_right);
        
        vec

    }
}

/// Represents a specific projection that can be transformed by the selected
/// rendering method.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Projection {
    /// Field of view in the y direction (in range [0, π/2]).
    pub fov: Rad<f32>,

    /// Ratio between the width and the height. The field of view in the x
    /// direction is `self.fov * aspect_ratio`.
    aspect_ratio: f32,

    /// Everything closer to the camera than this won't be rendered.
    pub near_plane: f32,

    /// Everything farther away from the camera than this won't be rendered.
    pub far_plane: f32,
}

impl Projection {
    /// Creates a new projection from the given parameters.
    ///
    /// The `proj_range` reflects the near and far plane for projection. The
    /// aspect ratio is calculated from the given screen dimension, which has
    /// to be non-zero in both axes.
    pub fn new(fov: Rad<f32>, proj_range: Range<f32>, (w, h): (u32, u32)) -> Self {
        assert!(w > 0 && h > 0, "given screen dimension {:?} musn't be zero", (w, h));

        Projection {
            fov: fov,
            aspect_ratio: (w as f32) / (h as f32),
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

        self.aspect_ratio = (width as f32) / (height as f32);
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
    pub fn transformation_matrix(&self) -> Matrix4<f32> {
        use std::f32::consts::FRAC_PI_2;

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

    pub fn get_height_of_near_plane(&self) -> f32 {
        2.0 * self.near_plane * (self.fov * 0.5).tan()
    }

    pub fn get_width_of_near_plane(&self) -> f32 {
        self.get_height_of_near_plane() * self.aspect_ratio
    }


}
