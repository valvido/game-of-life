use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};
extern crate sysinfo;
use sysinfo::{System, SystemExt}; 

// Function to print memory usage
fn print_memory_usage(label: &str) {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let memory_used = sys.used_memory();
    println!("{} - Memory Usage: {} MB", label, memory_used / 1024);
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
                let neighbor_row = ((row as i32 + delta_row + self.height as i32) % self.height as i32) as u32;
                let neighbor_col = ((col as i32 + delta_col + self.width as i32) % self.width as i32) as u32;
                neighbors.push((neighbor_row, neighbor_col));
            }
        }

        neighbors
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut neighbor_counts: HashMap<(u32, u32), u32> = HashMap::new();

        for &(row, col) in &self.live_cells {
            for neighbor in self.get_neighbors(row, col) {
                *neighbor_counts.entry(neighbor).or_insert(0) += 1;
            }
        }

        let mut next_live_cells = HashSet::new();

        for (cell, count) in neighbor_counts {
            if count == 3 || (count == 2 && self.live_cells.contains(&cell)) {
                next_live_cells.insert(cell);
            }
        }

        self.live_cells = next_live_cells;
    }

    pub fn new_with_matrix(width: u32, height: u32, flat_matrix: Vec<usize>) -> Universe {
        let mut live_cells = HashSet::new();

        for (index, &value) in flat_matrix.iter().enumerate() {
            if value == 1 {
                let row = index as u32 / width;
                let col = index as u32 % width;
                live_cells.insert((row, col));
            }
        }

        Universe {
            width,
            height,
            live_cells,
        }
    }

    pub fn run_iterations(&mut self, iterations: usize) {

        print_memory_usage("Before Running Iterations");

        for i in 0..iterations {
            self.tick();

            // Print memory usage every 5 iterations for better tracking
            if i % 5 == 0 {
                print_memory_usage(&format!("During Iteration {}", i));
            }
        }

        print_memory_usage("After Running Iterations");
    }

    pub fn render(&self) -> String {
        let mut buffer = String::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if self.live_cells.contains(&(row, col)) {
                    buffer.push('■');
                } else {
                    buffer.push('□');
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
