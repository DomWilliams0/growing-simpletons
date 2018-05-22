use nalgebra::{zero, Isometry3, Point3, UnitQuaternion, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::{joint, object, world};

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
    fn register_object(
        &mut self,
        collider: object::ColliderHandle,
        shape: &body::Shape,
        colour: Colour,
    ) {
        let object = WorldObject::new(ObjectShape::from_body_shape(shape), colour);
        self.objects.push((collider, object));
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

    /*
    fn new_shape(&mut self, shape: &body::Cuboid) -> Self::RealisedHandle {
        let pos = Vector3::new(0.0, 3.0, 0.0); // TODO calculate?
        let obj = WorldObject::new(ObjectShape::Cuboid(shape.dims.into()), COLOUR_DEFAULT);
        self.world.add_body(pos, obj)
    }
    */

    fn new_shape(
        &mut self,
        shape: &body::Shape,
        parent: Self::RealisedHandle,
        parent_joint: &body::Joint,
    ) -> Self::RealisedHandle {
        // helper
        fn add_link<J: joint::Joint<Coord>>(
            world: &mut World,
            parent: object::BodyHandle,
            joint: J,
            shape: &ShapeHandle<Coord>,
        ) -> object::BodyHandle {
            let inertia = shape.inertia(1.0);
            let com = shape.center_of_mass();
            world
                .physics
                .add_multibody_link(parent, joint, zero(), zero(), inertia, com)
        }

        let body_shape = match shape {
            body::Shape::Cuboid(dims) => ShapeHandle::new(Cuboid::new((*dims).into())),
        };

        let pos = Vector3::new(0.0, 10.0, 0.0); // TODO position?
        let relative = {
            let q = UnitQuaternion::new(Vector3::new(0.0, -1.5, 0.5));
            let mut i = Isometry3::new(Vector3::new(0.0, 2.0, 0.5), zero());
            i.append_rotation_mut(&q);
            i
        };
        let link = match parent_joint.joint_type {
            body::JointType::Ground => add_link(
                self.world,
                parent,
                joint::FreeJoint::new(Isometry3::new(pos, zero())),
                &body_shape,
            ),
            body::JointType::Fixed => add_link(
                self.world,
                parent,
                joint::FixedJoint::new(relative),
                &body_shape,
            ),
        };

        let collider = self.world.physics.add_collider(
            COLLIDER_MARGIN,
            body_shape,
            link,
            Isometry3::identity(),
            object::Material::default(),
        );

        self.world.register_object(collider, shape, COLOUR_DEFAULT);
        link
    }

    fn root(&self) -> (Self::RealisedHandle, body::Joint) {
        (
            object::BodyHandle::ground(),
            body::Joint::new(body::JointType::Ground),
        )
    }
}

impl ObjectShape {
    fn from_body_shape(from: &body::Shape) -> Self {
        match from {
            body::Shape::Cuboid(dims) => ObjectShape::Cuboid((*dims).into()),
        }
    }
}

impl Into<Vector3<Coord>> for body::Dims {
    fn into(self) -> Vector3<Coord> {
        Vector3::new(self.x, self.y, self.z)
    }
}
