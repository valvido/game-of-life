use wasm_game_of_life::{Universe as NaiveUniverse, Cell as NaiveCell};
use wasm_game_of_life::sparse_matrix::Universe as SparseUniverse;

fn main() {
    let width = 10;
    let height = 10;

    let initial_state = vec![
        vec![0, 1, 0, 0, 0, 0, 1, 0, 1, 0],
        vec![0, 0, 1, 0, 1, 1, 0, 1, 0, 0],
        vec![1, 0, 0, 1, 0, 0, 0, 0, 1, 1],
        vec![0, 0, 0, 0, 0, 1, 0, 0, 1, 0],
        vec![0, 1, 1, 0, 0, 0, 0, 1, 0, 1],
        vec![1, 0, 0, 1, 1, 1, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 0, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
        vec![0, 1, 0, 1, 0, 0, 0, 0, 1, 0],
    ];

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
    naive_universe.run_iterations(10);

    let naive_cells = naive_universe.get_cells();
    for row in 0..height {
        for col in 0..width {
            let idx = (row * width + col) as usize;
            print!("{}", naive_cells[idx]);
        }
        println!();
    }

    // Sparse matrix implementation
    println!("\nSparse Matrix Game of Life:");
    let mut sparse_universe = SparseUniverse::new_with_matrix(width, height, flat_matrix);
    sparse_universe.run_iterations(10);

    println!("{}", sparse_universe.render());
}
