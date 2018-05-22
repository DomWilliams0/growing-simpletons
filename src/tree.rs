use petgraph;
use petgraph::graph::NodeIndex;

use body;

#[derive(Debug)]
enum Node {
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

#[derive(Debug)]
struct BodyTree {
    tree: Tree,
    root: Option<NodeIndex>,
}

impl BodyTree {
    // TODO use results instead of panics

    fn new() -> Self {
        Self {
            tree: Tree::new(),
            root: None,
        }
    }

    fn set_root(&mut self, node: Node) -> NodeIndex {
        if self.tree.node_count() != 0 {
            panic!("Cannot set root of non-empty tree!")
        }

        let root = self.tree.add_node(node);
        self.root = Some(root);
        root
    }

    fn add_child(&mut self, parent: NodeIndex, child: Node) -> NodeIndex {
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

    fn get_children(&self, parent: NodeIndex) -> petgraph::graph::Neighbors<Edge> {
        self.tree
            .neighbors_directed(parent, petgraph::Direction::Incoming)
    }

    fn is_valid(&self) -> bool {
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
    fn valid_tree() {
        let tree = BodyTree::new();
        assert!(!tree.is_valid());

        let mut tree = BodyTree::new();
        tree.set_root(shape());
        assert!(tree.is_valid());

        let mut tree = BodyTree::new();
        let parent = tree.set_root(joint());
        assert!(!tree.is_valid());

        tree.add_child(parent, shape());
        assert!(!tree.is_valid());

        tree.add_child(parent, shape());
        assert!(tree.is_valid());
    }

    #[test]
    #[should_panic]
    fn one_root() {
        let mut tree = BodyTree::new();
        tree.set_root(shape());
        tree.set_root(shape());
    }

    #[test]
    #[should_panic]
    fn no_children_for_terminals() {
        let mut tree = BodyTree::new();
        let parent = tree.set_root(shape());
        tree.add_child(parent, shape());
    }

    #[test]
    #[should_panic]
    fn full_joint() {
        let mut tree = BodyTree::new();
        let parent = tree.set_root(joint());
        tree.add_child(parent, shape());
        tree.add_child(parent, shape());
        tree.add_child(parent, shape());
    }
}
