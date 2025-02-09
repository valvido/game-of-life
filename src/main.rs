// Main function for testing and running all the universe versions 
#![allow(unused_imports, dead_code)]
mod optimized_alg;
mod track_alive_cells;
mod hashed_parallel;
mod bitwise;
mod sparse_matrix;
mod naive;
mod hashlife; 

use hashed_parallel::Universe as HashParallelUniverse;
use hashlife::Universe as HashlifeUniverse;  
use optimized_alg::Universe as OptimizedUniverse;
use naive::{Universe as NaiveUniverse, Cell as NaiveCell};
use sparse_matrix::Universe as SparseUniverse;
use track_alive_cells::Universe as TrackAliveCellsUniverse;
use bitwise::Universe as BWUniverse;

use std::fmt::format;
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

// creating an enum with all possible universes to integrate them in general functions
enum AnyUniverse {
    Naive(NaiveUniverse),
    Sparse(SparseUniverse),
    Optimized(OptimizedUniverse),
    TrackAliveCells(TrackAliveCellsUniverse),
    HashParallel(HashParallelUniverse),
    Bitwise(BWUniverse),
    Hashlife(HashlifeUniverse)
}


// function to initialize all universes to the same given grid 
fn initialize_all(flat_matrix: Vec<u8>, width: usize, height: usize) -> (
    NaiveUniverse,
    SparseUniverse,
    OptimizedUniverse,
    TrackAliveCellsUniverse,
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

    // CACHE OPTIMIZED
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
        initial_trackparl_cells.clone(), 
    );

    //PARALLEL WITH HASHING
    let hashed_parallel_universe = HashParallelUniverse::new_with_matrix(width, height, flat_matrix.clone());

    // BITWISE 
    let bitwise_universe = BWUniverse::new(width, height, flat_matrix.clone());

    // HASHLIFE
    let hashlife_universe = HashlifeUniverse::new_with_matrix(width, height, flat_matrix.clone());

    (naive_universe, sparse_universe, optimized_universe, track_alive_cells_universe,
         hashed_parallel_universe, bitwise_universe, hashlife_universe)
}

// Advances one time step for any possible implementation
fn global_ticker(universe:&mut AnyUniverse){

    // Match on the enum and call the corresponding tick() method
    match universe {
        AnyUniverse::Naive(u) => u.tick(),
        AnyUniverse::Sparse(u) => u.tick(),
        AnyUniverse::Optimized(u) => u.tick(),
        AnyUniverse::TrackAliveCells(u) => u.tick(),
        AnyUniverse::HashParallel(u) => u.tick(),
        AnyUniverse::Bitwise(u) => u.tick(),
        AnyUniverse::Hashlife(u) => u.tick(),
    }
}

// Advances a number of iterations for any possible implementation
fn universe_iterator(universe:&mut AnyUniverse, n_iter: usize){

    match universe {
        AnyUniverse::Naive(u) => u.run_iterations(n_iter),
        AnyUniverse::Sparse(u) => u.run_iterations(n_iter),
        AnyUniverse::Optimized(u) => u.run_iterations(n_iter),
        AnyUniverse::TrackAliveCells(u) => u.run_iterations(n_iter),
        AnyUniverse::HashParallel(u) => u.run_iterations(n_iter),
        AnyUniverse::Bitwise(u) => u.run_iterations(n_iter),
        AnyUniverse::Hashlife(u) => u.run_iterations(n_iter),
    }
}

// Compute CRC32 for the current state
fn universe_check(universe:&mut AnyUniverse) -> u32{

    // Match on the enum and call the corresponding crc32() method
    match universe {
        AnyUniverse::Naive(u) => u.crc32(),
        AnyUniverse::Sparse(u) => u.crc32(),
        AnyUniverse::Optimized(u) => u.crc32(),
        AnyUniverse::TrackAliveCells(u) => u.crc32(),
        AnyUniverse::HashParallel(u) => u.crc32(),
        AnyUniverse::Bitwise(u) => u.crc32(),
        AnyUniverse::Hashlife(u) => u.crc32(),
    }
}


// this function will test run interations for all versions and collect information regarding runtime and memory usage
fn gather_iteration_info(universe: &mut AnyUniverse, iterations: usize) -> (u128, Vec<u128>, Vec<u64>, Vec<String>){
    
    let mut iteration_times = Vec::new();
    let mut checksums: Vec<String> = Vec::new();
    let mut memory_use = Vec::new();
    memory_use.push(get_memory_usage()/1024);

    let global_start = Instant::now();
    let mut iter_start = Instant::now();

    // measuring every 10 iterations
    for i in 0..iterations {
        if i % 10 == 0 {
            // Start the clock
            iter_start = Instant::now();
            global_ticker(universe);

        } else if i % 10 == 9 {

            global_ticker(universe);
            // Record time per 10 interations
            let iter_time = iter_start.elapsed().as_millis();
            let check = format!("{:06X}", universe_check(universe)) ;

            checksums.push(check);
            iteration_times.push(iter_time);
            
        } else {
            global_ticker(universe);
        }
    }
    let global_time = global_start.elapsed().as_millis(); // Total elapsed time
    memory_use.push(get_memory_usage()/1024);
    
    (global_time, iteration_times, memory_use, checksums)
}


// measuring only the global time for a given number of iterations
fn measure_time(universe: &mut AnyUniverse, iterations: usize) -> u128{
    
    let start = Instant::now();

    // Run the universe for the given number of iterations
    universe_iterator(universe, iterations);
    
    start.elapsed().as_millis() 
}





fn main() {

    ///////////////// PARAMETERS //////////////////////////
    // Get scale and iterations from command line
    let args: Vec<String> = env::args().collect();
    let scale = &args[1].parse::<u32>().unwrap();
    let iterations = args[2].parse::<usize>().unwrap();
    
    // grids will be square, only width is computed and used as gridsize
    let width = usize::pow(2, 6 + scale);

    // seed for random matrix initialization
    let seed: u64 = 420;

    // INITIAL STATES
    // File name of the grid
    let patterns = [
            "justyna.rle", 
            "ark1.rle", 
            "blom.rle", 
            "52513m.rle", 
            "rand_10",
            "rand_25",
            "rand_55",
            "rand_80"
        ];

    // Output CSV
    let output_filename = format!("results_csv/test_{}_{}.csv", scale, iterations);
    let mut output_file = File::create(&output_filename).expect("UNABLE TO CREATE CSV FILE: {}" );
    let headers = format!("PATTERN, ALGORITHM, RUNTIME(ms)");
    writeln!(output_file, "{}", headers).expect("Error writing headers");

    println!("Running {} iterations on universe of size {}^2", iterations, width);
    
    for pat in patterns{

        let flat_matrix: Vec<u8>;
        //either read initial matrix from file or from randomly generated matrix 
        if pat.contains(".rle"){

            // Read RLE file and initialize the flat matrix
            let file_path = format!("./grids/{}", pat);
            flat_matrix = init_from_file(&file_path, width);

        } else {
    
            let aux_str = pat.split("_").last().unwrap();
            let mut p_live: f64 = aux_str.parse().unwrap();
            p_live /=100.;
            flat_matrix = random_init(width, p_live, seed);
        }

        let sample_mean = flat_matrix.iter().filter(|x| **x==1).count() as f64 / (width*width) as f64;
        println!("{} -- % of Alive Cells: {:.3}", pat, sample_mean);

        // --- Initialization ---
        let (naive_universe,  sparse_universe,  optimized_universe, track_alive_cells_universe,
                hashed_parallel_universe, bitwise_universe, hashlife_universe) = initialize_all(flat_matrix, width, width);

        let mut initial_universes: Vec<AnyUniverse> = vec![
                AnyUniverse::Naive(naive_universe),
                AnyUniverse::Sparse(sparse_universe),
                AnyUniverse::Optimized(optimized_universe),
                AnyUniverse::TrackAliveCells(track_alive_cells_universe),
                AnyUniverse::HashParallel(hashed_parallel_universe),
                AnyUniverse::Bitwise(bitwise_universe),
                AnyUniverse::Hashlife(hashlife_universe)
        ];
        // --- Test runs ---
        let universe_names = [
            "Naive", 
            "Sparse", 
            "Optimized",
            "TrackAliveCells", 
            "HashParallel",
            "Bitwise",
            "Hashlife"
        ];

        for (i, univ) in initial_universes.iter_mut().enumerate(){

            let runtime = measure_time(univ, iterations);

            let entry = format!("{}, {}, {}", pat, universe_names[i], runtime);

            writeln!(output_file, "{}", entry).expect("Error writing line");
            println!("{}", entry);
        }
    }
}




