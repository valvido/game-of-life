
extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    live_cells: HashSet<(u32, u32)>,
}


impl Universe {
    fn get_neighbors(&self, row: u32, col: u32) -> Vec<(u32, u32)> {
        let deltas = [-1, 0, 1];
        let mut neighbors = Vec::new();

        for &delta_row in &deltas {
            for &delta_col in &deltas {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (self.height + row + delta_row as u32) % self.height;
                let neighbor_col = (self.width + col + delta_col as u32) % self.width;
                neighbors.push((neighbor_row, neighbor_col));
            }
        }

        neighbors
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        pub fn tick(&mut self) {
            let mut neighbor_counts: HashMap<(u32, u32), u32> = HashMap::new();
    
            // Count live neighbors for all cells
            for &(row, col) in &self.live_cells {
                for neighbor in self.get_neighbors(row, col) {
                    *neighbor_counts.entry(neighbor).or_insert(0) += 1;
                }
            }
    
            let mut next_live_cells = HashSet::new();
    
            // Apply rules based on neighbor counts
            for (cell, count) in neighbor_counts {
                if count == 3 || (count == 2 && self.live_cells.contains(&cell)) {
                    next_live_cells.insert(cell);
                }
            }
    
            self.live_cells = next_live_cells;
        }
        self.cells = next;
    }

    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        // Initialize with some live cells
        let mut live_cells = HashSet::new();
        for row in 0..height {
            for col in 0..width {
                if (row + col) % 7 == 0 {
                    live_cells.insert((row, col));
                }
            }
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        let mut buffer = String::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if self.live_cells.contains(&(row, col)) {
                    buffer.push('◼');
                } else {
                    buffer.push('◻');
                }
            }
            buffer.push('\n');
        }
        buffer
    }
}


use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}