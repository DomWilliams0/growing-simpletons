#![allow(dead_code)]

extern crate nalgebra;
extern crate nphysics3d;
extern crate petgraph;

mod body;
mod tree;

use nalgebra::Vector3;

type Coord = f32;
type Pos = Vector3<Coord>;
