#![allow(dead_code)]
extern crate cfg_if;
extern crate wasm_bindgen;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use rayon::prelude::*; 
use std::collections::{HashSet, HashMap};
use crc32fast::Hasher;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Universe {
    width: usize,
    height: usize,
    live_cells: HashSet<(usize, usize)>,
}

// helper functions
impl Universe {
    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize,  usize)> {
        let deltas = [-1, 0, 1];
        let mut neighbors = Vec::new();

        for &delta_row in &deltas {
            for &delta_col in &deltas {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = ((self.height as i32 + row as i32 + delta_row)  % self.height as i32) as usize ;
                let neighbor_col = ((self.width as i32  + col as i32 + delta_col) % self.width as i32) as usize;
                neighbors.push((neighbor_row, neighbor_col));
            }
        }

        neighbors
    }

    fn rules(is_alive: bool, neighbor_count: usize) -> bool {
        match (is_alive, neighbor_count) {
            (true, 2) | (_, 3) => true, // Stays alive or comes to life
            _ => false, // Dies
        }
    }
}


// generation calculation
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let live_cells = &self.live_cells;

         // Count neighbors using parallel iteration
        let neighbor_counts: HashMap<(usize, usize), usize> = live_cells
            .par_iter()
            .flat_map(|&(row, col)| self.get_neighbors(row, col))
            .fold(HashMap::new,
                |mut acc, cell| {
                    *acc.entry(cell).or_insert(0) += 1;
                    acc
                },
            )
            .reduce(HashMap::new,
                |mut acc, map| {
                    for (k, v) in map {
                        *acc.entry(k).or_insert(0) += v;
                    }
                    acc
                },
            );
        
        // Compute next state in parallel
        let next_state: HashSet<(usize, usize)> = neighbor_counts
            .par_iter()
            .filter_map(|(&cell, &count)| {
                let is_alive = live_cells.contains(&cell);
                if Self::rules(is_alive, count) {
                    Some(cell)
                } else {
                    None
                }
            })
            .collect();

        self.live_cells = next_state;


        /*
        pub fn tick(&mut self) {
            let mut neighbor_counts: HashMap<(u32, u32), u32> = HashMap::new();
    
            // Count live neighbors for all cells
            for &(row, col) in &self.live_cells {
                for neighbor in self.get_neighbors(row, col) {
                    *neighbor_counts.entry(neighbor).or_insert(0) += 1;
                }
            }
    
            let mut next_live_cells = HashSet::new();
    
            // Apply rules based on neighbor counts
            for (cell, count) in neighbor_counts {
                if count == 3 || (count == 2 && self.live_cells.contains(&cell)) {
                    next_live_cells.insert(cell);
                }
            }
    
            self.live_cells = next_live_cells;
        }
        self.cells = next;
        */

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

    /// Runs the game for the specified number of iterations (ticks).
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