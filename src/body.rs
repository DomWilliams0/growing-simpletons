use super::*;

pub struct Dims {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

trait Shape {
    // cuboid: (face, x, y)
    // sphere (TODO spherical coordinates)
    fn get_local_point(&self, cp: &ConnectPoint) -> Pos;
}

pub struct Cuboid {
    dims: Dims,
}

struct Sphere {
    radius: Coord,
}

struct ConnectPoint(Coord, Coord, Coord);

// TODO other joint types with controllable inputs
struct Joint {
    src_connect: ConnectPoint,
    dst_connect: ConnectPoint,
}

impl Cuboid {
    pub fn new(dims: Dims) -> Self {
        Self { dims }
    }
}
