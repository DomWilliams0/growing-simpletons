use petgraph;
pub use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use body;

type Node = body::ShapeDefinition;
type Edge = body::Joint;
type GraphSize = petgraph::graph::DefaultIx;
type Tree = petgraph::Graph<Node, Edge, petgraph::Directed, GraphSize>;

#[derive(Debug, Default)]
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
        parent_joint: &body::Joint,
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

    pub fn recurse<R: TreeRealiser>(&self, realiser: &mut R) {
        let (handle, joint) = realiser.root();
        self.actually_recurse(self.root, handle, &joint, realiser);
    }
}

pub trait TreeRealiser {
    type RealisedHandle: Clone;

    fn new_shape(
        &mut self,
        shape_def: &body::ShapeDefinition,
        parent: Self::RealisedHandle,
        parent_joint: &body::Joint,
    ) -> Self::RealisedHandle;

    fn root(&self) -> (Self::RealisedHandle, body::Joint);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DebugRealiser {
        last_node: i64,
        expected_order: Vec<(i64, i64)>,
    }

    impl TreeRealiser for DebugRealiser {
        type RealisedHandle = i64;

        fn new_shape(
            &mut self,
            _: &body::ShapeDefinition,
            parent: Self::RealisedHandle,
            _: &body::Joint,
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

        fn root(&self) -> (Self::RealisedHandle, body::Joint) {
            (0, body::Joint::new(body::JointType::Fixed))
        }
    }

    fn shape() -> Node {
        body::ShapeDefinition::Cuboid(
            body::Dims::new(5.0, 5.0, 5.0),
            body::RelativePosition::new(0.0, 0.0, 0.0),
            body::Rotation::new(0.0, 0.0, 0.0),
            )
    }

    fn joint() -> Edge {
        body::Joint::new(body::JointType::Fixed)
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
        tree.recurse(&mut r);
    }
}
