// Game of Life implementation with bitwise (1 cell = 1 bit) representation

use crc32fast::Hasher;


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<u8>, // Using Vec<u8> for bitwise implementation
}

impl Universe {
    fn get_index(&self, row: usize, column: usize) -> (usize, u8) {
        let bit_index = row * self.width + column;
        let byte_index = bit_index / 8;
        let bit_position = bit_index % 8;
        (byte_index, 1 << bit_position)
    }

    fn get_index_static(width: usize, row: usize, column: usize) -> (usize, u8) {
        let bit_index = row * width + column;
        let byte_index = bit_index / 8;
        let bit_position = bit_index % 8;
        (byte_index, 1 << bit_position)
    }

    fn live_neighbor_count(&self, row: usize, column: usize) -> usize {
        let mut count = 0;

        for delta_row in [-1, 0, 1].iter().cloned() {
            for delta_col in [-1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = ((row as isize + delta_row + self.height as isize) % self.height as isize) as usize;
                let neighbor_col = ((column as isize + delta_col + self.width as isize) % self.width as isize) as usize;

                let (byte_index, bit_mask) = self.get_index(neighbor_row, neighbor_col);
                if self.cells[byte_index] & bit_mask != 0 {
                    count += 1;
                }
            }
        }

        count
    }
}


impl Universe {
    pub fn new(width: usize, height: usize, flat_matrix: Vec<u8>) -> Universe {
        let num_bytes = (width * height + 7) / 8;
        let mut cells = vec![0u8; num_bytes];

        // Initialize from the flat matrix
        for (index, &value) in flat_matrix.iter().enumerate() {
            if value == Cell::Alive as u8 {
                let row = index / width;
                let col = index % width;
                let (byte_index, bit_mask) = Universe::get_index_static(width, row, col);
                cells[byte_index] |= bit_mask;
            }
        }

        Universe {
            width,
            height,
            cells,
        }
    }

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

    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn get_cells(&self) -> Vec<u8> {
        let mut univ: Vec<u8> = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                let (byte_index, bit_mask) = self.get_index(row, col);
                let cell = if self.cells[byte_index] & bit_mask != 0 {1} else {0};
                univ.push(cell);
            }
        }
        univ
    }

    // Computes a CRC32 checksum to ensure correct evolution
    pub fn crc32(&self ) -> u32 {
        let mut hasher = Hasher::new();
        let state = self.get_cells();
        hasher.update(&state);
        hasher.finalize()
    }

    pub fn render(&self) -> String {
        let mut result = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let (byte_index, bit_mask) = self.get_index(row, col);
                let symbol = if self.cells[byte_index] & bit_mask != 0 { "🦄" } else { "☁ " };
                result.push_str(symbol);
            }
            result.push('\n');
        }

        result
    }
}

impl std::fmt::Display for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}




