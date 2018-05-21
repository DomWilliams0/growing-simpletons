extern crate kiss3d;
extern crate nalgebra;
extern crate shapes;

use kiss3d::{camera, light, window};
use nalgebra::{Point3, UnitQuaternion, Vector3};

use shapes::physics;

fn main() {
    let mut window = window::Window::new("Shapes renderer");
    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.set_color(1.0, 0.0, 0.0);

    let mut camera =
        camera::ArcBall::new(Point3::new(10.0, 10.0, 10.0), Point3::new(0.0, 0.0, 0.0));

    window.set_light(light::Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render_with_camera(&mut camera) {
        c.prepend_to_local_rotation(&rot);
    }
}
