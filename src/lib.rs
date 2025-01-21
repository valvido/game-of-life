extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

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
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<u64>, // Each u64 represents 64 cells
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> (usize, u64) {
        let bit_index = row * self.width + column;
        let word_index = (bit_index / 64) as usize; // Index in the vector
        let bit_position = bit_index % 64;         // Position in the u64
        (word_index, 1 << bit_position)
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
                let (word_index, bit_mask) = self.get_index(neighbor_row, neighbor_col);

                if self.cells[word_index] & bit_mask != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    fn randomize_alive_cells(&mut self, alive_count: u32) {
        let mut rng = rand::thread_rng();
        let mut count = 0;

        while count < alive_count {
            let row = rng.gen_range(0..self.height);
            let col = rng.gen_range(0..self.width);
            let (word_index, bit_mask) = self.get_index(row, col);

            if self.cells[word_index] & bit_mask == 0 {
                self.cells[word_index] |= bit_mask;
                count += 1;
            }
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = vec![0u64; self.cells.len()];

        for row in 0..self.height {
            for col in 0..self.width {
                let (word_index, bit_mask) = self.get_index(row, col);
                let live_neighbors = self.live_neighbor_count(row, col);
                let is_alive = self.cells[word_index] & bit_mask != 0;

                let next_state = match (is_alive, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    _ => is_alive,
                };

                if next_state {
                    next[word_index] |= bit_mask;
                }
            }
        }

        self.cells = next;
    }

    pub fn new(width: u32, height: u32, alive_count: u32) -> Universe {
        let num_words = ((width * height) as usize + 63) / 64; // Number of u64 words needed
        let mut universe = Universe {
            width,
            height,
            cells: vec![0; num_words],
        };

        universe.randomize_alive_cells(alive_count);

        universe
    }

    pub fn run_iterations(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn render(&self) -> String {
        let mut result = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let (word_index, bit_mask) = self.get_index(row, col);
                let symbol = if self.cells[word_index] & bit_mask != 0 { "🦄" } else { "☁ " };
                result.push_str(symbol);
            }
            result.push('\n');
        }

        result
    }
}