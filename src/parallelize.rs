#![allow(dead_code)]
use rayon::prelude::*;
use crc32fast::Hasher;

pub struct Universe {
    width: usize,
    height: usize,
    current: Vec<u8>, // Flat representation of the grid
    next: Vec<u8>,    // Auxiliary grid for the next state
    active: Vec<bool>, // Flat representation of active cells
}

impl Universe {
    /// Creates a new `Universe` with the specified dimensions and initial live cells.
    pub fn new(width: usize, height: usize, initial_live_cells: Vec<(usize, usize)>) -> Self {
        let mut current = vec![0; width * height];
        let mut active = vec![false; width * height];

        for &(row, col) in &initial_live_cells {
            let idx = row * width + col;
            current[idx] = 1; // Mark the cell as alive
            active[idx] = true; // Mark the cell as active
        }

        Self {
            width,
            height,
            current,
            next: vec![0; width * height],
            active,
        }
    }

    /// Advances the game by one tick (parallelized).
    pub fn tick(&mut self) {
        let mut new_next = vec![0; self.width * self.height];
        let mut new_active = vec![false; self.width * self.height];

        // Process rows in parallel
        let updates: Vec<(usize, u8, bool)> = (0..self.height)
            .into_par_iter()
            .flat_map(|row| {
                let mut local_updates = Vec::new();

                for col in 0..self.width {
                    let idx = self.get_index(row, col);

                    // Skip inactive cells
                    if !self.active[idx] {
                        continue;
                    }

                    let live_neighbors = self.count_live_neighbors(row, col);

                    // Apply Game of Life rules
                    let next_state = match (self.current[idx], live_neighbors) {
                        (1, 2) | (1, 3) => 1, // Alive cell survives
                        (0, 3) => 1,          // Dead cell becomes alive
                        _ => 0,               // Otherwise, the cell dies
                    };

                    // Check if the cell changed
                    if next_state != self.current[idx] {
                        local_updates.push((idx, next_state, true));

                        // Mark neighbors as active
                        for &(dr, dc) in self.neighbor_deltas().iter() {
                            let neighbor_row = (row as isize + dr + self.height as isize) % self.height as isize;
                            let neighbor_col = (col as isize + dc + self.width as isize) % self.width as isize;
                            let neighbor_idx = self.get_index(neighbor_row as usize, neighbor_col as usize);
                            local_updates.push((neighbor_idx, self.next[neighbor_idx], true));
                        }
                    }
                }
                local_updates
            })
            .collect();

        // Merge updates sequentially to avoid data race
        for (idx, value, active) in updates {
            new_next[idx] = value;
            new_active[idx] = active;
        }

        // Swap grids and update active cells
        self.current = new_next;
        self.active = new_active;
    }

    /// Counts the number of live neighbors for a given cell.
    fn count_live_neighbors(&self, row: usize, col: usize) -> u8 {
        self.neighbor_deltas()
            .iter()
            .fold(0, |count, &(dr, dc)| {
                let neighbor_row = (row as isize + dr + self.height as isize) % self.height as isize;
                let neighbor_col = (col as isize + dc + self.width as isize) % self.width as isize;
                let neighbor_idx = self.get_index(neighbor_row as usize, neighbor_col as usize);
                count + self.current[neighbor_idx]
            })
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

    /// Runs the game for the specified number of iterations (ticks).
    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn get_cells(&self) -> Vec<u8> {

        let cells = self.active.clone();
        cells.iter().map(|&cell| cell as u8).collect()
    }

    // Computes a CRC32 checksum to ensure correct evolution
    pub fn crc32(&self ) -> u32 {
        let mut hasher = Hasher::new();
        let state = self.get_cells();
        hasher.update(&state);
        hasher.finalize()
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


