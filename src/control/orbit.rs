use camera::{Camera, Projection};
use glium::glutin::{Event, VirtualKeyCode};
use core::math::*;
use std::f64::consts::PI;


pub struct Orbit {
    origin: Point3f,
    distance: f64,
    cam: Camera,
    // last_mouse_pos
}

impl Orbit {
    pub fn around(origin: Point3f, proj: Projection) -> Self {
        let init_dir = Vector3::new(1.0, 0.0, 0.0).normalize();
        let distance = 5.0;
        Orbit {
            origin: origin,
            distance: distance,
            cam: Camera::new(origin + -(init_dir * distance), init_dir, proj).unwrap(),
        }
    }

    fn turn_camera_phi(&mut self, amount: Rad<f64>) {
        let (theta, mut phi) = self.cam.spherical_coords();
        phi += amount;

        self.update_camera_from_theta_phi(theta, phi);
    }

    fn turn_camera_theta(&mut self, amount: Rad<f64>) {
        let (mut theta, phi) = self.cam.spherical_coords();
        theta += amount;
        if theta < Rad(0.05) {
            theta = Rad(0.05);
        }
        if theta > Rad(PI - 0.05) {
            theta = Rad(PI - 0.05);
        }

        self.update_camera_from_theta_phi(theta, phi);
    }

    fn update_camera_from_theta_phi(&mut self, theta: Rad<f64>, phi: Rad<f64>) {
        let eye_to_origin = Vector3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );

        self.cam.position = self.origin + self.distance * -eye_to_origin;
        self.cam.look_in(eye_to_origin);
    }

    pub fn camera(&self) -> &Camera {
        // debug!("{:?}", self.cam.spherical_coords());
        // debug!("{:?} || {:?}", self.cam.position, self.cam.direction);
        &self.cam
    }

    pub fn handle_event(&mut self, event: &Event) {
        // debug!("------------");
        use glium::glutin::ElementState;

        match *event {
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Left)) => {
                self.turn_camera_phi(Rad(-0.1));
            }
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Right)) => {
                self.turn_camera_phi(Rad(0.1));
            }
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Up)) => {
                self.turn_camera_theta(Rad(0.05));
            }
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Down)) => {
                self.turn_camera_theta(Rad(-0.05));
            }
            _ => ()
        }
    }
}
