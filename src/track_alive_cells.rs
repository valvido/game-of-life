#![allow(dead_code)]
use crc32fast::Hasher;

//this version does not update the whole matrix (grid) but only keeps track of the part of the grid 
//which is alive and active

pub struct Universe {
    width: usize,
    height: usize,
    current: Vec<u8>,       // Flat representation of the grid
    next: Vec<u8>,          // Auxiliary grid for the next state
    active: Vec<bool>,      // Flat representation of active cells
}

impl Universe {
    /// Creates a new `Universe` with the specified dimensions and initial live cells.
    pub fn new(width: usize, height: usize, initial_live_cells: Vec<(usize, usize)>) -> Self {
        let mut current = vec![0; width * height];
        let mut active = vec![false; width * height];

        for &(row, col) in &initial_live_cells {
            let idx = row * width + col;
            current[idx] = 1;        // Mark the cell as alive
            active[idx] = true;      // Mark the cell as active
        }

        Self {
            width,
            height,
            current,
            next: vec![0; width * height],
            active,
        }
    }

   /// Advances the game by one tick.
pub fn tick(&mut self) {
    let mut new_active = vec![false; self.width * self.height];

    for row in 0..self.height {
        for col in 0..self.width {
            let idx = self.get_index(row, col);

            // Count live neighbors correctly
            let live_neighbors = self.count_live_neighbors(row, col);

            // Apply Game of Life rules correctly
            self.next[idx] = match (self.current[idx], live_neighbors) {
                (1, 2) | (1, 3) => 1, // Alive cell survives
                (0, 3) => 1,          // Dead cell becomes alive
                _ => 0,               // Otherwise, the cell dies
            };

            // If a cell is alive in the next state, mark itself and its neighbors as active
            if self.next[idx] == 1 {
                new_active[idx] = true;
                for &(dr, dc) in self.neighbor_deltas().iter() {
                    let neighbor_row = (row as isize + dr + self.height as isize) % self.height as isize;
                    let neighbor_col = (col as isize + dc + self.width as isize) % self.width as isize;
                    let neighbor_idx = self.get_index(neighbor_row as usize, neighbor_col as usize);
                    new_active[neighbor_idx] = true; // Track immediate neighbors
                }
            }
        }
    }

    // Swap grids and update active cells
    std::mem::swap(&mut self.current, &mut self.next);
    self.active = new_active;
}

/// Counts the number of live neighbors for a given cell.
fn count_live_neighbors(&self, row: usize, col: usize) -> u8 {
    let mut count = 0;

    for &(dr, dc) in self.neighbor_deltas().iter() {
        let neighbor_row = (row as isize + dr + self.height as isize) % self.height as isize;
        let neighbor_col = (col as isize + dc + self.width as isize) % self.width as isize;
        let neighbor_idx = self.get_index(neighbor_row as usize, neighbor_col as usize);
        
        count += self.current[neighbor_idx]; // Correctly sum up live neighbors
    }

    count
}



    /// Returns the precomputed neighbor offsets.
    fn neighbor_deltas(&self) -> &'static [(isize, isize)] {
        &[
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),          (0, 1),
            (1, -1), (1, 0), (1, 1),
        ]
    }

    /// Converts a 2D coordinate `(row, col)` into a 1D index for the flat grid.
    #[inline]
    fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    pub fn get_cells(&self) -> Vec<u8> {

        let cells = self.current.clone();
        cells.iter().map(|&cell| cell as u8).collect()
    }

    // Computes a CRC32 checksum to ensure correct evolution
    pub fn crc32(&self) -> u32 {
        let mut hasher = Hasher::new();
        let state = self.get_cells();
        hasher.update(&state);
        hasher.finalize()
    }

    /// Runs the game for the specified number of iterations (ticks).
    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    /// Renders the current grid state as a string.
    pub fn render(&self) -> String {
        let mut buffer = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                buffer.push(if self.current[self.get_index(row, col)] == 1 { '1' } else { '0' });
            }
            buffer.push('\n');
        }

        buffer
    }
}