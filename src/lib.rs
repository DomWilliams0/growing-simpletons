#![allow(dead_code)]

extern crate nalgebra;
extern crate ncollide3d;
extern crate nphysics3d;
extern crate petgraph;

pub mod body;
pub mod physics;
pub mod tree;

use nalgebra::Vector3;

type Coord = f32;
type Pos = Vector3<Coord>;
