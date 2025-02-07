// This is a sequential implementation using sparse matrices to store the alive cells 

#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use sysinfo::{System, SystemExt}; 
use crc32fast::Hasher;


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

// here the Universe implements the cells in a Hashset, only storing the alive cells 
pub struct Universe {
    width: usize,
    height: usize,
    live_cells: HashSet<(usize, usize)>,
}

impl Universe {
    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let deltas: [isize; 3] = [-1, 0, 1];
        let mut neighbors = Vec::new();

        for &delta_row in &deltas {
            for &delta_col in &deltas {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = ((row as isize + delta_row + self.height as isize) % self.height as isize) as usize;
                let neighbor_col = ((col as isize + delta_col + self.width as isize) % self.width as isize) as usize;
                neighbors.push((neighbor_row, neighbor_col));
            }
        }
        neighbors
    }
}

impl Universe {
    // function to advance the universe by one generation
    pub fn tick(&mut self) {
        let mut neighbor_counts: HashMap<(usize, usize), usize> = HashMap::new();

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

    pub fn new_with_matrix(width: usize, height: usize, flat_matrix: Vec<u8>) -> Universe {
        let mut live_cells = HashSet::new();

        for (index, &value) in flat_matrix.iter().enumerate() {
            if value == 1 {
                let row = index / width;
                let col = index % width;
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

        for _ in 0..iterations {
            self.tick();
        }
    }

    pub fn get_cells(&self) -> Vec<u8> {
        let mut univ: Vec<u8> = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if self.live_cells.contains(&(row, col)) {
                    univ.push(1);
                } else {
                    univ.push(0);
                }
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

