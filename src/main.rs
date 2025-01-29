mod optimized_alg; 
mod track_alive_cells; 


use optimized_alg::Universe as OptimizedUniverse; 
use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;
use track_alive_cells::Universe as TrackAliveCellsUniverse;
use std::time::Instant;
use rand::Rng;

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

    // Naive implementation
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

    //let naive_cells = naive_universe.get_cells();
    //for row in 0..height {
    //    for col in 0..width {
    //        let idx = (row * width + col) as usize;
    //        print!("{}", naive_cells[idx]);
    //    }
    //    println!();
    //}
    println!("Naive Approach: {} ms", naive_time);

    // Sparse matrix implementation
    println!("\nSparse Matrix Game of Life Algorithm:");
    let mut sparse_universe = SparseUniverse::new_with_matrix(width, height, flat_matrix);
    let start_sparse = Instant::now();
    sparse_universe.run_iterations(10);
    let sparse_time = start_sparse.elapsed().as_millis();

    //println!("{}", sparse_universe.render());
    println!("Sparse-Matrix Approach: {} ms", sparse_time);

    // Optimized version
    println!("\nOptimized Game of Life Algorithm:");
    let flat_initial_state: Vec<u8> = initial_state.iter().flatten().cloned().collect();
    let mut optimized_universe = OptimizedUniverse::new(width as usize,
        height as usize, flat_initial_state);

    let start_optimized = Instant::now();
    optimized_universe.run_iterations(10);
    let optimized_time = start_optimized.elapsed().as_millis();

    //println!("{}", optimized_universe.render());
    println!("Optimized Cache Algorithm: {} ms", optimized_time);

    println!("\nTrack live cells algorithm:");

     // Track-alive-cells implementation
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
 
     let mut track_alive_cells_universe = TrackAliveCellsUniverse::new(width as usize, height as usize, initial_live_cells);
 
     let start_track_alive = Instant::now();
     track_alive_cells_universe.run_iterations(10);
     let track_alive_time = start_track_alive.elapsed().as_millis();
 
     println!("Track-Alive-Cells Approach: {} ms", track_alive_time);
}
