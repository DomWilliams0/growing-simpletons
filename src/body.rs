use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

pub type Dims = Vec3;
pub type RelativePosition = Vec3;
pub type Rotation = Vec3;

#[derive(Debug)]
pub enum ShapeDefinition {
    Cuboid(Dims, RelativePosition, Rotation),
}

// struct Sphere {
//     radius: Coord,
// }

#[derive(Debug, Default, Copy, Clone)]
struct ConnectPoint(Coord, Coord, Coord);

// TODO other joint types with controllable inputs
#[derive(Debug, Copy, Clone)]
pub struct Joint {
    src_connect: ConnectPoint,
    dst_connect: ConnectPoint,
    pub joint_type: JointType,
}

#[derive(Debug, Copy, Clone)]
pub enum JointType {
    Ground,
    Fixed,
}

impl Joint {
    pub fn new(joint_type: JointType) -> Self {
        Self {
            src_connect: Default::default(),
            dst_connect: Default::default(),
            joint_type,
        }
    }
}

impl Dims {
    pub fn new(x: Coord, y: Coord, z: Coord) -> Self {
        Self { x, y, z }
    }
}
