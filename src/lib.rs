extern crate cfg_if;
extern crate wasm_bindgen;
extern crate sysinfo; // Add sysinfo crate

pub mod sparse_matrix;
mod utils;

use cfg_if::cfg_if;
use sysinfo::{System, SystemExt}; // Import sysinfo
use wasm_bindgen::prelude::*;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::INIT;
    }
}

// Function to print memory usage
fn print_memory_usage(label: &str) {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let memory_used = sys.used_memory();
    println!("{} - Memory Usage: {} MB", label, memory_used / 1024);
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
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
    cells: Vec<Cell>, // Private field
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new_with_cells(width: u32, height: u32, cells: Vec<Cell>) -> Universe {
        assert_eq!(cells.len(), (width * height) as usize);
        Universe { width, height, cells }
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
        self.to_string()
    }

    // Getter method to expose the cells for rendering
    pub fn get_cells(&self) -> Vec<u8> {
        self.cells.iter().map(|&cell| cell as u8).collect()
    }

    // Additional method to expose width and height if needed
    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { "☁ " } else { "🦄" };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
