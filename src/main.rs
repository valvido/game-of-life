#![allow(unused_imports)]
mod optimized_alg;
mod track_alive_cells;
mod parallelize;
mod hashed_parallel;
mod bitwise;

use hashed_parallel::Universe as HashParallelUniverse;
use parallelize::Universe as ParallelUniverse;
use optimized_alg::Universe as OptimizedUniverse;
use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;
use track_alive_cells::Universe as TrackAliveCellsUniverse;
use bitwise::Universe as BWUniverse;

use std::time::Instant;
use rand::Rng;
use sysinfo::{System, SystemExt};
use std::env;
use std::convert::TryInto;

mod utils;
use utils::*;

fn main() {
    // File name of the grid
    let file_name = "52513m.rle";
    let file_path = format!("./grids/{}", file_name);

    // Number of iterations:
    let n_iter: usize = 100;

    // Size of the universe:
    let scale = 2;
    let width = usize::pow(2, 9 + scale);

    // Read RLE file and initialize the flat matrix
    let flat_matrix: Vec<u8> = init_from_file(&file_path, width);

    // Convert flat matrix to a 2D representation
    let initial_state = vec_to_matrix(&flat_matrix, width);

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

    // Calculate initial number of live cells to avoid borrowing issues later:
    let alive_count = initial_live_cells.len() ;

    // ===== Naive Implementation =====
    println!("Naive Game of Life:");
    let initial_cells: Vec<NaiveCell> = initial_state
        .iter()
        .flatten()
        .map(|&x| if x == 1 { NaiveCell::Alive } else { NaiveCell::Dead })
        .collect();

    let mut naive_universe = NaiveUniverse::new_with_cells(width, width, initial_cells);
    let start_naive = Instant::now();
    naive_universe.run_iterations(n_iter);
    let naive_time = start_naive.elapsed().as_millis();
    println!("Naive Approach: {} ms", naive_time);

    // ===== Sparse Matrix Implementation =====
    println!("\nSparse Matrix Game of Life Algorithm:");

    // Convert `flat_matrix` from `Vec<u8>` to `Vec<usize>` before passing
    // let flat_matrix_usize: Vec<usize> = flat_matrix.iter().map(|&x| x as usize).collect();

    let mut sparse_universe = SparseUniverse::new_with_matrix(width,
         width, 
         flat_matrix.clone());
    let start_sparse = Instant::now();
    sparse_universe.run_iterations(n_iter);
    let sparse_time = start_sparse.elapsed().as_millis();
    println!("Sparse-Matrix Approach: {} ms", sparse_time);

    // ===== Optimized Version =====
    println!("\nOptimized Game of Life Algorithm:");
    let flat_initial_state: Vec<u8> = flat_matrix.clone();
    let mut optimized_universe = OptimizedUniverse::new(width, width, flat_initial_state);
    let start_optimized = Instant::now();
    optimized_universe.run_iterations(n_iter);
    let optimized_time = start_optimized.elapsed().as_millis();
    println!("Optimized Cache Algorithm: {} ms", optimized_time);

    // ===== Track-Alive-Cells Implementation =====
    println!("\nTrack-Alive-Cells Algorithm:");
    let mut track_alive_cells_universe = TrackAliveCellsUniverse::new(
        width,
        width,
        initial_live_cells.clone(), // Clone here to preserve for parallel version
    );

    let start_track_alive = Instant::now();
    track_alive_cells_universe.run_iterations(n_iter);
    let track_alive_time = start_track_alive.elapsed().as_millis();
    println!("Track-Alive-Cells Approach: {} ms", track_alive_time);

    // ===== Parallelized Version =====
    println!("\nParallelized Game of Life:");
    let mut parallel_universe = ParallelUniverse::new(
        width,
        width,
        initial_live_cells, // Use original since it was cloned before
    );

    let start_parallel = Instant::now();
    parallel_universe.run_iterations(n_iter);
    let parallel_time = start_parallel.elapsed().as_millis();
    println!("Parallelized Approach: {} ms", parallel_time);

    // ===== Alex's Parallelized Version =====
    println!("\nParallelized Game of Life:");
    let mut hashed_parallel_universe = HashParallelUniverse::new_with_matrix(width, width, flat_matrix);
    let start_hp = Instant::now();
    hashed_parallel_universe.run_iterations(n_iter);
    let total_time_hp = start_hp.elapsed().as_millis();
    println!("Hashed parallel Approach: {} ms", total_time_hp);

    // ===== Bitwise Implementation =====
    println!("\nBitwise Game of Life:");
    let mut bitwise_universe = BWUniverse::new(width, width, alive_count);
    let start_bitwise = Instant::now();
    bitwise_universe.run_iterations(10);
    let bitwise_time = start_bitwise.elapsed().as_millis();
    println!("Bitwise Approach: {} ms", bitwise_time);
}
