extern crate glfw;
extern crate kiss3d;
extern crate nalgebra;
extern crate nphysics3d;
extern crate shapes;

use glfw::{Action, Key, WindowEvent};
use kiss3d::{camera, light, scene, window};
use nalgebra::{Point3, Vector3};
use nphysics3d::object::ColliderHandle;
use std::collections::HashMap;
use std::env;

use shapes::body_tree::serialise;
use shapes::physics;

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

struct Renderer {
    window: window::Window,
    world: physics::World,
    objects: HashMap<ColliderHandle, scene::SceneNode>,
}

impl Renderer {
    fn new() -> Self {
        Self {
            window: window::Window::new("Simpletons"),
            world: physics::World::default(),
            objects: HashMap::new(),
        }
    }

    fn reset_population<P: Into<::std::path::PathBuf>>(&mut self, path: P) {
        let pop = serialise::load(path);
        let padding = 10.0;

        // clear old population
        {
            self.world.clear();
            for mut node in self.objects.values_mut() {
                self.window.remove(&mut node);
            }
            self.objects.clear();
        }
        // add new population
        {
            let mut r = physics::PhysicalRealiser::new(&mut self.world);
            for (i, tree) in pop.iter().enumerate() {
                let i = i as shapes::body_tree::Coord;
                r.next_spawn_pos = Vector3::new(i * padding, 5.0, 0.0);
                tree.realise(&mut r);
            }
        }

        // add objects from physics world to renderer
        for (handle, _collider, obj) in self.world.objects() {
            let node = new_node(&mut self.window, &obj.shape);
            self.objects.insert(handle, node);
        }
    }

    fn start(&mut self) {
        let path = env::args()
            .nth(1)
            .unwrap_or_else(|| "./population.json".to_owned());

        self.reset_population(&path);

        let mut camera =
            camera::ArcBall::new(Point3::new(30.0, 30.0, 30.0), Point3::new(0.0, 0.0, 0.0));

        self.window.set_light(light::Light::StickToCamera);

        while self.window.render_with_camera(&mut camera) {
            // step world
            self.world.tick();

            // update scene
            for (handle, obj, collider, body) in self.world.colliders() {
                let mut node = match self.objects.get_mut(&handle) {
                    Some(n) => n,
                    None => continue,
                };
                let active = body.is_active();
                let color = obj.colour;
                if active {
                    node.set_local_transformation(*collider.position());
                    node.set_color(color.r, color.g, color.b);
                } else {
                    node.set_color(color.r * 0.25, color.g * 0.25, color.b * 0.25);
                }
            }

            // keyboard
            for mut e in self.window.events().iter() {
                if let WindowEvent::Key(Key::Enter, _, Action::Press, _) = e.value {
                    self.reset_population(&path);
                }
            }
        }
    }
}

fn main() {
    Renderer::new().start();
}
