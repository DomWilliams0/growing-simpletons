use super::*;
use nalgebra::Vector3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

pub type Dims = Vec3;
pub type RelativePosition = Vec3;
pub type Rotation = Vec3;

#[derive(Debug, Serialize, Deserialize)]
pub enum ShapeDefinition {
    Cuboid(Dims, RelativePosition, Rotation),
}

// struct Sphere {
//     radius: Coord,
// }

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
struct ConnectPoint(Coord, Coord, Coord);

// TODO other joint types with controllable inputs
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Joint {
    src_connect: ConnectPoint,
    dst_connect: ConnectPoint,
    pub joint_type: JointType,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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

impl Into<Vector3<Coord>> for body::Vec3 {
    fn into(self) -> Vector3<Coord> {
        Vector3::new(self.x, self.y, self.z)
    }
}
