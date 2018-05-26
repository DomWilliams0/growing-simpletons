#![allow(dead_code)]

extern crate nalgebra;
extern crate ncollide3d;
extern crate petgraph;
extern crate rand;

extern crate generic_mutation;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate derive_new;

pub mod body;
pub mod serialise;
pub mod tree;

pub type Coord = f64;

pub type Population = Vec<tree::BodyTree>;
