/// This is a simple implementation of a quadtree data structure
/// to partition a 2D space into 4 quadrants recursively
/// with the objective of evaluating a phyiscs simulation
/// that computes both mechanical forces and collisions
/// amont point particles
use std::collections::VecDeque;

use crate::physics::Body;

const DEFAULT_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct SquareBox {
    /// The center of the square
    center: [f64; 2],

    /// Half the side-length of the square
    half_size: f64,
}

impl SquareBox {
    pub fn new(center: [f64; 2], half_size: f64) -> Self {
        Self { center, half_size }
    }

    pub fn from_bodies(bodies: &[Body]) -> Self {
        let bbox: [f64; 4] =
            bodies
                .iter()
                .fold([f64::MAX, f64::MIN, f64::MAX, f64::MIN], |acc, body| {
                    let x = body.position[0];
                    let y = body.position[1];
                    [acc[0].min(x), acc[1].max(x), acc[2].min(y), acc[3].max(y)]
                });
        Self {
            center: [(bbox[0] + bbox[1]) / 2.0, (bbox[2] + bbox[3]) / 2.0],
            half_size: (bbox[1] - bbox[0]).max(bbox[3] - bbox[2]) / 2.0,
        }
    }

    #[inline(always)]
    pub fn x_min(&self) -> f64 {
        self.center[0] - self.half_size
    }

    #[inline(always)]
    pub fn x_max(&self) -> f64 {
        self.center[0] + self.half_size
    }

    #[inline(always)]
    pub fn y_min(&self) -> f64 {
        self.center[1] - self.half_size
    }

    #[inline(always)]
    pub fn y_max(&self) -> f64 {
        self.center[1] + self.half_size
    }

    #[inline(always)]
    pub fn center(&self) -> [f64; 2] {
        self.center
    }

    #[inline(always)]
    pub fn size(&self) -> f64 {
        self.half_size * 2.0
    }

    #[inline(always)]
    pub fn contains(&self, point: &[f64; 2]) -> bool {
        self.x_min() <= point[0]
            && point[0] <= self.x_max()
            && self.y_min() <= point[1]
            && point[1] <= self.y_max()
    }

    #[inline(always)]
    pub fn contains_box(&self, other: &SquareBox) -> bool {
        self.x_min() <= other.x_min()
            && other.x_max() <= self.x_max()
            && self.y_min() <= other.y_min()
            && other.y_max() <= self.y_max()
    }

    /// Returns the quadrant of the square where the point is located
    /// It assumes the point is within the square !!!
    pub fn get_quadrant_unchecked(&self, point: &[f64; 2]) -> usize {
        let x = point[0];
        let y = point[1];
        if y > self.center[1] {
            if x > self.center[0] {
                0 // North-East
            } else {
                1 // North-West
            }
        } else if x < self.center[0] {
            2 // South-West
        } else {
            3 // South-East
        }
    }

    /// Quadrants of the square
    pub fn north_east(&self) -> Self {
        let x = self.center[0] + self.half_size / 2.0;
        let y = self.center[1] + self.half_size / 2.0;
        SquareBox {
            center: [x, y],
            half_size: self.half_size / 2.0,
        }
    }

    pub fn north_west(&self) -> Self {
        let x = self.center[0] - self.half_size / 2.0;
        let y = self.center[1] + self.half_size / 2.0;
        SquareBox {
            center: [x, y],
            half_size: self.half_size / 2.0,
        }
    }

    pub fn south_west(&self) -> Self {
        let x = self.center[0] - self.half_size / 2.0;
        let y = self.center[1] - self.half_size / 2.0;
        SquareBox {
            center: [x, y],
            half_size: self.half_size / 2.0,
        }
    }

    pub fn south_east(&self) -> Self {
        let x = self.center[0] + self.half_size / 2.0;
        let y = self.center[1] - self.half_size / 2.0;
        SquareBox {
            center: [x, y],
            half_size: self.half_size / 2.0,
        }
    }
}

/// Represents a given quadrant (subdivision) of a quadtree
pub struct QuadTreeNode {
    /// The quadrant geometry
    boundary: SquareBox,

    /// The indexes of the points stored in this quadrant
    /// Empty unless this is a leaf node
    referenced_indices: Vec<usize>,

    /// The index of where the children nodes start in the nodes vector
    /// (which are contiguous in the vector)
    children_idx: usize,

    /// mass of the quadrant
    /// (as in the sum of the masses of the bodies living in this quadrant including its children)
    /// This is done to optimize gravity force computation
    mass: f64,
}

impl QuadTreeNode {
    fn new(boundary: SquareBox) -> Self {
        Self {
            boundary,
            referenced_indices: Vec::with_capacity(DEFAULT_CAPACITY),
            children_idx: 0,
            mass: 0.0,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children_idx == 0
    }

    pub fn referenced_indices(&self) -> &[usize] {
        self.referenced_indices.as_slice()
    }

    pub fn boundary(&self) -> &SquareBox {
        &self.boundary
    }

    pub fn children_idx(&self) -> usize {
        self.children_idx
    }

    pub fn mass(&self) -> f64 {
        self.mass
    }
}

/// Represents a quadtree data structure
pub struct SquareQuadtree {
    /// Maximum number of nodes stored in a given quadrant
    capacity: usize,

    /// The nodes of the tree (including the root node)
    /// storing the different subdivisions of the tree
    nodes: Vec<QuadTreeNode>,
}

impl SquareQuadtree {
    const ROOT_IDX: usize = 0;

    /// Creates a new quadtree with the given capacity
    pub fn new(boundary: SquareBox) -> Self {
        let root = QuadTreeNode::new(boundary);
        SquareQuadtree {
            capacity: DEFAULT_CAPACITY,
            nodes: vec![root],
        }
    }

    /// Builder method to set the capacity of the quadtree
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Clear the quadtree but maintain the capacity
    pub fn clear(&mut self, boundary: SquareBox) {
        self.nodes.clear(); // but maintain the capacity
        self.nodes.push(QuadTreeNode::new(boundary));
    }

    /// Inserts a body in the quadtree provided its reference index
    /// Returns true if the body was inserted in the tree
    pub fn insert(&mut self, index: usize, bodies: &[Body]) -> bool {
        if !self.nodes[Self::ROOT_IDX]
            .boundary
            .contains(&bodies[index].position)
        {
            return false;
        }
        self.insert_unchecked(index, bodies);
        true
    }

    /// Inserts a body in the quadtree provided its reference index
    /// It does not check if the point is within the boundary of the root node
    pub fn insert_unchecked(&mut self, index: usize, bodies: &[Body]) {
        // Breadth-first search to find the leaf node where the point should be inserted
        let mut deque: VecDeque<usize> = vec![Self::ROOT_IDX].into();
        while let Some(node_idx) = deque.pop_front() {
            self.nodes[node_idx].mass += bodies[index].mass;
            if self.nodes[node_idx].is_leaf() {
                if self.nodes[node_idx].referenced_indices.len() < self.capacity {
                    self.nodes[node_idx].referenced_indices.push(index);
                    return;
                } else {
                    // Node's capacity limit reached
                    self.subdivide(node_idx, bodies);
                }
            }
            let first_idx = self.nodes[node_idx].children_idx;
            let quadrant = self.nodes[node_idx]
                .boundary
                .get_quadrant_unchecked(&bodies[index].position);
            deque.push_back(first_idx + quadrant);
        }
    }

    pub fn query_range(&self, boundary: SquareBox, bodies: &[Body]) -> Vec<usize> {
        let mut result = Vec::new();
        let mut deque: VecDeque<usize> = vec![Self::ROOT_IDX].into();
        while let Some(node_idx) = deque.pop_front() {
            // Fast-Path: query boundary wraps around this quadtree division
            if boundary.contains_box(&self.nodes[node_idx].boundary) {
                if self.nodes[node_idx].is_leaf() {
                    result.extend(self.nodes[node_idx].referenced_indices());
                } else {
                    let first_idx = self.nodes[node_idx].children_idx;
                    let mut deque: VecDeque<usize> =
                        Vec::from_iter(first_idx..first_idx + 4).into();
                    while let Some(child_idx) = deque.pop_front() {
                        if self.nodes[child_idx].is_leaf() {
                            result.extend(self.nodes[child_idx].referenced_indices());
                        } else {
                            let first_idx = self.nodes[child_idx].children_idx;
                            deque.extend(first_idx..first_idx + 4);
                        }
                    }
                }
            // Slow-Path: Brute-force check (boundaries intersection)
            } else if self.nodes[node_idx].is_leaf() {
                for &idx in self.nodes[node_idx].referenced_indices() {
                    if boundary.contains(&bodies[idx].position) {
                        result.push(idx);
                    }
                }
            } else {
                let first_idx = self.nodes[node_idx].children_idx;
                deque.extend(first_idx..first_idx + 4);
            }
        }
        result
    }

    /// Returns the nodes of the quadtree
    pub fn get_nodes(&self) -> &[QuadTreeNode] {
        self.nodes.as_slice()
    }

    pub fn depth(&self) -> usize {
        let mut curr_depth = 0usize;
        let mut deque: VecDeque<(usize, usize)> = vec![(0, Self::ROOT_IDX)].into();

        while let Some((depth, node_idx)) = deque.pop_front() {
            curr_depth = curr_depth.max(depth);
            if self.nodes[node_idx].is_leaf() {
                continue;
            }
            for child in 0..4 {
                deque.push_back((depth + 1, self.nodes[node_idx].children_idx + child));
            }
        }
        curr_depth
    }
}

/// Private of the SquareQuadtree
impl SquareQuadtree {
    fn subdivide(&mut self, parent_idx: usize, bodies: &[Body]) {
        self.nodes[parent_idx].children_idx = self.nodes.len();

        // Create the 4 children nodes
        let nw = self.nodes[parent_idx].boundary.north_west();
        let ne = self.nodes[parent_idx].boundary.north_east();
        let sw = self.nodes[parent_idx].boundary.south_west();
        let se = self.nodes[parent_idx].boundary.south_east();

        self.nodes.push(QuadTreeNode::new(nw));
        self.nodes.push(QuadTreeNode::new(ne));
        self.nodes.push(QuadTreeNode::new(sw));
        self.nodes.push(QuadTreeNode::new(se));

        // Now transfer the referenced indexes to the new leaf nodes
        let first_child = self.nodes[parent_idx].children_idx;
        for idx in std::mem::take(&mut self.nodes[parent_idx].referenced_indices) {
            let quadrant = self.nodes[parent_idx]
                .boundary
                .get_quadrant_unchecked(&bodies[idx].position);
            self.nodes[first_child + quadrant]
                .referenced_indices
                .push(idx);
            self.nodes[first_child + quadrant].mass += bodies[idx].mass;
        }
    }
}

// Write some tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_box() {
        let square = SquareBox {
            center: [0.0, 0.0],
            half_size: 1.0,
        };

        assert_eq!(square.x_min(), -1.0);
        assert_eq!(square.x_max(), 1.0);
        assert_eq!(square.y_min(), -1.0);
        assert_eq!(square.y_max(), 1.0);

        let point = [0.5, 0.5];
        assert!(square.contains(&point));
        assert_eq!(square.get_quadrant_unchecked(&point), 0);

        let point = [-0.5, 0.5];
        assert!(square.contains(&point));
        assert_eq!(square.get_quadrant_unchecked(&point), 1);

        let point = [-0.5, -0.5];
        assert!(square.contains(&point));
        assert_eq!(square.get_quadrant_unchecked(&point), 2);

        let point = [0.5, -0.5];
        assert!(square.contains(&point));
        assert_eq!(square.get_quadrant_unchecked(&point), 3);
    }

    #[test]
    fn test_quadtree() {
        let boundary = SquareBox {
            center: [0.0, 0.0],
            half_size: 1.0,
        };
        let mut quadtree = SquareQuadtree::new(boundary).with_capacity(2);
        let bodies = vec![
            Body {
                position: [0.5, 0.5],
                mass: 1.0,
                velocity: [0.0, 0.0],
                radius: 1.0,
                color: [255; 4],
            },
            Body {
                position: [-0.5, 0.5],
                mass: 1.0,
                velocity: [0.0, 0.0],
                radius: 1.0,
                color: [255; 4],
            },
            Body {
                position: [-0.5, -0.5],
                mass: 1.0,
                velocity: [0.0, 0.0],
                radius: 1.0,
                color: [255; 4],
            },
            Body {
                position: [0.5, -0.5],
                mass: 1.0,
                velocity: [0.0, 0.0],
                radius: 1.0,
                color: [255; 4],
            },
        ];

        for i in 0..bodies.len() {
            quadtree.insert_unchecked(i, &bodies);
        }

        let nodes = quadtree.get_nodes();
        assert_eq!(nodes.len(), 5);

        let root = &nodes[0];
        assert_eq!(root.referenced_indices.len(), 0);
        assert_eq!(root.mass, 4.0);

        let nw = &nodes[1];
        assert_eq!(nw.referenced_indices.len(), 1);

        let ne = &nodes[2];
        assert_eq!(ne.referenced_indices.len(), 1);

        let sw = &nodes[3];
        assert_eq!(sw.referenced_indices.len(), 1);

        let se = &nodes[4];
        assert_eq!(se.referenced_indices.len(), 1);

        // Query range
        let boundary = SquareBox {
            center: [0.0, 0.0],
            half_size: 1000.0,
        };
        let result = quadtree.query_range(boundary, &bodies);
        assert_eq!(result.len(), bodies.len());

        let boundary = SquareBox {
            center: [0.5, 0.5],
            half_size: 0.001,
        };
        let result = quadtree.query_range(boundary, &bodies);
        assert_eq!(result.len(), 1);
    }
}
