#![allow(unused_imports)]
use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;

use std::env;

mod utils;
use utils::*;


fn main() {
     /* // Get initial grid file from command line
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
     */

    let file_name = "blom.rle";

    let file_path = format!("./grids/{}", file_name);

    let flat_matrix =  init_from_file(&file_path);

    let grid_size = (flat_matrix.len() as f64).sqrt().floor() as usize;

    // Naive implementation
    println!("Naive Game of Life:");
    let initial_state = vec_to_matrix(&flat_matrix, grid_size);
    let initial_cells: Vec<NaiveCell> = initial_state
        .iter()
        .flatten()
        .map(|&x| if x == 1 { NaiveCell::Alive } else { NaiveCell::Dead })
        .collect();

    
    let grid_w =  grid_size as u32;

    let mut naive_universe = NaiveUniverse::new_with_cells(grid_w, grid_w, initial_cells);
    naive_universe.run_iterations(10);

    let naive_cells = naive_universe.get_cells();
    for row in 0..grid_size {
        for col in 0..grid_size {
            let idx = (row * grid_size + col) as usize;
            print!("{}", naive_cells[idx]);
        }
        println!();
    }

    // Sparse matrix implementation
    println!("\nSparse Matrix Game of Life:");
    let mut sparse_universe = SparseUniverse::new_with_matrix(grid_w, grid_w, flat_matrix);

    sparse_universe.run_iterations(10);
}
