#![allow(dead_code)]

extern crate nalgebra;
extern crate ncollide3d;
extern crate petgraph;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub mod body;
pub mod serialise;
pub mod tree;

pub type Coord = f32;
