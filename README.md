<div align="center">

  <h1><code>Game of Life Implementations in Rust</code></h1>

  <strong>Valentina Vidovic, Alexandra Kübelbäck, <br>
  Karolina Muciek, Carlos Ruiz</strong>

</div>

##  🦄 About

We implemented and tested seven different algorithms that compute new generations of Conway's Game of Life.

* **Naive:** A basic implementation.
* **Cache Optimized:** Some memory optimization to speed up access and copying.
* **Sparse Matrix Representation:** Since the rules of Conway's game tend to generate sparse universes, representing the grid as a sparse matrix allows to optimize computations.
* **Bitwise:** Stores cells as single bits to save memory for big universes.

* **Live Cell Tracker:** Keeps a record of active zones in the universe to avoid unnecessary computations.
* **Hashed Parallel:** Keeps a list of active cells as a Hash Set (with the same purpose fo only working on active areas of the grid) and parallelizes computations at each time step.

* **Hashlife:** A recursive, tree-based approach that uses memoization to speed up computation as more generations pass.

## 🦄 Usage

### ☁ Use `cargo run` to measure different Game of Life algorithms  ☁ 

Compute a number of iterations of the Game in a toroidal grid of size $2^k$, where $k = 6 + \texttt{scale}$:

```
cargo run <scale> <iters>
```
Seven different algorithms to compute generations of the Game of Life are timed and the outputs are saved in `results_csv/test_<size>_<iters>.csv`


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
