use petgraph;
pub use petgraph::graph::NodeIndex;

use body;

#[derive(Debug)]
pub enum Node {
    Joint(body::Joint),
    Shape(body::Cuboid),
}

impl Node {
    fn arity(&self) -> usize {
        match self {
            Node::Joint(_) => 2,
            Node::Shape(_) => 0,
        }
    }
}

type Edge = i32; // unused?
type GraphSize = petgraph::graph::DefaultIx;
type Tree = petgraph::Graph<Node, Edge, petgraph::Directed, GraphSize>;

#[derive(Debug, Default)]
pub struct BodyTree {
    tree: Tree,
    root: Option<NodeIndex>,
}

impl BodyTree {
    // TODO use results instead of panics

    pub fn with_root(root: Node) -> Self {
        let mut tree = Self::default();
        tree.set_root(root);
        tree
    }

    pub fn set_root(&mut self, node: Node) -> NodeIndex {
        if self.tree.node_count() != 0 {
            panic!("Cannot set root of non-empty tree!")
        }

        let root = self.tree.add_node(node);
        self.root = Some(root);
        root
    }

    /// Panics if invalid
    pub fn root(&self) -> NodeIndex {
        self.root.unwrap()
    }

    pub fn add_child(&mut self, parent: NodeIndex, child: Node) -> NodeIndex {
        if self.children_count(parent) + 1 > self.tree[parent].arity() {
            panic!("Parent is full");
        }

        let new_node = self.tree.add_node(child);
        self.tree.add_edge(new_node, parent, 0);
        new_node
    }

    fn children_count(&self, parent: NodeIndex) -> usize {
        self.get_children(parent).count()
    }

    pub fn get_children(&self, parent: NodeIndex) -> petgraph::graph::Neighbors<Edge> {
        self.tree
            .neighbors_directed(parent, petgraph::Direction::Incoming)
    }

    pub fn is_valid(&self) -> bool {
        match self.root {
            None => false,
            Some(root) => {
                let mut visit = petgraph::visit::Dfs::new(&self.tree, root);
                while let Some(node) = visit.next(&self.tree) {
                    if self.tree[node].arity() != self.children_count(node) {
                        return false;
                    }
                }
                true
            }
        }
    }

    // TODO trait for producing them
    // has item for bodyhandle
    //      physics body handle for this, or can be node int for rendering
    fn actually_recurse<R: TreeRealiser>(
        &self,
        current: NodeIndex,
        realiser: &mut R,
    ) -> R::RealisedHandle {
        let node = &self.tree[current];
        match node {
            Node::Shape(cuboid) => realiser.new_shape(cuboid),

            Node::Joint(joint) => {
                let children: Vec<R::RealisedHandle> = self.get_children(current)
                    .map(|child| self.actually_recurse(child, realiser))
                    .collect();
                realiser.new_joint(joint, &children)
            }
        }
    }

    pub fn recurse<R: TreeRealiser>(&self, realiser: &mut R) {
        assert!(self.is_valid());
        self.actually_recurse(self.root.unwrap(), realiser);
    }
}

pub trait TreeRealiser {
    type RealisedHandle;

    fn new_shape(&mut self, shape: &body::Cuboid) -> Self::RealisedHandle;
    fn new_joint(
        &mut self,
        shape: &body::Joint,
        children: &[Self::RealisedHandle],
    ) -> Self::RealisedHandle;
}

#[derive(Default)]
struct DebugRealiser {
    last_node: i64,
    root: String,
}

impl TreeRealiser for DebugRealiser {
    type RealisedHandle = String;

    fn new_shape(&mut self, shape: &body::Cuboid) -> Self::RealisedHandle {
        let id = {
            self.last_node += 1;
            self.last_node
        };
        format!("Shape{}", id)
    }

    fn new_joint(
        &mut self,
        shape: &body::Joint,
        children: &[Self::RealisedHandle],
    ) -> Self::RealisedHandle {
        let s = format!("Joint({})", children.join(";"));
        self.root = s.clone();
        s
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn shape() -> Node {
        Node::Shape(body::Cuboid::new(body::Dims::new(5.0, 5.0, 5.0)))
    }
    fn joint() -> Node {
        Node::Joint(body::Joint::default())
    }

    #[test]
    fn realiser() {
        let mut tree = BodyTree::with_root(joint());
        let parent = tree.root();
        tree.add_child(parent, shape());
        let parent = tree.add_child(parent, joint());
        tree.add_child(parent, shape());
        tree.add_child(parent, shape());

        let mut r = DebugRealiser::default();
        tree.recurse(&mut r);
        assert_eq!(&r.root, "Joint(Joint(Shape1;Shape2);Shape3)");
    }

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
}
