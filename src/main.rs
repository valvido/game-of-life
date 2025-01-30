mod optimized_alg; 
mod track_alive_cells; 
mod parallelize;
use parallelize::Universe as ParallelUniverse;
use optimized_alg::Universe as OptimizedUniverse; 
use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;
use track_alive_cells::Universe as TrackAliveCellsUniverse;
use std::time::Instant;
use rand::Rng;
use sysinfo::{System, SystemExt};

// function for printing memory usage that I transwered inside the files ionstead of in the main
// fn print_memory_usage(label: &str) {
//     let mut sys = System::new_all();
//     sys.refresh_all();
//     let memory_used = sys.used_memory();
//     let memory_total = sys.total_memory();

//     println!(
//         "{} - Memory Usage: {} MB / {} MB",
//         label,
//         memory_used / 1024,
//         memory_total / 1024
//     );
// }

fn main() {
    let width = 1000;
    let height = 1000;

   

    // Generate initial state with random 0s and 1s
    let mut rng = rand::thread_rng();
    let initial_state: Vec<Vec<u8>> = (0..height)
        .map(|_| (0..width).map(|_| rng.gen_range(0..=1)).collect())
        .collect();

    // Flatten the matrix for sparse implementation
    let flat_matrix: Vec<u8> = initial_state.iter().flatten().cloned().collect();

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
    let mut sparse_universe = SparseUniverse::new_with_matrix(width, height, flat_matrix);
    let start_sparse = Instant::now();
    sparse_universe.run_iterations(10);
    let sparse_time = start_sparse.elapsed().as_millis();
    println!("Sparse-Matrix Approach: {} ms", sparse_time);

    // ===== Optimized Version =====
    println!("\nOptimized Game of Life Algorithm:");
    let flat_initial_state: Vec<u8> = initial_state.iter().flatten().cloned().collect();
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
        initial_live_cells.clone() // Clone here to preserve for parallel version
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
        initial_live_cells // Use original since it was cloned before
    );    

    let start_parallel = Instant::now();
    parallel_universe.run_iterations(10);
    let parallel_time = start_parallel.elapsed().as_millis();
    println!("Parallelized Approach: {} ms", parallel_time);
}
