use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;

#[cfg(feature = "serialize")]
extern crate serde;

#[macro_use]
#[cfg(feature = "serialize")]
extern crate serde_derive;

pub type Param = f64;
pub type ParamHolderRef<P> = Rc<RefCell<P>>;

#[derive(Debug)]
pub struct GenericParams<PH: ParamHolder> {
    owner: Rc<RefCell<PH>>,
    n: usize,
}

/// An entity with multiple parameters.
pub trait ParamHolder {
    fn param_count(&self) -> usize;
    fn get_param(&mut self, index: usize) -> &mut RangedParam;
}

/// An individual parameter with a specified range.
pub trait RangedParam {
    /// (min, max)
    fn range(&self) -> (Param, Param) {
        (0.0, 1.0) // unscaled
    }

    fn get(&self) -> Param;

    fn get_mut(&mut self) -> &mut Param;

    fn get_scaled(&self) -> Param {
        let (min, max) = self.range();
        (max - min) * self.get() + min
    }
}

/// Collection of related parameters in multiple dimensions.
pub trait ParamSet<P: RangedParam>: ParamHolder + Default {}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Default)]
pub struct ParamSet3d<P: RangedParam> {
    x: P,
    y: P,
    z: P,
}

impl<P: RangedParam + Default> ParamSet<P> for ParamSet3d<P> {}

impl<P: RangedParam> ParamHolder for ParamSet3d<P> {
    fn param_count(&self) -> usize {
        3
    }
    fn get_param(&mut self, index: usize) -> &mut RangedParam {
        match index % 3 {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("out of bounds"),
        }
    }
}

impl<P: RangedParam> ParamSet3d<P> {
    pub fn new(x: P, y: P, z: P) -> Self {
        Self { x, y, z }
    }

    pub fn components_scaled(&self) -> (Param, Param, Param) {
        (
            self.x.get_scaled(),
            self.y.get_scaled(),
            self.z.get_scaled(),
        )
    }
}

/// A mutation generator, that produces an offset to add to the current value.
/// Should range between -1.0 and 1.0, but the result will be clamped anyway
pub trait MutationGen {
    fn gen(&mut self) -> Param;
}

impl<PH: ParamHolder> GenericParams<PH> {
    fn new(holder: Rc<RefCell<PH>>) -> Self {
        let n = holder.borrow().param_count();
        Self { owner: holder, n }
    }
}

impl<'a> AddAssign<Param> for &'a mut RangedParam {
    fn add_assign(&mut self, rhs: Param) {
        let clamped = {
            let val = *self.get_mut() + rhs;
            if val < 0.0 {
                0.0
            } else if val > 1.0 {
                1.0
            } else {
                val
            }
        };
        *self.get_mut() = clamped;
    }
}

pub fn mutate<P: ParamHolder, MG: MutationGen>(param_holder: ParamHolderRef<P>, mut_gen: &mut MG) {
    let params = GenericParams::new(param_holder);

    for i in 0..params.n {
        let mut holder = params.owner.borrow_mut();
        let mut p: &mut RangedParam = holder.get_param(i);
        p += mut_gen.gen();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParam(Param);

    struct TestHolder {
        x: TestParam,
    }

    impl ParamHolder for TestHolder {
        fn param_count(&self) -> usize {
            1
        }

        fn get_param(&mut self, index: usize) -> &mut RangedParam {
            match index {
                0 => &mut self.x,
                _ => panic!("Bad param index"),
            }
        }
    }

    impl RangedParam for TestParam {
        fn range(&self) -> (Param, Param) {
            (0.0, 20.0)
        }

        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

    struct ConstGen(Param);

    impl MutationGen for ConstGen {
        fn gen(&mut self) -> Param {
            self.0
        }
    }

    #[test]
    fn test_mutate() {
        let holder = Rc::new(RefCell::new(TestHolder {
            x: TestParam { 0: 0.0 },
        }));
        mutate(holder.clone(), &mut ConstGen { 0: 0.5 });

        let expected = 10.0; // 20.0 * 0.5
        let diff = (holder.borrow().x.get_scaled() - expected).abs();
        assert!(diff < 0.001);
    }

    #[test]
    fn test_clamp() {
        let holder = Rc::new(RefCell::new(TestHolder {
            x: TestParam { 0: 0.0 },
        }));
        mutate(holder.clone(), &mut ConstGen { 0: -0.5 });
        assert!(holder.borrow().x.get_scaled() < 0.001);

        // should be equal to max
        mutate(holder.clone(), &mut ConstGen { 0: 1.5 });
        assert!((holder.borrow().x.get_scaled() - 20.0).abs() < 0.001);
    }

    #[derive(Debug, Default)]
    struct Pos(Param);

    #[derive(Debug, Default)]
    struct MultiShape {
        pos: ParamSet3d<Pos>,
    }

    impl RangedParam for Pos {
        fn range(&self) -> (Param, Param) {
            (0.0, 10.0)
        }

        fn get(&self) -> Param {
            self.0
        }

        fn get_mut(&mut self) -> &mut Param {
            &mut self.0
        }
    }

    impl ParamHolder for MultiShape {
        fn param_count(&self) -> usize {
            3
        }

        fn get_param(&mut self, index: usize) -> &mut RangedParam {
            match index {
                0...2 => self.pos.get_param(index),
                _ => panic!("Bad param index"),
            }
        }
    }

    #[test]
    fn test_paramset() {
        let holder = Rc::new(RefCell::new(MultiShape::default()));
        mutate(holder.clone(), &mut ConstGen { 0: 0.25 });

        let expected = 2.5; // 10.0 * 0.25
        let pos = &holder.borrow().pos;
        assert!((pos.x.get_scaled() - expected).abs() < 0.001);
        assert!((pos.y.get_scaled() - expected).abs() < 0.001);
        assert!((pos.z.get_scaled() - expected).abs() < 0.001);
    }
}
