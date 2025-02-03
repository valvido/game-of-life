#![allow(unused_imports)]
mod optimized_alg;
mod track_alive_cells;
mod parallelize;
mod traits;  // This declares the traits module
mod hashed_parallel;
mod bitwise;

use crate::traits::TickUniv;
use hashed_parallel::Universe as HashParallelUniverse;

mod hashlife;  // New Hashlife module

use hashlife::Universe as HashlifeUniverse;  // New import for Hashlife
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
    Bitwise(BWUniverse),
    Hashlife(HashlifeUniverse)
}

fn initialize_all(flat_matrix: Vec<u8>, width: usize, height: usize) -> (
    NaiveUniverse,
    SparseUniverse,
    OptimizedUniverse,
    TrackAliveCellsUniverse,
    ParallelUniverse,
    HashParallelUniverse,
    BWUniverse,
    HashlifeUniverse
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

    //PARALLEL WITH HASHING
    let hashed_parallel_universe = HashParallelUniverse::new_with_matrix(width, height, flat_matrix.clone());

    // BITWISE 
    let bitwise_universe = BWUniverse::new(width, height, flat_matrix.clone());

    // HASHLIFE
    let hashlife_universe = HashlifeUniverse::new_with_matrix(width, height, flat_matrix.clone());

    (naive_universe, sparse_universe, optimized_universe, track_alive_cells_universe, 
        parallel_universe, hashed_parallel_universe, bitwise_universe, hashlife_universe)
}

fn get_memory_usage() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.used_memory() // Returns memory usage in KB
}

// Advances one time step for any possible impl
fn global_ticker(universe:&mut AnyUniverse){

    // Match on the enum and call the corresponding tick() method
    match universe {
        AnyUniverse::Naive(u) => u.tick(),
        AnyUniverse::Sparse(u) => u.tick(),
        AnyUniverse::Optimized(u) => u.tick(),
        AnyUniverse::TrackAliveCells(u) => u.tick(),
        AnyUniverse::Parallel(u) => u.tick(),
        AnyUniverse::HashParallel(u) => u.tick(),
        AnyUniverse::Bitwise(u) => u.tick(),
        AnyUniverse::Hashlife(u) => u.tick(),
    }
}

fn gather_iteration_info(universe: &mut AnyUniverse, iterations: usize) -> (u128, Vec<u128>, Vec<u64>) {
    
    let mut iteration_times = Vec::new();
    let mut memory_use = Vec::new();
    memory_use.push(get_memory_usage()/1024);

    /* let global_start = Instant::now(); 

    match universe {
        AnyUniverse::Naive(u) => u.run_iterations(iterations),
        AnyUniverse::Sparse(u) => u.run_iterations(iterations),
        AnyUniverse::Optimized(u) => u.run_iterations(iterations),
        AnyUniverse::TrackAliveCells(u) => u.run_iterations(iterations),
        AnyUniverse::Parallel(u) => u.run_iterations(iterations),
        AnyUniverse::HashParallel(u) => u.run_iterations(iterations),
        AnyUniverse::Bitwise(u) => u.run_iterations(iterations),
    }
    let global_time = global_start.elapsed().as_millis(); // Total elapsed time
 */
    let global_start = Instant::now();
    let mut iter_start = Instant::now();

    // measuring every 10 iterations
    for i in 0..iterations {
        if i % 10 == 0 {
            // Start the clock
            iter_start = Instant::now();
            global_ticker(universe);
            //memory_use.push(get_memory_usage()/1024);

        } else if i % 10 == 9 {

            global_ticker(universe);
            // Record time per 10 interations
            let iter_time = iter_start.elapsed().as_millis();
            iteration_times.push(iter_time);
            
        } else {
            global_ticker(universe);
        }
    }
    let global_time = global_start.elapsed().as_millis(); // Total elapsed time
    memory_use.push(get_memory_usage()/1024);
    
    (global_time, iteration_times, memory_use)
}


fn write_results_to_csv(
    all_results: &Vec<Vec<(usize, String, u128, Vec<u128>, Vec<u64>)>>, 
    filename: &str,  
    iterations: usize, 
    file_name: &str) -> Result<(), Box<dyn std::error::Error>> {

    let dir_name = "results_csv";
    fs::create_dir_all(dir_name)?;
    let file_path = format!("{}/{}", dir_name, filename);
    let mut wtr = Writer::from_path(file_path)?;

    // Write metadata as the first row
    wtr.write_record([
        &format!("File Name: {}", file_name),  // This is the file name of the input file (e.g., "justyna.rle") 
        //&format!(" Width: {}", grid_size.0),  // Width
        //&format!(" Height: {}", grid_size.1),  // Height
        &format!(" No. Iterations: {}", iterations), 
        "", "", "" // Iterations
    ])?;
    // Write the headers
    wtr.write_record(["Grid size", "Name", "Global Time (ms)", "Times per 10 Iterations", "Memory Usage before and after (MB)"])?;

    for results in all_results{
        for version_result in results {
            let grid_size = version_result.0;
            let name = &version_result.1;
            let global_time = version_result.2;
            let iteration_times = format!("{:?}", version_result.3);  // Convert Vec to string
            let memory_use = format!("{:?}", version_result.4);

            // Write each row in the CSV
            wtr.write_record([&grid_size.to_string(), name, &global_time.to_string(), &iteration_times, &memory_use.to_string()])?;
        }
    }

    wtr.flush()?;
    Ok(())
}
    


fn main() {
    // File name of the grid
    let file_name = "blom.rle";
    let file_path = format!("./grids/{}", file_name);
    // Number of iterations:
    let iterations: usize = 1000;

    


    // ==== looping over grid sizes === 
    let scales = vec![3];
    let mut all_results = Vec::new();

    for &scale in &scales {
        // Size of the universe:
        let width = usize::pow(2, 6 + scale);
        // Read RLE file and initialize the flat matrix
        let flat_matrix: Vec<u8> = init_from_file(&file_path, width);

        // --- Initialization ---
        let (naive_universe,  sparse_universe,  optimized_universe, track_alive_cells_universe, parallel_universe, 
            hashed_parallel_universe, bitwise_universe, hashlife_universe) = initialize_all(flat_matrix, width, width);
        let mut initial_universes: Vec<AnyUniverse> = vec![
            AnyUniverse::Naive(naive_universe),
            AnyUniverse::Sparse(sparse_universe),
            AnyUniverse::Optimized(optimized_universe),
            AnyUniverse::TrackAliveCells(track_alive_cells_universe),
            AnyUniverse::Parallel(parallel_universe),
            AnyUniverse::HashParallel(hashed_parallel_universe),
            AnyUniverse::Bitwise(bitwise_universe),
            AnyUniverse::Hashlife(hashlife_universe)
        ];
        let universe_names = [
            "Naive", 
            "Sparse", 
            "Optimized", 
            "TrackAliveCells", 
            "Parallel", 
            "HashParallel",
            "Bitwise",
            "Hashlife"
        ];

        // --- Result Printing ---
        let mut version_results = Vec::new();

        for (i, univ) in initial_universes.iter_mut().enumerate() {
            let (global_time, iteration_times, memory_use) = gather_iteration_info(univ, iterations);
            let name = universe_names[i];  // Get the name based on index


            // Add the result to the results vector
            version_results.push((width, name.to_string(), global_time, iteration_times.clone(), memory_use.clone()));

            // Print results
            println!("Done: {} {}: ", width, name);
            //println!();  // Empty line after each result
        }
        all_results.push(version_results);
        
    }

    let output_file_name = format!("{}_{}_{:?}_results.csv", file_name, iterations, scales.last().unwrap());  // Use the existing `file_name` variable

    if let Err(e) = write_results_to_csv(&all_results, &output_file_name, iterations, file_name) {
        eprintln!("Error writing to CSV file: {}", e);
    }
}




