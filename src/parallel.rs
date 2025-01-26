extern crate num_cpus;
extern crate rand;


use std::cmp::{min,max};
use std::sync::Arc;
use std::collections::HashSet;
use std::collections::hash_set;
use std::collections::HashMap;
use std::thread;
use std;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;


//use common::{LifeAlgorithm,Bounds};

use std::iter::Iterator;

pub trait LifeAlgorithm<I: Iterator<Item=(isize, isize)>> {
	/// Advances the simulation forward [count] step(s) 
    fn advance_by(&mut self,count:u64);
	
    /// Sets the value (false or true, dead or alive) of a given cell (x,y)
    fn set(&mut self, cell: (isize, isize), value: bool);
	
    /// Performs any necessary clean up after setting values (for resizing the hashmap) 
    fn clean_up(&mut self);
	
    /// Clears the entire grid
    fn clear(&mut self);
	
    /// Get the current generation
    fn get_generation(&self) -> u64;
	
    /// Gets the bounds of this life simulation
    fn get_bounds(&self) -> Bounds;
    
    /// Gets the value of the cell (x,y) as a bool
    fn get_value(&self, cell: (isize, isize)) -> bool;
    
    /// Gets an iterator over all the live cells. Used to draw on screen or output as ASCII in terminal
	fn live_cells(&self) -> I;
}

#[derive(Clone)]
pub struct Bounds {   
    pub x_min: isize,
    pub x_max: isize,
    pub y_min: isize,
    pub y_max: isize,
}

impl Bounds {
    pub fn new() -> Bounds {
        Bounds { x_min: 0, x_max: 0, y_min: 0, y_max: 0 }
    }

    pub fn from_half_side(s: isize) -> Bounds {
        assert!(s >= 0);
        Bounds { x_min: -s,
                 x_max:  s,
                 y_min: -s,
                 y_max:  s }
    }

    pub fn update_bounds(&mut self, x:isize, y:isize) {
        if x < self.x_min {
            self.x_min = x;
        }
        if x > self.x_max {
            self.x_max = x;
        }
        if y < self.y_min {
            self.y_min = y;
        }
        if y > self.y_max {
            self.y_max = y;
        }
    }
}

// Extra functionality for bounds 
// since the grid will be split, this is a function to merge the pieces back together
impl Bounds {
    fn merge(&mut self, other:Bounds) {
        self.x_min = min(self.x_min, other.x_min);
        self.x_max = max(self.x_max, other.x_max);
        self.y_min = min(self.y_min, other.y_min);
        self.y_max = max(self.y_max, other.y_max);
    }
}


// Life is the current (generation) state of the game, there is no grid but a HashMap that maps the index of a cell to its value (1/0)
// HashMap is a sparse representation, only live cells are included, or dead ones that are relevant (eg. as neighbors of live ones)
//Arc: thread safe shared value
//parts: HashSet of indices as part of the grid for parallel computation, rect: bounds of the grid
// !! parts are not rectangular chunks/parts of the grid but randomly allocated cells across the grid
#[derive(Clone)]
pub struct Life {
    pub generation: u64,
    pub cells: Arc<HashMap<(isize, isize), bool>>,
    parts: Vec<Arc<HashSet<(isize, isize)>>>,
    rect: Bounds,
    num_threads:usize,
}

impl Life {
    pub fn new() -> Life {
        let num_threads = num_cpus::get() * 2; //Use twice as many threads as we have cores
        // Life is set, parts is a vector of one HashSet per threads used
        Life { generation: 0, cells: Arc::new(HashMap::new()), parts: vec![Arc::new(HashSet::new()); num_threads], rect: Bounds::new(), num_threads:num_threads }
    }
    // checks if there are multiple accesses to the cells vector, prints where (for which function) it happened 
    //for debugging
    fn cells_access_record(s:&str) {
        println!("Arc::get_mut(&mut self.cells) returned None at {}", s);
    }

    // getting the next state by applying the rules, version for Arc (shared cells vector)
    fn next_val_from_arc(cells_ref:&Arc<HashMap<(isize, isize), bool>>, x:isize, y:isize) -> bool {
        let mut neighbors: i8 = 0;
        for (i,j) in Life::get_adjacent(x,y) {
            if cells_ref.contains_key(&(i,j)) {
                if cells_ref[&(i,j)] == true { neighbors+=1; }
            }
        }
        if (neighbors == 3) | ((neighbors == 2) & (cells_ref[&(x,y)] == true)) {
            true
        } else {
            false
        }
    }

    fn get_adjacent(x:isize, y:isize) -> Vec<(isize, isize)> {
        vec![(x+1, y  ),
             (x+1, y+1),
             (x  , y+1),
             (x-1, y+1),
             (x-1, y  ),
             (x-1, y-1),
             (x  , y-1),
             (x+1, y-1)]
    }

    pub fn randomize_alive_cells(&mut self, num_alive_cells: usize, width: isize, height: isize, rng: &mut StdRng) {
        //let mut rng = rand::thread_rng(); // Initialize random number generator

        // Randomize the alive cells within these bounds
        for _ in 0..num_alive_cells {
            let x = rng.gen_range(0..width); // x coordinate between 0 and width-1
            let y = rng.gen_range(0..height); // y coordinate between 0 and height-1
            self.set((x, y), true); // This is valid because `Life` implements `LifeAlgorithm`
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

// hashSet IntoIter allows HashSet to be itereted in parallelization
impl LifeAlgorithm<hash_set::IntoIter<(isize, isize)>> for Life {
    fn advance_by(&mut self, count:u64){
        for _ in 0..count {
            let mut thread_handles = vec![];
            for k in 0..self.num_threads {
                // copy all the cells
                // all of them needed bc  of ghost rows (?)
                //every thread knows about (has a copy of) all cells, but only computes some of them 
                let my_cells = self.cells.clone();
                // each thread gets their part (amount of indices) of the cells and computes the new state for them
                let my_part = self.parts[k].clone();
                thread_handles.push(thread::spawn(move || {
                    let mut cells_new = HashMap::new();
                    for &(x,y) in my_part.iter() {
                        cells_new.insert((x,y), Life::next_val_from_arc(&my_cells,x,y));
                    }
                    cells_new
                }));
            }
            //collects and merges result in one vector
            let mut cells_new: Vec<HashMap<(isize, isize), bool>> = vec![];
            for hand in thread_handles {
                cells_new.push(hand.join().unwrap());
            }
            // now the newly calculated values are updated in the hashmap (life grid)
            // num_threads only indicated how many parts there were, here no parallelization/spawn takes place!
            for k in 0..self.num_threads {
                // & dereferences, value is extracted!
                for (&(x,y),v) in &cells_new[k] {
                    if let Some(re) = Arc::get_mut(&mut self.cells) {
                        //If we successfully got a mutable reference re to the HashMap, we try to access the specific cell (x, y):
                        //(*re) dereferences the mutable reference to the HashMap inside the Arc.
                        //get_mut(&(x, y)) tries to get a mutable reference to the value of the key (x, y).
                        if let Some(z) = (*re).get_mut(&(x,y)) {
                            // z is the mutable reference to the cell
                            // gets dereferenced, the value v too, so that z can be changed and updated
                            *z = *v;
                        }
                    } else {
                        //else use the access function to give the warning
                        Life::cells_access_record("Life::advance");
                    }
                }
            }
            self.clean_up();
            self.generation += 1;
        }
        
    }
    //setting a state (not with rules)
    fn set(&mut self, (x, y): (isize, isize), v: bool){
        // if the cell doesnt exist yet and the cells vector is not accessed by multiple threads -> cell and value are added
        if !self.cells.contains_key(&(x,y)) {
            if let Some(re) = Arc::get_mut(&mut self.cells) {
                (*re).insert((x,y), v);
            } else {
                Life::cells_access_record("Life::set, does not contain key");
            }
            // cells are allocated randomly to threads to ensure even workload for each thread (there could be chunks 
            // with almost only dead cells which dont suppose any work)
            let ind = rand::random::<usize>()%self.num_threads;
            (*Arc::make_mut(&mut self.parts[ind])).insert((x,y));
            // if let Some(pe) = Arc::get_mut(&mut self.parts[ind]) {
            //     (*pe).insert((x,y));
            // } else {
            //     parts_access_record(ind, "Life::set, does not contain_key");
            // }
        }
        else {
            // if the cell already exists, just update the value
            if let Some(re) = Arc::get_mut(&mut self.cells) {
                if let Some(z) = (*re).get_mut(&(x,y)) {
                    *z = v;
                }
            } else {
                Life::cells_access_record("Life::set, contains key");
            }
        }
    }

    //keeps the grid minimal: "barren" cells (dead cells with no live neighbors) are deleted from the grid since they will not come alive
    // making sure that live cells have always all neighbors in the grid (even if dead), to be considered for next computation
    // rect. bounds will always be close to the last live cells
    fn clean_up(&mut self){
        self.rect.x_min = std::isize::MAX;
        self.rect.x_max = std::isize::MIN;
        self.rect.y_min = std::isize::MAX;
        self.rect.y_max = std::isize::MIN;
        let mut thread_handles = vec![];
        for k in 0..self.num_threads {
            let my_cells = self.cells.clone();
            let my_part = self.parts[k].clone();
            let mut temp = self.rect.clone();
            //for every thread check cells and their environments if the cells are alive/dead and included already in the grid
            thread_handles.push(thread::spawn(move || {
                let mut to_add: Vec<(isize, isize)> = vec![];
                let mut to_del: Vec<(isize, isize)> = vec![];
                for &(x,y) in my_part.iter() {
                    if my_cells[&(x,y)] == true {
                        // if a cell is alive update the bounds to make the grid boundaries (up until alive cells) include that cell
                        temp.update_bounds(x,y);
                        // if the adjacent cells are not yet part of the grid, mark them as to be added
                        for (i,j) in Life::get_adjacent(x,y) {
                            if !my_cells.contains_key(&(i,j)) {
                                to_add.push((i,j));
                            }
                        }
                    } else {
                        // check for the dead cells if any of their neighbors are alive, else if that 3x3 block is entirely dead,
                        // the barren variable is TRUE
                        let mut barren = true;
                        for (i,j) in Life::get_adjacent(x,y) {
                            if my_cells.contains_key(&(i,j)) {
                                if my_cells[&(i,j)] == true {
                                    barren = false;
                                    break;
                                }
                            }
                        }
                        // if the cell and their neighbors are dead, mark as to delete 
                        if barren {
                            to_del.push((x,y));
                        } else {
                            temp.update_bounds(x,y);
                        }
                    }
                }
                (temp, to_add, to_del)
            }));
        }
        // collect all the results and updates the outer bounds
        let mut to_adds: Vec<Vec<(isize, isize)>> = vec![];
        let mut to_dels: Vec<Vec<(isize, isize)>> = vec![];
        for hand in thread_handles {
            let ret = hand.join().unwrap();
            self.rect.merge(ret.0);
            to_adds.push(ret.1);
            to_dels.push(ret.2);
        }

        for k in 0..self.num_threads {
            // for every new cell (to add)
            for &(x,y) in &to_adds[k] {
                if !self.cells.contains_key(&(x,y)) {
                    if let Some(re) = Arc::get_mut(&mut self.cells) {
                        (*re).insert((x,y), false);
                    } else {
                        Life::cells_access_record("Life::cleanup, inserting new cells");
                    }
                    // new cell added, now maybe update the bounds if it was at border
                    //assign that new cell randomly to a thread
                    self.rect.update_bounds(x,y);
                    let ind = rand::random::<usize>()%self.num_threads;
                    (*Arc::make_mut(&mut self.parts[ind])).insert((x,y));
                    // if let Some(pe) = Arc::get_mut(&mut self.parts[ind]) {
                    //     (*pe).insert((x,y));
                    // } else {
                    //     parts_access_record(ind, "Life::cleanup, inserting new cells");
                    // }
                }
            }
            // tb deleted cells are removed from the grid (Life.cells) and from their allocated thread
            for &(x,y) in &to_dels[k] {
                if let Some(re) = Arc::get_mut(&mut self.cells) {
                    (*re).remove(&(x,y));
                } else {
                    Life::cells_access_record("Life::cleanup, removing cells");
                }
                (*Arc::make_mut(&mut self.parts[k])).remove(&(x,y));
                // if let Some(pe) = Arc::get_mut(&mut self.parts[k]) {
                //     (*pe).remove(&(x,y));
                // } else {
                //     parts_access_record(k, "Life::cleanup, removing cells");
                // }
            }
        }
    }

    fn get_generation(&self) -> u64 {
        self.generation
    }

    fn get_bounds(&self) -> Bounds {
        self.rect.clone()
    }

    fn get_value(&self, cell: (isize, isize)) -> bool {
        if let Some(v) = self.cells.get(&cell) {
            *v
        } else {
            false
        }
    }
    // remove all the cells (clear the grid)
    fn clear(&mut self) {
        if let Some(re) = Arc::get_mut(&mut self.cells) {
            (*re).drain();
        } else {
            Life::cells_access_record("Life::clear");
        }
    }

    // give an iterator over the life cells
    fn live_cells(&self) -> hash_set::IntoIter<(isize, isize)> {
        let mut out: HashSet<(isize, isize)> = HashSet::new();
        for (key, value) in self.cells.iter() {
            if *value {
                out.insert(*key);
            }
        }
        out.into_iter()
    }
}


use std::fmt;


impl fmt::Display for Life {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let width = (self.rect.x_max - self.rect.x_min + 1) as usize;
        //let height = (self.rect.y_max - self.rect.y_min + 1) as usize;

        // Create a flat 1D vector to simulate the grid
        let mut grid = vec!["☁ "; width * height];

        for (&(x, y), &alive) in self.cells.iter() {
            if alive {
                let col = (x) as usize;
                let row = (y) as usize;
                grid[row * width + col] = "🦄";
            }
        }

        // Render the grid as lines using chunks
        for line in grid.chunks(width) {
            for &cell in line {
                write!(f, "{}", cell)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}