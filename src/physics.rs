use nalgebra::{zero, Isometry3, Point3, UnitQuaternion, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::joint::{FixedJoint, FreeJoint, Joint};
use nphysics3d::object::{Body, BodyHandle, Collider, ColliderHandle, Material};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::world;
use rand::{self, Rng};

use body_tree::tree::TreeRealiser;
use body_tree::{body::def, Coord};

const COLLIDER_MARGIN: Coord = 0.01;

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum ObjectShape {
    Cuboid(Vector3<Coord>),
    Plane(Point3<Coord>, Vector3<Coord>, Coord),
}

#[derive(Debug)]
pub struct WorldObject {
    pub shape: ObjectShape,
    pub colour: Colour,
}

pub struct World {
    physics: world::World<Coord>,
    objects: Vec<(ColliderHandle, WorldObject)>,
    ground_collider: Option<ColliderHandle>,
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
        Self {
            physics: world,
            objects: Vec::new(),
            ground_collider: None,
        }
    }
}

impl World {
    fn register_object(
        &mut self,
        collider: ColliderHandle,
        def: &def::ShapeDefinition,
        colour: Colour,
    ) {
        let object = WorldObject::new(ObjectShape::from_def(def), colour);
        self.register_created_object(collider, object);
    }

    fn register_created_object(&mut self, collider: ColliderHandle, object: WorldObject) {
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

    fn shape(&self, ch: ColliderHandle) -> Option<ObjectShape> {
        self.objects
            .iter()
            .find(|(x, _)| x.uid() == ch.uid())
            .map(|tup| tup.1.shape)
    }

    pub fn tick(&mut self) {
        self.physics.step();
    }

    fn add_ground(&mut self) {
        let material = Material::default();
        let ground_size = 50.0;
        let ground_shape =
            ShapeHandle::new(Cuboid::new(Vector3::repeat(ground_size - COLLIDER_MARGIN)));
        let ground_pos = Isometry3::new(Vector3::y() * -ground_size, zero());

        let ground = self.physics.add_collider(
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
        self.ground_collider = Some(ground);
        self.register_created_object(ground, ground_obj);
    }

    pub fn clear(&mut self) {
        for (ch, _) in &self.objects {
            let bh = {
                match self.physics.collider(*ch) {
                    Some(col) => col.data().body(),
                    None => continue,
                }
            };
            self.physics.remove_bodies(&[bh]);
        }
        self.ground_collider = None;
        self.objects.clear();

        // rather awful
        self.add_ground();
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
            next_spawn_pos: Vector3::new(0.0, 10.0, 0.0),
            random: rand::thread_rng(),
        }
    }
}

fn position_on_face(
    face_index: u32,
    face_coords: (Coord, Coord),
    my_dims: &Vector3<Coord>,
    parent_dims: &Vector3<Coord>,
) -> Vector3<Coord> {
    let (f1, f2) = face_coords;

    match face_index {
        // top/bottom
        0 => Vector3::new(
            f1 * parent_dims.x,
            -(parent_dims.y / 2.0 + my_dims.y),
            f2 * parent_dims.z,
        ),
        1 => Vector3::new(
            f1 * parent_dims.x,
            parent_dims.y / 2.0 + my_dims.y,
            f2 * parent_dims.z,
        ),

        // back/front
        2 => Vector3::new(
            -(parent_dims.x / 2.0 + my_dims.x),
            f1 * parent_dims.y,
            f2 * parent_dims.z,
        ),
        3 => Vector3::new(
            parent_dims.x / 2.0 + my_dims.x,
            f1 * parent_dims.y,
            f2 * parent_dims.z,
        ),

        // left/right
        4 => Vector3::new(
            f1 * parent_dims.x,
            f2 * parent_dims.y,
            parent_dims.z / 2.0 + my_dims.z,
        ),
        5 => Vector3::new(
            f1 * parent_dims.x,
            f2 * parent_dims.y,
            -(parent_dims.z / 2.0 + my_dims.z),
        ),

        _ => panic!(format!("bad face index {}", face_index)),
    }
}

fn shape_from_def(
    definition: &def::ShapeDefinition,
    parent_shape: &ObjectShape,
) -> (ShapeHandle<Coord>, Vector3<Coord>, Vector3<Coord>) {
    match definition {
        def::ShapeDefinition::Cuboid { dims, pos, rot } => {
            let (w, h, d) = dims.components_scaled();
            let (face_idx, face_1, face_2) = pos;
            let (rx, ry, rz) = rot.components_scaled();
            let cuboid = Cuboid::new(Vector3::new(w, h, d));

            let offset = match parent_shape {
                ObjectShape::Plane(_, _, _) => Vector3::identity(),
                ObjectShape::Cuboid(parent_dims) => {
                    use body_tree::body::def::RangedParam; // ooer

                    const FACE_COUNT: f64 = 6.0;
                    let face = (face_idx.get() * FACE_COUNT).min(FACE_COUNT - 1.0) as u32;
                    position_on_face(
                        face,
                        (face_1.get_scaled(), face_2.get_scaled()),
                        &Vector3::new(w, h, d),
                        parent_dims,
                    )
                }
            };

            (ShapeHandle::new(cuboid), offset, Vector3::new(rx, ry, rz))
        }
    }
}

impl<'w> TreeRealiser for PhysicalRealiser<'w> {
    type RealisedHandle = (ColliderHandle, BodyHandle);

    fn new_shape(
        &mut self,
        shape_def: &def::ShapeDefinition,
        parent: Self::RealisedHandle,
        parent_joint: &def::Joint,
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

        let (parent_ch, parent_body) = parent;
        let parent_shape = self.world.shape(parent_ch).expect("Parent has no collider");

        // get parameters from shape definition
        let (body_shape, rel_pos, rotation) = shape_from_def(&shape_def, &parent_shape);

        // get parent global position
        let parent_pos = {
            match self.world.physics.multibody_link(parent_body) {
                Some(link) => link.position(),
                None => Isometry3::new(self.next_spawn_pos, rotation), // spawn position of full entity
            }
        };

        // parse parameters
        let joint_params = {
            let mut shift = Isometry3::new(rel_pos, zero());
            shift.append_rotation_mut(&UnitQuaternion::new(rotation));
            shift
        };

        let link = match parent_joint {
            def::Joint::Ground => add_link(
                self.world,
                parent_body,
                FreeJoint::new(parent_pos),
                &body_shape,
            ),
            def::Joint::Fixed => add_link(
                self.world,
                parent_body,
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

        (collider, link)
    }

    fn root(&self) -> (Self::RealisedHandle, def::Joint) {
        (
            (
                self.world.ground_collider.expect("Ground required"),
                BodyHandle::ground(),
            ),
            def::Joint::Ground,
        )
    }
}

impl ObjectShape {
    fn from_def(def: &def::ShapeDefinition) -> Self {
        match def {
            def::ShapeDefinition::Cuboid { dims, .. } => {
                let (w, h, d) = dims.components_scaled();
                ObjectShape::Cuboid(Vector3::new(w, h, d))
            }
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
