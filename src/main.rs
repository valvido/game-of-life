use std::time::{Duration, Instant};
use std::thread;
use wasm_game_of_life::Universe;
// use wasm_game_of_life::Cell;
mod parallel;
use parallel::Life;
//use parallel::Bounds;
use parallel::LifeAlgorithm;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;


// Import everything else as is from your original code.


fn main() {
    
    let sizes = vec![100, 200, 500, 1000, 2000, 5000, 10000, 20000]; // Different grid sizes
    let iterations_list = vec![50, 100, 200]; 
    let seed: u64 = 42; // Set a seed value (can be user-defined)
    let mut rng = StdRng::seed_from_u64(seed); // Create a seeded RNG
    /*
    // First loop: Vary size, keep iterations fixed
    for size in &sizes {
        let iterations = 1000; // Set a fixed number of iterations for this loop
        let alive_cells = (0.25 * *size as f64).round() as usize;
        // Naive simulation
        /*let start = Instant::now();
        let mut universe = Universe::new(*size, *size, 1000); // Initialize Universe with square grid size
        for _ in 0..iterations {
            universe.tick();
        }
        let naive_duration = start.elapsed();
        */ 
        // Parallel simulation
        let start = Instant::now();
        let mut parl_life = Life::new();
        parl_life.randomize_alive_cells(alive_cells, *size as isize, *size as isize, &mut rng);
        parl_life.advance_by(iterations);
        let parl_duration = start.elapsed();

        // Print the results
        println!(
            "Size: {}x{}, Iterations: {} - Naive finished in {:.2} sec, Parallel finished in {:.2} sec",
            size, size, iterations, 0, parl_duration.as_secs_f64()
        );
    }

    // Second loop: Vary iterations, keep size fixed (e.g., 100)
    let size = 100; // Fixed grid size for this loop
    for iterations in &iterations_list {
        // Naive simulation
        let start = Instant::now();
        let mut universe = Universe::new(size, size, 1000); // Initialize Universe with square grid size
        for _ in 0..*iterations {
            universe.tick();
        }
        let naive_duration = start.elapsed();

        // Parallel simulation
        let start = Instant::now();
        let mut parl_life = Life::new();
        parl_life.randomize_alive_cells(1000, size as isize, size as isize, &mut rng);
        parl_life.advance_by(*iterations);
        let parl_duration = start.elapsed();

        // Print the results
        println!(
            "Size: {}x{}, Iterations: {} - Naive finished in {:.2} sec, Parallel finished in {:.2} sec",
            size, size, iterations, naive_duration.as_secs_f64(), parl_duration.as_secs_f64()
        );
    }
    */

    //grid equality check
    let size_check = 10;
    let iter_check = 10;
    let alive_check = (0.45 * (size_check*size_check) as f64).round() as usize;

    let mut naive_grid_init = Universe::new(size_check, size_check, alive_check as u32, seed); // Initial grid
    let mut naive_grid_end = naive_grid_init.clone(); // Clone for ticking
    for _ in 0..iter_check {
        naive_grid_end.tick();
    }
    let mut parl_grid_init = Life::new();
    parl_grid_init.randomize_alive_cells(alive_check, size_check as isize, size_check as isize, &mut rng);
    let mut parl_grid_end = parl_grid_init.clone(); // Clone for ticking
    //parl_grid_end.advance_by(iter_check);

    println!("Naive Implementation - Initial Grid:");
    println!("{}", naive_grid_init.render()); // Uses the render method for display
    println!("Naive Implementation - Final Grid (after {} iterations):", iter_check);
    println!("{}", naive_grid_end.render()); // Uses the render method for display

    println!("Parallel Implementation - Initial Grid:");
    println!("{}", parl_grid_init.render()); // Uses the render method for display
    println!("Parallel Implementation - Final Grid (after {} iterations):", iter_check);
    println!("{}", parl_grid_end.render()); // Uses the render method for display
}
