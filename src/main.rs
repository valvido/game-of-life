#![allow(unused_imports)]
mod optimized_alg;
mod track_alive_cells;
mod parallelize;
mod traits;  // This declares the traits module
mod hashed_parallel;
mod bitwise;

use crate::traits::TickUniv;
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
use std::fs::File;
use std::io::{self, Write};
use csv::Writer;
use std::fs;

mod utils;
use utils::*;

enum AnyUniverse {
    Naive(NaiveUniverse),
    Sparse(SparseUniverse),
    Optimized(OptimizedUniverse),
    TrackAliveCells(TrackAliveCellsUniverse),
    Parallel(ParallelUniverse),
    HashParallel(HashParallelUniverse),
}

fn initialize_all(flat_matrix: Vec<u8>, width: usize, height: usize) -> (
    NaiveUniverse,
    SparseUniverse,
    OptimizedUniverse,
    TrackAliveCellsUniverse,
    ParallelUniverse,
    HashParallelUniverse,
) {
      // NAIVE
    // Convert flat matrix to 2D representation
    let initial_state = vec_to_matrix(&flat_matrix, width);
    let initial_structcells: Vec<NaiveCell> = initial_state
        .iter()
        .flatten()
        .map(|&x| if x == 1 { NaiveCell::Alive } else { NaiveCell::Dead })
        .collect();
    let naive_universe = NaiveUniverse::new_with_cells(width, height, initial_structcells);

    // SPARSE
    let sparse_universe = SparseUniverse::new_with_matrix(width, height, flat_matrix.clone());

    // OPTIMIZED
    let optimized_universe = OptimizedUniverse::new(width, height, flat_matrix.clone());

    // TRACK
    let initial_trackparl_cells: Vec<(usize, usize)> = initial_state
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

    let track_alive_cells_universe = TrackAliveCellsUniverse::new(
        width,
        height,
        initial_trackparl_cells.clone(), // Clone here to preserve for parallel version
    );

    // PARALLEL
    let parallel_universe = ParallelUniverse::new(width, height, initial_trackparl_cells.clone());

    //PARALLEL ALEX
    let hashed_parallel_universe = HashParallelUniverse::new_with_matrix(width, height, flat_matrix.clone());

    (naive_universe, sparse_universe, optimized_universe, track_alive_cells_universe, parallel_universe, hashed_parallel_universe)
}



fn get_memory_usage() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.used_memory() // Returns memory usage in KB
}

fn gather_iteration_info(universe: &mut AnyUniverse, iterations: usize) -> (u128, Vec<u128>, Vec<u64>) {
    
    let mut iteration_times = Vec::new();
    let mut memory_use = Vec::new();
    memory_use.push(get_memory_usage()/1024);
    let global_start = Instant::now(); 

    match universe {
        AnyUniverse::Naive(u) => u.run_iterations(iterations),
        AnyUniverse::Sparse(u) => u.run_iterations(iterations),
        AnyUniverse::Optimized(u) => u.run_iterations(iterations),
        AnyUniverse::TrackAliveCells(u) => u.run_iterations(iterations),
        AnyUniverse::Parallel(u) => u.run_iterations(iterations),
        AnyUniverse::HashParallel(u) => u.run_iterations(iterations),
    }
    let global_time = global_start.elapsed().as_millis(); // Total elapsed time

    let mut iter_start = Instant::now();
    //measuring every 10 iterations
    for i in 0..iterations {
        if i % 10 == 0 {
            iter_start = Instant::now();
            // Match on the enum and call the corresponding tick() method
            match universe {
                AnyUniverse::Naive(u) => u.tick(),
                AnyUniverse::Sparse(u) => u.tick(),
                AnyUniverse::Optimized(u) => u.tick(),
                AnyUniverse::TrackAliveCells(u) => u.tick(),
                AnyUniverse::Parallel(u) => u.tick(),
                AnyUniverse::HashParallel(u) => u.tick(),
            }
            //memory_use.push(get_memory_usage()/1024);
        } else if i % 10 == 9 {
            match universe {
                AnyUniverse::Naive(u) => u.tick(),
                AnyUniverse::Sparse(u) => u.tick(),
                AnyUniverse::Optimized(u) => u.tick(),
                AnyUniverse::TrackAliveCells(u) => u.tick(),
                AnyUniverse::Parallel(u) => u.tick(),
                AnyUniverse::HashParallel(u) => u.tick(),
            }
            let iter_time = iter_start.elapsed().as_millis();
            iteration_times.push(iter_time);
        } else {
            match universe {
                AnyUniverse::Naive(u) => u.tick(),
                AnyUniverse::Sparse(u) => u.tick(),
                AnyUniverse::Optimized(u) => u.tick(),
                AnyUniverse::TrackAliveCells(u) => u.tick(),
                AnyUniverse::Parallel(u) => u.tick(),
                AnyUniverse::HashParallel(u) => u.tick(),
            }
        }
    }
    memory_use.push(get_memory_usage()/1024);
    
    (global_time, iteration_times, memory_use)
}

fn write_results_to_csv(results: &Vec<(String, u128, Vec<u128>, Vec<u64>)>, filename: &str, grid_size: (usize, usize), iterations: usize, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir_name = "results_csv";
    fs::create_dir_all(dir_name)?;
    let file_path = format!("{}/{}", dir_name, filename);
    let mut wtr = Writer::from_path(file_path)?;

    // Write metadata as the first row
    wtr.write_record(&[
        &format!("File Name: {}", file_name),  // This is the file name of the input file (e.g., "justyna.rle")  // Grid Size in width x height format
        &format!(" Width: {}", grid_size.0.to_string()),  // Width
        &format!(" Height: {}", grid_size.1.to_string()),  // Height
        &format!(" no. Iterations: {}", iterations.to_string()),  // Iterations
    ])?;
    // Write the headers
    wtr.write_record(&["Name", "Global Time (ms)", "Times per 10 Iterations", "Memory Usage before and after (MB)"])?;

    for result in results {
        let name = &result.0;
        let global_time = result.1;
        let iteration_times = format!("{:?}", result.2);  // Convert Vec to string
        let memory_use = format!("{:?}", result.3);

        // Write each row in the CSV
        wtr.write_record(&[name, &global_time.to_string(), &iteration_times, &memory_use.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}



fn main() {
    // File name of the grid
    let file_name = "52513m.rle";
    let file_path = format!("./grids/{}", file_name);
    // Number of iterations:
    let iterations: usize = 100;

    // Size of the universe:
    let scale = 2;
    let width = usize::pow(2, 9 + scale);

    // Read RLE file and initialize the flat matrix
    let flat_matrix: Vec<u8> = init_from_file(&file_path, width).into_iter().map(|x| x as u8).collect();

    // --- Initialization ---
    let ( naive_universe,  sparse_universe,  optimized_universe,  
        track_alive_cells_universe,  parallel_universe, hashed_parallel_universe) = initialize_all(flat_matrix, width, width);
    let mut initial_universes: Vec<AnyUniverse> = vec![
        AnyUniverse::Naive(naive_universe),
        AnyUniverse::Sparse(sparse_universe),
        AnyUniverse::Optimized(optimized_universe),
        AnyUniverse::TrackAliveCells(track_alive_cells_universe),
        AnyUniverse::Parallel(parallel_universe),
        AnyUniverse::HashParallel(hashed_parallel_universe),
    ];
    let universe_names = vec![
        "Naive", 
        "Sparse", 
        "Optimized", 
        "TrackAliveCells", 
        "Parallel", 
        "HashParallel"
    ];

    // --- Result Printing ---
    let mut results = Vec::new();

    for (i, univ) in initial_universes.iter_mut().enumerate() {
        let (global_time, iteration_times, memory_use) = gather_iteration_info(univ, iterations);
        let name = universe_names[i];  // Get the name based on index


        // Add the result to the results vector
        results.push((name.to_string(), global_time, iteration_times.clone(), memory_use.clone()));

        // Print results
        println!("{}: \nGlobal time: {} ms, Time for 5 iterations: {:?}, Memory use in MB: {:?}", 
                 name, global_time, iteration_times, memory_use);
        println!();  // Empty line after each result
    }

    let output_file_name = format!("{}_{}_results.csv", file_name, iterations);  // Use the existing `file_name` variable
    if let Err(e) = write_results_to_csv(&results, &output_file_name, (width, width), iterations, file_name) {
        eprintln!("Error writing to CSV file: {}", e);
    }
}
