use petgraph;
pub use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use body;

type Node = body::Shape;
type Edge = body::Joint;
type GraphSize = petgraph::graph::DefaultIx;
type Tree = petgraph::Graph<Node, Edge, petgraph::Directed, GraphSize>;

#[derive(Debug, Default)]
pub struct BodyTree {
    tree: Tree,
    root: NodeIndex,
}

impl BodyTree {
    // TODO use results instead of panics

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
        shape: &body::Shape,
        parent: Self::RealisedHandle,
        parent_joint: &body::Joint,
    ) -> Self::RealisedHandle;

    fn root(&self) -> (Self::RealisedHandle, body::Joint);
}

#[derive(Default)]
struct DebugRealiser {
    last_node: i64,
    root: String,
}

/*
impl TreeRealiser for DebugRealiser {
    type RealisedHandle = String;

    fn new_shape(&mut self, _shape: &body::Cuboid) -> Self::RealisedHandle {
        let id = {
            self.last_node += 1;
            self.last_node
        };
        format!("Shape{}", id)
    }

    fn new_joint(
        &mut self,
        src: &Self::RealisedHandle,
        dst: &Self::RealisedHandle,
        _joint: &body::Joint,
    ) -> Self::RealisedHandle {
        let s = format!("Joint({} => {})", src, dst);
        self.root = s.clone();
        s
    }

    fn root(&self) -> (Self::RealisedHandle, body::Joint) { (String::from("ROOT"), body::Joint::default()) }
}
*/

#[cfg(test)]
mod tests {

    use super::*;

    fn shape() -> Node {
        body::Cuboid::new(body::Dims::new(5.0, 5.0, 5.0))
    }
    fn joint() -> Edge {
        body::Joint::default()
    }

    #[test]
    fn realiser() {
        let mut tree = BodyTree::with_root(shape());
        let mut parent = tree.root();
        parent = tree.add_child(parent, shape(), joint());
        tree.add_child(parent, shape(), joint());
        tree.add_child(parent, shape(), joint());

        let mut r = DebugRealiser::default();
        tree.recurse(&mut r);
        println!("{}", r.root);
        //assert_eq!(&r.root, "Joint(Joint(Shape1;Shape2);Shape3)");
    }

    /*
    #[test]
    fn valid_tree() {
        let tree = BodyTree::default();
        assert!(!tree.is_valid());

        let tree = BodyTree::with_root(shape());
        assert!(tree.is_valid());

        let mut tree = BodyTree::with_root(joint());
        let parent = tree.root();
        assert!(!tree.is_valid());

        tree.add_child(parent, shape());
        assert!(!tree.is_valid());

        tree.add_child(parent, shape());
        assert!(tree.is_valid());
    }

    #[test]
    #[should_panic]
    fn one_root() {
        let mut tree = BodyTree::default();
        tree.set_root(shape());
        tree.set_root(shape());
    }

    #[test]
    #[should_panic]
    fn no_children_for_terminals() {
        let mut tree = BodyTree::with_root(shape());
        let root = tree.root();
        tree.add_child(root, shape());
    }

    #[test]
    #[should_panic]
    fn full_joint() {
        let mut tree = BodyTree::with_root(joint());
        let parent = tree.root();
        tree.add_child(parent, shape());
        tree.add_child(parent, shape());
        tree.add_child(parent, shape());
    }
    */
}
