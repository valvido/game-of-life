extern crate cfg_if;
extern crate wasm_bindgen;
extern crate rand; // For random number generation

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use rand::Rng; // For random number generation

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::INIT;
    }
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
    width: usize,
    height: usize,
    cells: Vec<u8>, // Use Vec<u8> instead of Vec<u64> (bitwise implementation)
}

impl Universe {
    fn get_index(&self, row: usize, column: usize) -> (usize, u8) {
        let bit_index = row * self.width + column;
        let byte_index = bit_index / 8; // Index in the vector (each byte represents 8 cells)
        let bit_position = bit_index % 8; // Position in the byte
        (byte_index, 1 << bit_position)
    }

    fn live_neighbor_count(&self, row: usize, column: usize) -> usize {
        let mut count = 0;

        // Use isize for delta_row and delta_col for handling negative values
        for delta_row in [-1, 0, 1].iter().cloned() {
            for delta_col in [-1, 0, 1].iter().cloned() {
                // Skip the current cell
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                // Wrap around grid indices
                let neighbor_row = ((row as isize + delta_row) + self.height as isize) % self.height as isize;
                let neighbor_col = ((column as isize + delta_col) + self.width as isize) % self.width as isize;

                // Convert back to usize for indexing
                let neighbor_row = neighbor_row as usize;
                let neighbor_col = neighbor_col as usize;

                let (byte_index, bit_mask) = self.get_index(neighbor_row, neighbor_col);

                if self.cells[byte_index] & bit_mask != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    fn randomize_alive_cells(&mut self, alive_count: usize) {
        let mut rng = rand::thread_rng();
        let mut count = 0;

        while count < alive_count {
            let row = rng.gen_range(0..self.height);
            let col = rng.gen_range(0..self.width);
            let (byte_index, bit_mask) = self.get_index(row, col);

            if self.cells[byte_index] & bit_mask == 0 {
                self.cells[byte_index] |= bit_mask;
                count += 1;
            }
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = vec![0u8; self.cells.len()];

        for row in 0..self.height {
            for col in 0..self.width {
                let (byte_index, bit_mask) = self.get_index(row, col);
                let live_neighbors = self.live_neighbor_count(row, col);
                let is_alive = self.cells[byte_index] & bit_mask != 0;

                let next_state = match (is_alive, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    _ => is_alive,
                };

                if next_state {
                    next[byte_index] |= bit_mask;
                }
            }
        }

        self.cells = next;
    }

    pub fn new(width: usize, height: usize, alive_count: usize) -> Universe {
        let num_bytes = (width * height + 7) / 8; // Number of bytes needed
        let mut universe = Universe {
            width,
            height,
            cells: vec![0; num_bytes],
        };

        universe.randomize_alive_cells(alive_count);

        universe
    }

    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn render(&self) -> String {
        let mut result = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let (byte_index, bit_mask) = self.get_index(row, col);
                let symbol = if self.cells[byte_index] & bit_mask != 0 { '■' } else { '□' };
                result.push(symbol);
            }
            result.push('\n');
        }

        result
    }
}
