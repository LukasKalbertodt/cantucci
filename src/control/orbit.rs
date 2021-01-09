use glium::glutin::Event;

use camera::{Camera, Projection};
use math::*;
use shape::Shape;
use event::{EventHandler, EventResponse};
use super::CamControl;

/// This describes the maximum speed (per seconds) in which theta can change.
/// Phi can change twice as fast, because the range is twice as big.
const MAX_TURN_SPEED: Rad<f32> = Rad(1.0);

/// This describes how slowly the maximum speed is reached. Precisely, it's
/// the time (in seconds) it takes to accelerate from speed 'x' to speed
/// '(MAX_TURN_SPEED + x) / 2'.
const TURN_DELAY: f32 = 0.05;

/// Describes how quickly the user can zoom in and out. Precisely,
/// '2.pow(ZOOM_SPEED)' describes the factor by which the distance can grow
/// or shrink each second. When ZOOM_SPEED=1.0 (default), then the distance
/// doubles or shrinks every second when zooming.
const ZOOM_SPEED: f32 = 1.0;

/// Offers orbital control around a fixed origin point.
pub struct Orbit {
    origin: Point3<f32>,
    cam: Camera,

    // These four values are used for smooth rotations. The `speed` values
    // hold the current speed (in rad/s) by which the spherical coordinates
    // change the next frame. The `accel` values are either 1, 0 or -1 and
    // described the acceleration by which the speed changes.
    theta_speed: Rad<f32>,
    theta_accel: Rad<f32>,
    phi_speed: Rad<f32>,
    phi_accel: Rad<f32>,

    // This is either `ZOOM_SPEED`, 0 or `-ZOOM_SPEED`. See its documentation
    // for more information.
    zoom_speed: f32,
}

impl Orbit {
    /// Creates an orbital control around the given point with the given
    /// projection.
    pub fn around(origin: Point3<f32>, proj: Projection) -> Self {
        let init_dir = Vector3::new(1.0, 0.0, 0.0).normalize();
        let distance = 3.0;

        Orbit {
            origin: origin,
            cam: Camera::new(origin + -(init_dir * distance), init_dir, proj),
            theta_speed: Rad(0.0),
            theta_accel: Rad(0.0),
            phi_speed: Rad(0.0),
            phi_accel: Rad(0.0),
            zoom_speed: 0.0,
        }
    }

    fn distance(&self) -> f32 {
        (self.cam.position - self.origin).magnitude()
    }

    fn update_distance(&mut self, distance: f32) {
        self.cam.position = self.origin + distance * -self.cam.direction();
    }
}

impl CamControl for Orbit {
    fn camera(&self) -> Camera {
        self.cam
    }

    fn projection_mut(&mut self) -> &mut Projection {
        &mut self.cam.projection
    }

    fn update(&mut self, delta: f32, _: &dyn Shape) {
        // Update the theta and phi turning speeds
        self.theta_speed = lerp(
            self.theta_speed,
            self.theta_accel * MAX_TURN_SPEED.0,
            1.0 - 2.0f32.powf(-delta / TURN_DELAY),
        );
        self.phi_speed = lerp(
            self.phi_speed,
            self.phi_accel * MAX_TURN_SPEED.0 * 2.0,
            1.0 - 2.0f32.powf(-delta / TURN_DELAY),
        );

        // Update actual turning position with those calculates speeds and
        // update the camera accordingly.
        let (mut theta, mut phi) = self.cam.spherical_coords();
        theta += self.theta_speed * delta;
        phi += self.phi_speed * delta;

        self.cam.look_at_sphere(theta, phi);
        self.cam.position = self.origin + self.distance() * -self.cam.direction();

        // Update distance from origin
        let rate_of_change = self.zoom_speed * delta;
        let new_distance = self.distance() * 2.0f32.powf(rate_of_change);
        self.update_distance(new_distance);
    }

    fn as_event_handler(&mut self) -> &mut dyn EventHandler {
        self
    }

    fn match_view(&mut self, other: &Camera) {
        let view_dir = self.origin - other.position;
        self.cam.look_in(view_dir);
        self.cam.position = other.position;
    }
}

impl EventHandler for Orbit {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        // We are only interested in keyboard input ...
        if let Event::KeyboardInput(state, _, Some(key)) = *e {
            use glium::glutin::ElementState::*;
            use glium::glutin::VirtualKeyCode as Vkc;

            match (state, key) {
                // Update accelerations for turning
                (Pressed,  Vkc::Up) | (Released, Vkc::Down) if self.theta_accel <= Rad(0.0)
                    => self.theta_accel += Rad(1.0),
                (Released, Vkc::Up) | (Pressed,  Vkc::Down) if self.theta_accel >= Rad(0.0)
                    => self.theta_accel -= Rad(1.0),
                (Pressed, Vkc::Right) | (Released,  Vkc::Left) if self.phi_accel <= Rad(0.0)
                    => self.phi_accel += Rad(1.0),
                (Released,  Vkc::Right) | (Pressed, Vkc::Left) if self.phi_accel >= Rad(0.0)
                    => self.phi_accel -= Rad(1.0),

                // Update zoom speed
                (Released, Vkc::Add) | (Pressed, Vkc::Subtract) |
                (Released, Vkc::Equals) | (Pressed, Vkc::Minus) if self.zoom_speed <= 0.0
                    => self.zoom_speed += ZOOM_SPEED,
                (Pressed, Vkc::Add) | (Released, Vkc::Subtract) |
                (Pressed, Vkc::Equals) | (Released, Vkc::Minus) if self.zoom_speed >= 0.0
                    => self.zoom_speed -= ZOOM_SPEED,

                _ => return EventResponse::NotHandled,
            }

            EventResponse::Break
        } else {
            EventResponse::NotHandled
        }
    }
}
