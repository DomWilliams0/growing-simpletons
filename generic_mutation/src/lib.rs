use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;

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
    fn range(&self) -> (Param, Param);

    fn get(&mut self) -> &mut Param;

    fn update(&mut self, val: Param) {
        let (min, max) = self.range();
        *self.get() = (max - min) * val + min;
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
            let val = *self.get() + rhs;
            if val < 0.0 {
                0.0
            } else if val > 1.0 {
                1.0
            } else {
                val
            }
        };
        self.update(clamped);
    }
}

pub fn mutate<P: ParamHolder, MG: MutationGen>(param_holder: ParamHolderRef<P>, mut mut_gen: MG) {
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

        fn get(&mut self) -> &mut Param {
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
        mutate(holder.clone(), ConstGen { 0: 0.5 });

        let expected = 10.0; // 20.0 * 0.5
        let diff = (holder.borrow().x.0 - expected).abs();
        assert!(diff < 0.001);
    }

    #[test]
    fn test_clamp() {
        let holder = Rc::new(RefCell::new(TestHolder {
            x: TestParam { 0: 0.0 },
        }));
        mutate(holder.clone(), ConstGen { 0: -0.5 });
        assert!(holder.borrow().x.0 < 0.001);

        // should be equal to max
        mutate(holder.clone(), ConstGen { 0: 1.5 });
        assert!((holder.borrow().x.0 - 20.0).abs() < 0.001);
    }
}
