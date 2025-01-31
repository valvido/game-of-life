#![allow(unused_imports)]
mod optimized_alg;
mod track_alive_cells;
mod parallelize;
mod parallel_alex;
mod hashlife;  // New Hashlife module

use hashlife::Universe as HashlifeUniverse;  // New import for Hashlife
use parallel_alex:: Universe as ParallelAlexUniverse;
use parallelize::Universe as ParallelUniverse;
use optimized_alg::Universe as OptimizedUniverse;
use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;
use track_alive_cells::Universe as TrackAliveCellsUniverse;


use std::time::Instant;
use rand::Rng;
use sysinfo::{System, SystemExt};
use std::env;
use std::convert::TryInto;

mod utils;
use utils::*;

fn main() {
    // File name of the grid
    let file_name = "blom.rle";
    let file_path = format!("../grids/{}", file_name);


    // Read RLE file and initialize the flat matrix
    let flat_matrix: Vec<u8> = init_from_file(&file_path).into_iter().map(|x| x as u8).collect();

    let grid_size = (flat_matrix.len() as f64).sqrt().floor() as usize;
    let width: u32 = grid_size.try_into().unwrap();
    let height: u32 = grid_size.try_into().unwrap();

    // Convert flat matrix to a 2D representation
    let initial_state = vec_to_matrix(&flat_matrix, grid_size);

    // Convert initial_state to a list of live cells (for Track-Alive-Cells & Parallelized version)
    let initial_live_cells: Vec<(usize, usize)> = initial_state
        .iter()
        .enumerate()
        .flat_map(|(row, cols)| {
            cols.iter().enumerate().filter_map(move |(col, &value)| {
                if value == 1 {
                    Some((row, col))
                } else {
                    None
                }
            })
        })
        .collect();

    // ===== Naive Implementation =====
    println!("Naive Game of Life:");
    let initial_cells: Vec<NaiveCell> = initial_state
        .iter()
        .flatten()
        .map(|&x| if x == 1 { NaiveCell::Alive } else { NaiveCell::Dead })
        .collect();

    let mut naive_universe = NaiveUniverse::new_with_cells(width, height, initial_cells);
    let start_naive = Instant::now();
    naive_universe.run_iterations(10);
    let naive_time = start_naive.elapsed().as_millis();
    println!("Naive Approach: {} ms", naive_time);

    // ===== Sparse Matrix Implementation =====
    println!("\nSparse Matrix Game of Life Algorithm:");

    // Convert `flat_matrix` from `Vec<u8>` to `Vec<usize>` before passing
    let flat_matrix_usize: Vec<usize> = flat_matrix.iter().map(|&x| x as usize).collect();

    let mut sparse_universe = SparseUniverse::new_with_matrix(width, height, flat_matrix_usize);
    let start_sparse = Instant::now();
    sparse_universe.run_iterations(10);
    let sparse_time = start_sparse.elapsed().as_millis();
    println!("Sparse-Matrix Approach: {} ms", sparse_time);

    // ===== Optimized Version =====
    println!("\nOptimized Game of Life Algorithm:");
    let flat_initial_state: Vec<u8> = flat_matrix.clone();
    let mut optimized_universe = OptimizedUniverse::new(width as usize, height as usize, flat_initial_state);
    let start_optimized = Instant::now();
    optimized_universe.run_iterations(10);
    let optimized_time = start_optimized.elapsed().as_millis();
    println!("Optimized Cache Algorithm: {} ms", optimized_time);

    // ===== Track-Alive-Cells Implementation =====
    println!("\nTrack-Alive-Cells Algorithm:");
    let mut track_alive_cells_universe = TrackAliveCellsUniverse::new(
        width as usize,
        height as usize,
        initial_live_cells.clone(), // Clone here to preserve for parallel version
    );

    let start_track_alive = Instant::now();
    track_alive_cells_universe.run_iterations(10);
    let track_alive_time = start_track_alive.elapsed().as_millis();
    println!("Track-Alive-Cells Approach: {} ms", track_alive_time);

    // ===== Parallelized Version =====
    println!("\nParallelized Game of Life:");
    let mut parallel_universe = ParallelUniverse::new(
        width as usize,
        height as usize,
        initial_live_cells, // Use original since it was cloned before
    );

    let start_parallel = Instant::now();
    parallel_universe.run_iterations(10);
    let parallel_time = start_parallel.elapsed().as_millis();
    println!("Parallelized Approach: {} ms", parallel_time);

    // ===== Alex's Parallelized Version =====
    println!("\nParallelized Game of Life:");
    let mut Alex_parallel_universe = ParallelAlexUniverse::new_with_matrix(width, height, flat_matrix.clone());
    let start_parallel_Alex = Instant::now();
    Alex_parallel_universe.run_iterations(10);
    let parallel_time_Alex = start_parallel_Alex.elapsed().as_millis();
    println!("Alex parallelized Approach: {} ms", parallel_time_Alex);

    // ===== Hashlife Implementation =====
    println!("\nHashlife Game of Life Algorithm:");
    let mut hashlife_universe = HashlifeUniverse::new_with_matrix(width, height, flat_matrix.clone());
    let start_hashlife = Instant::now();
    hashlife_universe.run_iterations(10);
    let hashlife_time = start_hashlife.elapsed().as_millis();
    println!("Hashlife Approach: {} ms", hashlife_time);    
}
