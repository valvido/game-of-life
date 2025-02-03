use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_bindgen::prelude::*;

/// Enum representing the state of a cell
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] // ✅ Ensure Cell implements Hash
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

/// Struct representing a quadrant-based Hashlife Node
#[derive(Clone, PartialEq, Eq, Debug)]
struct Node {
    size: usize,
    nw: Option<Box<Node>>,
    ne: Option<Box<Node>>,
    sw: Option<Box<Node>>,
    se: Option<Box<Node>>,
    center: Option<Cell>, // Central cell for base case
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.nw.as_ref().map(|n| n.size).hash(state);
        self.ne.as_ref().map(|n| n.size).hash(state);
        self.sw.as_ref().map(|n| n.size).hash(state);
        self.se.as_ref().map(|n| n.size).hash(state);
        self.center.hash(state);
    }
}

/// Universe struct for Hashlife implementation
#[wasm_bindgen]
pub struct Universe {
    cache: HashMap<Node, Node>,
    root: Node,
}

impl Universe {
    /// Creates a new universe from a flat matrix
    pub fn new_with_matrix(width: usize, height: usize, flat_matrix: Vec<u8>) -> Universe {
        let root = Universe::build_tree(width, height, &flat_matrix);
        Universe {
            cache: HashMap::new(),
            root,
        }
    }

    /// Recursively builds a quadtree from the flat matrix
    fn build_tree(width: usize, height: usize, flat_matrix: &[u8]) -> Node {
        if width == 0 || height == 0 || flat_matrix.is_empty() {
            return Node {
                size: 1,
                nw: None,
                ne: None,
                sw: None,
                se: None,
                center: Some(Cell::Dead), // Default to dead
            };
        }

        if width == 1 && height == 1 {
            let cell = if flat_matrix.first().copied().unwrap_or(0) == 1 { 
                Cell::Alive 
            } else { 
                Cell::Dead 
            };
            return Node {
                size: 1,
                nw: None,
                ne: None,
                sw: None,
                se: None,
                center: Some(cell),
            };
        }

        let half_width = width / 2;
        let half_height = height / 2;
        
        // Ensure there is enough data
        if flat_matrix.len() < width * height {
            return Node {
                size: width,
                nw: None,
                ne: None,
                sw: None,
                se: None,
                center: None, // Return minimal node
            };
        }

        // Create quadrants by extracting rows from the flat matrix
        let mut nw_matrix = Vec::new();
        let mut ne_matrix = Vec::new();
        let mut sw_matrix = Vec::new();
        let mut se_matrix = Vec::new();

        for row in 0..half_height {
            let start = row * width;
            let mid = start + half_width;
            let end = start + width;

            nw_matrix.extend_from_slice(&flat_matrix[start..mid]);  // Top-left
            ne_matrix.extend_from_slice(&flat_matrix[mid..end]);    // Top-right
        }

        for row in half_height..height {
            let start = row * width;
            let mid = start + half_width;
            let end = start + width;

            sw_matrix.extend_from_slice(&flat_matrix[start..mid]);  // Bottom-left
            se_matrix.extend_from_slice(&flat_matrix[mid..end]);    // Bottom-right
        }

        // Recursively build the quad-tree
        let nw = Universe::build_tree(half_width, half_height, &nw_matrix);
        let ne = Universe::build_tree(half_width, half_height, &ne_matrix);
        let sw = Universe::build_tree(half_width, half_height, &sw_matrix);
        let se = Universe::build_tree(half_width, half_height, &se_matrix);

        Node {
            size: width,
            nw: Some(Box::new(nw)),
            ne: Some(Box::new(ne)),
            sw: Some(Box::new(sw)),
            se: Some(Box::new(se)),
            center: None,
        }
    }

    /// Advances the universe by one tick using Hashlife
    pub fn tick(&mut self) {
        let new_root = self.compute_next(self.root.clone()); // ✅ FIX: Clone before passing
        self.root = new_root;
    }

    /// Computes the next state of the universe using memoized Hashlife
    fn compute_next(&mut self, node: Node) -> Node {
        if let Some(cached) = self.cache.get(&node) {
            return cached.clone();
        }

        if node.size == 1 {
            return Node {
                size: 1,
                nw: None,
                ne: None,
                sw: None,
                se: None,
                center: Some(match node.center {
                    Some(Cell::Alive) => Cell::Dead,
                    _ => Cell::Alive,
                }),
            };
        }

        let nw = self.compute_next(*node.nw.clone().unwrap());
        let ne = self.compute_next(*node.ne.clone().unwrap());
        let sw = self.compute_next(*node.sw.clone().unwrap());
        let se = self.compute_next(*node.se.clone().unwrap());

        let next_node = Node {
            size: node.size,
            nw: Some(Box::new(nw)),
            ne: Some(Box::new(ne)),
            sw: Some(Box::new(sw)),
            se: Some(Box::new(se)),
            center: None,
        };

        self.cache.insert(node, next_node.clone());
        next_node
    }

    /// Runs multiple iterations using Hashlife
    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }
}
