extern crate kiss3d;
extern crate nalgebra;
extern crate nphysics3d;
extern crate shapes;

use kiss3d::{camera, light, scene, window};
use nalgebra::{Point3, Vector3};
use nphysics3d::object::ColliderHandle;
use std::collections::HashMap;

use shapes::{body, physics, tree};

// TODO tidy this up with a struct
fn new_node(window: &mut window::Window, object: &physics::ObjectShape) -> scene::SceneNode {
    match object {
        physics::ObjectShape::Cuboid(dims) => {
            window.add_cube(dims.x * 2.0, dims.y * 2.0, dims.z * 2.0)
        }
        physics::ObjectShape::Plane(pos, norm, size) => {
            let mut plane = window.add_quad(size * 2.0, size * 2.0, 100, 100);
            let up = if norm.z == 0.0 && norm.y == 0.0 {
                Vector3::z()
            } else {
                Vector3::x()
            };
            plane.reorient(pos, &(*pos + *norm), &up);
            plane
        }
    }
}

fn main() {
    let mut window = window::Window::new("Shapes renderer");
    let mut world = physics::World::default();

    let tree = {
        let mut t = tree::BodyTree::default();
        t.set_root(tree::Node::Shape(body::Cuboid::new(body::Dims::new(
            1.0, 3.0, 0.1,
        ))));
        t
    };

    {
        let mut r = physics::PhysicalRealiser::new(&mut world);
        tree.recurse(&mut r);
    }

    // add objects from physics world to renderer
    let mut objects = HashMap::<ColliderHandle, scene::SceneNode>::new();
    for (handle, _collider, obj) in world.objects() {
        let node = new_node(&mut window, &obj.shape);
        objects.insert(handle, node);
    }

    let mut camera =
        camera::ArcBall::new(Point3::new(30.0, 30.0, 30.0), Point3::new(0.0, 0.0, 0.0));

    window.set_light(light::Light::StickToCamera);

    while window.render_with_camera(&mut camera) {
        // step world
        world.tick();

        // update scene
        for (handle, obj, collider, body) in world.colliders() {
            let mut node = objects.get_mut(&handle).unwrap();
            let active = body.is_active();
            let color = obj.colour;
            if active {
                node.set_local_transformation(*collider.position());
                node.set_color(color.r, color.g, color.b);
            } else {
                node.set_color(color.r * 0.25, color.g * 0.25, color.b * 0.25);
            }
        }
    }
}
