pub mod def {
    use super::params::*;
    pub use generic_mutation::{ParamHolder, ParamSet3d, RangedParam};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ShapeDefinition {
        Cuboid {
            dims: ParamSet3d<Dimension>,
            pos: (FaceIndex, FaceCoord, FaceCoord),
            rot: ParamSet3d<Rotation>,
        },
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum Joint {
        Fixed,
        Ground,
    }

    // TODO this should be so much shorter
    pub fn new_cuboid(
        dims: (f64, f64, f64),
        pos: (f64, f64, f64),
        rot: (f64, f64, f64),
    ) -> ShapeDefinition {
        let dims = ParamSet3d::new(
            Dimension::new(dims.0),
            Dimension::new(dims.1),
            Dimension::new(dims.2),
        );
        let pos = (
            FaceIndex::new(pos.0),
            FaceCoord::new(pos.1),
            FaceCoord::new(pos.2),
        );

        let rot = ParamSet3d::new(
            Rotation::new(rot.0),
            Rotation::new(rot.1),
            Rotation::new(rot.2),
        );
        ShapeDefinition::Cuboid { dims, pos, rot }
    }

    impl ParamHolder for ShapeDefinition {
        fn param_count(&self) -> usize {
            match self {
                ShapeDefinition::Cuboid { dims, rot, .. } => {
                    dims.param_count() + 3 + rot.param_count()
                }
            }
        }

        fn get_param(&mut self, index: usize) -> &mut RangedParam {
            match self {
                ShapeDefinition::Cuboid { dims, pos, rot } => match index {
                    0...2 => dims.get_param(index % 3),
                    3 => &mut pos.0,
                    4 => &mut pos.1,
                    5 => &mut pos.2,
                    6...8 => rot.get_param(index % 3),
                    _ => panic!("out of bounds"),
                },
            }
        }
    }
}

pub mod params {
    use generic_mutation::{Param, RangedParam};

    /// x y z size of a cuboid
    #[derive(Debug, Default, Clone, Copy, new, Serialize, Deserialize)]
    pub struct Dimension(f64);

    // (x, y) on the parent cuboid
    // TODO generalise for sphere too?
    #[derive(Debug, Default, Clone, Copy, new, Serialize, Deserialize)]
    pub struct FaceCoord(f64);

    // face index, unscaled here
    #[derive(Debug, Default, Clone, Copy, new, Serialize, Deserialize)]
    pub struct FaceIndex(f64);

    /// x y z rotation relative to parent
    #[derive(Debug, Default, Clone, Copy, new, Serialize, Deserialize)]
    pub struct Rotation(f64);

    impl RangedParam for Dimension {
        fn range(&self) -> (Param, Param) {
            (0.1, 4.0)
        }

        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

    impl RangedParam for FaceCoord {
        fn range(&self) -> (Param, Param) {
            (-1.0, 1.0)
        }

        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

    impl RangedParam for FaceIndex {
        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

    impl RangedParam for Rotation {
        fn range(&self) -> (Param, Param) {
            (0.0, ::std::f64::consts::PI)
        }

        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

}
