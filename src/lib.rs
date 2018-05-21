#![allow(dead_code)]

extern crate nalgebra;
extern crate ncollide3d;
extern crate nphysics3d;
extern crate petgraph;

#[macro_use]
extern crate lazy_static;

mod body;
pub mod physics;
mod tree;

use nalgebra::Vector3;

type Coord = f32;
type Pos = Vector3<Coord>;
