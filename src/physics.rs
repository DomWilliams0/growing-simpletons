use nalgebra::{zero, Isometry3, Point3, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::{object, world};

use body;
use tree::TreeRealiser;
use Coord;

const COLLIDER_MARGIN: f32 = 0.01;

#[derive(Copy, Clone)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

const COLOUR_GROUND: Colour = Colour {
    r: 0.1,
    g: 0.1,
    b: 0.1,
};
const COLOUR_DEFAULT: Colour = Colour {
    r: 0.6,
    g: 0.8,
    b: 0.2,
};

pub enum ObjectShape {
    Cuboid(Vector3<Coord>),
    Plane(Point3<Coord>, Vector3<Coord>, Coord),
}

pub struct WorldObject {
    pub shape: ObjectShape,
    pub colour: Colour,
}

pub struct World {
    physics: world::World<Coord>,
    objects: Vec<(object::ColliderHandle, WorldObject)>,
}

impl WorldObject {
    fn new(shape: ObjectShape, colour: Colour) -> Self {
        Self { shape, colour }
    }
}

impl Default for World {
    fn default() -> Self {
        let mut world = world::World::new();
        world.set_gravity(Vector3::new(0.0, -9.81, 0.0));
        let material = object::Material::default();

        let ground_size = 50.0;
        let ground_shape =
            ShapeHandle::new(Cuboid::new(Vector3::repeat(ground_size - COLLIDER_MARGIN)));
        let ground_pos = Isometry3::new(Vector3::y() * -ground_size, zero());

        let ground = world.add_collider(
            COLLIDER_MARGIN,
            ground_shape,
            object::BodyHandle::ground(),
            ground_pos,
            material,
        );

        let ground_obj = WorldObject::new(
            ObjectShape::Plane(Point3::new(0.0, 0.0, 0.0), Vector3::y(), ground_size),
            COLOUR_GROUND,
        );
        let objects = vec![(ground, ground_obj)];

        Self {
            physics: world,
            objects,
        }
    }
}

impl World {
    fn add_body(&mut self, pos: Vector3<Coord>, object: WorldObject) -> object::BodyHandle {
        let shape = match object.shape {
            ObjectShape::Cuboid(dims) => ShapeHandle::new(Cuboid::new(dims)),
            _ => unimplemented!(),
        };
        let inertia = shape.inertia(1.0);
        let com = shape.center_of_mass();
        let pos = Isometry3::new(pos, zero());

        let handle = self.physics.add_rigid_body(pos, inertia, com);
        let collider = self.physics.add_collider(
            COLLIDER_MARGIN,
            shape,
            handle,
            Isometry3::identity(),
            object::Material::default(),
        );

        self.objects.push((collider, object));
        handle
    }

    pub fn add_test_bodies(&mut self) {
        let dims = Vector3::new(2.0, 1.0, 0.5);
        self.add_body(
            Vector3::new(0.0, 2.0, 0.0),
            WorldObject::new(ObjectShape::Cuboid(dims), COLOUR_DEFAULT),
        );
        self.add_body(
            Vector3::new(0.2, 4.0, 1.0),
            WorldObject::new(ObjectShape::Cuboid(dims), COLOUR_DEFAULT),
        );
    }

    pub fn objects(
        &self,
    ) -> impl Iterator<
        Item = (
            object::ColliderHandle,
            &object::Collider<Coord>,
            &WorldObject,
        ),
    > {
        self.objects
            .iter()
            .filter_map(move |(ch, o)| self.physics.collider(*ch).map(|coll| (*ch, coll, o)))
    }

    pub fn colliders(
        &self,
    ) -> impl Iterator<
        Item = (
            object::ColliderHandle,
            &WorldObject,
            &object::Collider<Coord>,
            object::Body<Coord>,
        ),
    > {
        self.objects.iter().filter_map(move |(ch, o)| {
            self.physics
                .collider(*ch)
                .map(|c| (*ch, o, c, self.physics.body(c.data().body())))
        })
    }

    pub fn tick(&mut self) {
        self.physics.step();
    }
}

pub struct PhysicalRealiser<'w> {
    world: &'w mut World,
}

impl<'w> PhysicalRealiser<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self { world }
    }
}

impl<'w> TreeRealiser for PhysicalRealiser<'w> {
    type RealisedHandle = object::BodyHandle;

    fn new_shape(&mut self, shape: &body::Cuboid) -> Self::RealisedHandle {
        let pos = Vector3::new(0.0, 3.0, 0.0); // TODO calculate?
        let obj = WorldObject::new(ObjectShape::Cuboid(shape.dims.into()), COLOUR_DEFAULT);
        self.world.add_body(pos, obj)
    }

    fn new_joint(
        &mut self,
        shape: &body::Joint,
        children: &[Self::RealisedHandle],
    ) -> Self::RealisedHandle {
        unimplemented!();
    }
}

impl Into<Vector3<Coord>> for body::Dims {
    fn into(self) -> Vector3<Coord> {
        Vector3::new(self.x, self.y, self.z)
    }
}
