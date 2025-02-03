#![allow(dead_code)]
use crc32fast::Hasher;

//this is still a sequential algorithm but it has some optimizations for cache eficiency
pub struct Universe {
    width: usize,           // The width of the grid (number of columns)
    height: usize,          // The height of the grid (number of rows)
    current: Vec<u8>,       // Flat representation of the grid's current state; 0 for dead, 1 for alive
    next: Vec<u8>,          // Flat representation of the grid's next state
}

impl Universe {
    /// Creates a new `Universe` instance with the given width, height, and initial state.
    /// The `initial_state` must have a length equal to `width * height`.
    pub fn new(width: usize, height: usize, initial_state: Vec<u8>) -> Self {
        assert_eq!(initial_state.len(), width * height); // Ensure the initial state matches the grid size.
        Self {
            width,
            height,
            current: initial_state,           // Set the current grid to the provided initial state.
            next: vec![0; width * height],    // Initialize the next grid with all cells dead (0).
        }
    }

    /// Advances the game by one tick (generation).
    /// Updates the `next` grid based on the current state and swaps the grids at the end.
    pub fn tick(&mut self) {
        // Define neighbor relative positions for a cell, including diagonals.
        let deltas = [
            (-1, -1), (-1, 0), (-1, 1), // Top row neighbors
            (0, -1),          (0, 1),  // Middle row neighbors (left and right)
            (1, -1), (1, 0), (1, 1),   // Bottom row neighbors
        ];

        // Iterate over each cell in the grid.
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col); // Calculate the 1D index for the cell.
                let live_neighbors = self.count_live_neighbors(row, col, &deltas); // Count the live neighbors.

                // Apply the Game of Life rules to determine the next state of the cell.
                self.next[idx] = match (self.current[idx], live_neighbors) {
                    (1, 2) | (1, 3) => 1, // A live cell with 2 or 3 live neighbors stays alive.
                    (0, 3) => 1,          // A dead cell with exactly 3 live neighbors becomes alive.
                    _ => 0,               // Otherwise, the cell dies or remains dead.
                };
            }
        }

        // Swap the current and next grids to avoid copying data.
        std::mem::swap(&mut self.current, &mut self.next);
    }

    /// Counts the number of live neighbors for a given cell at `(row, col)`.
    fn count_live_neighbors(&self, row: usize, col: usize, deltas: &[(i32, i32)]) -> u8 {
        let mut count = 0;

        // Iterate through all neighbor positions defined by `deltas`.
        for &(dr, dc) in deltas {
            // Calculate the wrapped row and column positions (toroidal wrapping).
            let neighbor_row = (row as isize + dr as isize + self.height as isize) % self.height as isize;
            let neighbor_col = (col as isize + dc as isize + self.width as isize) % self.width as isize;

            // Convert the wrapped row and column back to 1D index.
            let idx = self.get_index(neighbor_row as usize, neighbor_col as usize);

            // Add the value of the neighbor cell to the count (0 or 1).
            count += self.current[idx];
        }

        count
    }

    /// Converts a 2D coordinate `(row, col)` into a 1D index for the flat grid representation.
    fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col // Calculate the 1D index using row-major order.
    }

    /// Runs the game for the specified number of iterations (ticks).
    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick(); // Advance the game by one tick for each iteration.
        }
    }

    // Computes a CRC32 checksum to ensure correct evolution
    pub fn crc32(&self ) -> u32 {
        let mut hasher = Hasher::new();
        let state = &self.current;
        hasher.update(state);
        hasher.finalize()
    }

    /// Renders the current grid state as a string, where:
    /// '1' represents a live cell, and '0' represents a dead cell.
    pub fn render(&self) -> String {
        let mut buffer = String::new();

        // Iterate over each row of the grid.
        for row in 0..self.height {
            for col in 0..self.width {
                // Append '1' or '0' based on the cell's current state.
                buffer.push(if self.current[self.get_index(row, col)] == 1 { '1' } else { '0' });
            }
            buffer.push('\n'); // Add a newline character at the end of each row.
        }

        buffer
    }
}

