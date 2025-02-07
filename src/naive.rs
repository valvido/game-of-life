// This is the naive, sequential implementation of the Game of Life 


#![allow(dead_code)]

use sysinfo::{System, SystemExt}; // Import sysinfo
use crc32fast::Hasher;


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

// Universe is the grid of size width*height and implements a vector of cells
pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<Cell>, // Private field
}

impl Universe {
    fn get_index(&self, row: usize, column: usize) -> usize {
        row * self.width + column 
    }
    // number of neighbors is later relevant for computing the next state of the cell
    // the grid is treated as a torus, therefore edge rows and column have to adjust their neighboring row/col index to wrap around
    fn live_neighbor_count(&self, row: usize, column: usize) -> usize {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as usize;
            }
        }
        count
    }
}


impl Universe{
    // function to advance the universe by one generation
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                
                //applying the rules of the game
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

    pub fn new_with_cells(width: usize, height: usize, cells: Vec<Cell>) -> Universe {
        assert_eq!(cells.len(), width * height);
        Universe { width, height, cells }
    }

    // function to advance several generations at a time
    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    // Getter method to expose the cells for rendering
    pub fn get_cells(&self) -> Vec<u8> {
        self.cells.iter().map(|&cell| cell as u8).collect()
    }

    // Additional method to expose width and height if needed
    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    // Computes a CRC32 checksum to ensure correct evolution
    pub fn crc32(&self ) -> u32 {
        let mut hasher = Hasher::new();
        let state = self.get_cells();
        hasher.update(&state);
        hasher.finalize()
    }
}


use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { "☁ " } else { "🦄" };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}