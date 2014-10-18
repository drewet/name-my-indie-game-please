use cgmath;
use cgmath::Plane;

enum Tree<LeafType> {
    Subtree(INode),
    Leaf(LeafType)
}

struct INode<LeafType> {
    plane: Plane<f32>,

    /// away from normal
    inside: Box<Tree<LeafType>>,
    /// towards normal
    outside: Box<Tree<LeafType>>
}

impl<LeafType> Tree<LeafType> {
    pub fn map_leaves<NewLeafType>(self, f: |LeafType| -> NewLeafType) -> Tree<NewLeafType> {
        match self {
            Subtree(inode) => {
                Subtree(INode {
                    plane: inode.plane,
                    inside: box inode.inside.map_leaves(f),
                    outside: box inode.outside.map_leaves(f),
                })
            },
            Leaf(leaf) => Leaf(f(leaf))
        }
    }
