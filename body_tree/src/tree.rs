use petgraph;
pub use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, EdgeRef};
use rand::{self, Rng, RngCore};

use body::def;
use generic_mutation;

type Node = def::ShapeDefinition;
type Edge = def::Joint;
type GraphSize = petgraph::graph::DefaultIx;
type Tree = petgraph::Graph<Node, Edge, petgraph::Directed, GraphSize>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BodyTree {
    tree: Tree,
    root: NodeIndex,
}

impl BodyTree {
    pub fn with_root(root_node: Node) -> Self {
        let mut tree = Tree::new();
        let root = tree.add_node(root_node);
        Self { tree, root }
    }

    pub fn root(&self) -> NodeIndex {
        self.root
    }

    pub fn add_child(&mut self, parent: NodeIndex, child: Node, edge: Edge) -> NodeIndex {
        // TODO limit children count at all?
        let new_node = self.tree.add_node(child);
        self.tree.add_edge(new_node, parent, edge);
        new_node
    }

    fn children_count(&self, parent: NodeIndex) -> usize {
        self.get_children(parent).count()
    }

    pub fn get_children(
        &self,
        parent: NodeIndex,
    ) -> petgraph::graph::Edges<Edge, petgraph::Directed> {
        self.tree
            .edges_directed(parent, petgraph::Direction::Incoming)
    }

    fn actually_recurse<R: TreeRealiser>(
        &self,
        current: NodeIndex,
        parent_handle: R::RealisedHandle,
        parent_joint: &def::Joint,
        realiser: &mut R,
    ) {
        let node = &self.tree[current];

        // create shape for self
        let new_node = realiser.new_shape(node, parent_handle, parent_joint);

        // children
        for edge_ref in self.get_children(current) {
            let child = edge_ref.source();
            let joint = edge_ref.weight();
            self.actually_recurse(child, new_node.clone(), joint, realiser);
        }
    }

    pub fn realise<R: TreeRealiser>(&self, realiser: &mut R) {
        let (handle, joint) = realiser.root();
        self.actually_recurse(self.root, handle, &joint, realiser);
    }

    fn actually_mutate<MG: generic_mutation::MutationGen>(&mut self, mut _mut_gen: MG) {
        let mut dfs = Dfs::new(&self.tree, self.root);
        while let Some(_node) = dfs.next(&self.tree) {
            // TODO mutate
        }
    }

    pub fn mutate(&mut self, mut_rate: f64) {
        let mut rng = rand::thread_rng();
        let mutator = RandomMutationGen {
            rng: &mut rng,
            rate: mut_rate,
        };
        self.actually_mutate(mutator)
    }
}

struct RandomMutationGen<'a> {
    rng: &'a mut RngCore,
    rate: f64,
}

impl<'a> generic_mutation::MutationGen for RandomMutationGen<'a> {
    fn gen(&mut self) -> generic_mutation::Param {
        self.rng.gen()
    }
}

pub trait TreeRealiser {
    type RealisedHandle: Clone;

    fn new_shape(
        &mut self,
        shape_def: &def::ShapeDefinition,
        parent: Self::RealisedHandle,
        parent_joint: &def::Joint,
    ) -> Self::RealisedHandle;

    fn root(&self) -> (Self::RealisedHandle, def::Joint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use body::def;

    struct DebugRealiser {
        last_node: i64,
        expected_order: Vec<(i64, i64)>,
    }

    impl TreeRealiser for DebugRealiser {
        type RealisedHandle = i64;

        fn new_shape(
            &mut self,
            _: &def::ShapeDefinition,
            parent: Self::RealisedHandle,
            _: &def::Joint,
        ) -> Self::RealisedHandle {
            let id = {
                self.last_node += 1;
                self.last_node
            };

            assert!(!self.expected_order.is_empty());
            let (expected_id, expected_parent) = self.expected_order.remove(0);

            assert_eq!(id, expected_id);
            assert_eq!(parent, expected_parent);
            id
        }

        fn root(&self) -> (Self::RealisedHandle, def::Joint) {
            (0, def::Joint::Fixed)
        }
    }

    fn shape() -> Node {
        def::new_cuboid((5.0, 5.0, 5.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0))
    }

    fn joint() -> Edge {
        def::Joint::Fixed
    }

    #[test]
    fn realiser() {
        let mut tree = BodyTree::with_root(shape());
        let mut parent = tree.root();
        parent = tree.add_child(parent, shape(), joint());
        tree.add_child(parent, shape(), joint());
        tree.add_child(parent, shape(), joint());

        let mut r = DebugRealiser {
            last_node: 0,
            expected_order: vec![(1, 0), (2, 1), (3, 2), (4, 2)],
        };
        tree.realise(&mut r);
    }
}
