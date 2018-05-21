use nalgebra::{zero, Isometry3, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::{object, world};

use Coord;

const COLLIDER_MARGIN: f32 = 0.01;

struct World {
    physics: world::World<Coord>,
}

impl World {
    fn new() -> Self {
        let mut world = world::World::new();
        let material = object::Material::default();

        let ground_size = 50.0;
        let ground_shape =
            ShapeHandle::new(Cuboid::new(Vector3::repeat(ground_size - COLLIDER_MARGIN)));
        let ground_pos = Isometry3::new(Vector3::y() * -ground_size, zero());

        world.add_collider(
            COLLIDER_MARGIN,
            ground_shape,
            object::BodyHandle::ground(),
            ground_pos,
            material,
        );

        Self { physics: world }
    }
}
