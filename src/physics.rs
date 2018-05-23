use nalgebra::{zero, Isometry3, Point3, UnitQuaternion, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::joint::{FixedJoint, FreeJoint, Joint};
use nphysics3d::object::{Body, BodyHandle, Collider, ColliderHandle, Material};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::world;
use rand::{self, Rng};

use body_tree::tree::TreeRealiser;
use body_tree::{body, Coord};

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
    objects: Vec<(ColliderHandle, WorldObject)>,
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
        let material = Material::default();

        let ground_size = 50.0;
        let ground_shape =
            ShapeHandle::new(Cuboid::new(Vector3::repeat(ground_size - COLLIDER_MARGIN)));
        let ground_pos = Isometry3::new(Vector3::y() * -ground_size, zero());

        let ground = world.add_collider(
            COLLIDER_MARGIN,
            ground_shape,
            BodyHandle::ground(),
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
        collider: ColliderHandle,
        def: &body::ShapeDefinition,
        colour: Colour,
    ) {
        let object = WorldObject::new(ObjectShape::from_def(def), colour);
        self.objects.push((collider, object));
    }

    pub fn objects(
        &self,
    ) -> impl Iterator<Item = (ColliderHandle, &Collider<Coord>, &WorldObject)> {
        self.objects
            .iter()
            .filter_map(move |(ch, o)| self.physics.collider(*ch).map(|coll| (*ch, coll, o)))
    }

    pub fn colliders(
        &self,
    ) -> impl Iterator<Item = (ColliderHandle, &WorldObject, &Collider<Coord>, Body<Coord>)> {
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
    pub next_spawn_pos: Vector3<Coord>,
    random: rand::ThreadRng,
}

impl<'w> PhysicalRealiser<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            next_spawn_pos: Vector3::new(0.0, 5.0, 0.0),
            random: rand::thread_rng(),
        }
    }
}

impl<'w> TreeRealiser for PhysicalRealiser<'w> {
    type RealisedHandle = BodyHandle;

    fn new_shape(
        &mut self,
        shape_def: &body::ShapeDefinition,
        parent: Self::RealisedHandle,
        parent_joint: &body::Joint,
    ) -> Self::RealisedHandle {
        // helper
        fn add_link<J: Joint<Coord>>(
            world: &mut World,
            parent: BodyHandle,
            joint: J,
            shape: &ShapeHandle<Coord>,
        ) -> BodyHandle {
            let inertia = shape.inertia(1.0);
            let com = shape.center_of_mass();
            world
                .physics
                .add_multibody_link(parent, joint, zero(), zero(), inertia, com)
        }

        // get parent global position
        let parent_pos = {
            match self.world.physics.multibody_link(parent) {
                Some(link) => link.position(),
                None => Isometry3::new(self.next_spawn_pos, zero()), // spawn position of full entity
            }
        };

        // get parameters from shape definition
        let (body_shape, rel_pos, rotation) = match shape_def {
            body::ShapeDefinition::Cuboid(dims, pos, rot) => {
                (ShapeHandle::new(Cuboid::new((*dims).into())), pos, rot)
            }
        };

        // parse parameters
        let joint_params = {
            let mut shift = Isometry3::new((*rel_pos).into(), zero());
            shift.append_rotation_mut(&UnitQuaternion::new((*rotation).into()));
            shift
        };
        let link = match parent_joint.joint_type {
            body::JointType::Ground => {
                add_link(self.world, parent, FreeJoint::new(parent_pos), &body_shape)
            }
            body::JointType::Fixed => add_link(
                self.world,
                parent,
                FixedJoint::new(joint_params),
                &body_shape,
            ),
        };

        let collider = self.world.physics.add_collider(
            COLLIDER_MARGIN,
            body_shape,
            link,
            Isometry3::identity(),
            Material::default(),
        );

        self.world
            .register_object(collider, shape_def, Colour::random(&mut self.random));
        link
    }

    fn root(&self) -> (Self::RealisedHandle, body::Joint) {
        (
            BodyHandle::ground(),
            body::Joint::new(body::JointType::Ground),
        )
    }
}

impl ObjectShape {
    fn from_def(def: &body::ShapeDefinition) -> Self {
        match def {
            body::ShapeDefinition::Cuboid(dims, ..) => ObjectShape::Cuboid((*dims).into()),
        }
    }
}

impl Colour {
    fn random(random: &mut rand::ThreadRng) -> Self {
        Self {
            r: random.gen(),
            g: random.gen(),
            b: random.gen(),
        }
    }
}
