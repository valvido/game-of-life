#![allow(dead_code)]
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::cmp;
use std::cmp::Ordering;

use sysinfo::{System, SystemExt};

use csv::Writer;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn parse_header(line: &str) -> (usize, usize) {
    // Extract x value from header line "x = N, y = N, rule = B3/S23"
    let parts: Vec<&str> = line.split(',').collect();
    let mut dims: (usize, usize) = (0, 0);

    for part in parts {

        let kv: Vec<&str> = part.trim().split('=').collect();
        
        if kv.len() == 2 {
            match kv[0].trim(){
                "x" => dims.0 = kv[1].trim().parse().unwrap(),
                "y" => dims.1 = kv[1].trim().parse().unwrap(),
                _ => ()
            } 
        }
    }
    dims
}

pub fn iter_coords<F>(boardrow: &str, dims: (usize, usize), func: &mut F)
where
    F: FnMut(u8)
{
    let mut prefixnum: i64 = 0;
    let mut prefixset = false;
    let mut row_width  = 0;
    let mut row_count = 0;
    let width = cmp::max(dims.0, dims.1);
    
    for c in boardrow.chars() {
        if c.is_numeric() {
            prefixnum = prefixnum * 10 + c.to_digit(10).unwrap() as i64;
            prefixset = true;
        } else {
            let repeat = if prefixset { prefixnum } else { 1 } as usize;
            if c == 'b' {
                for _ in 0..repeat {
                    func(0);
                    row_width += 1;
                }
            } else if c == 'o' {
                for _ in 0..repeat {
                    func(1);
                    row_width += 1;
                }
            } else if c == '$' {

                // Number of processed rows
                row_count += repeat;

                // Count how many blanks are needed to pad this row
                let row_pad = width - row_width;
                match row_pad.cmp(&0) {
                    Ordering::Equal => {row_width = 0;
                                    continue; 
                                    },
                    Ordering::Greater => {
                        for _ in 0..row_pad{ func(0) }
                        },
                    Ordering::Less => {
                        panic!("RLE file incorrectly formatted, row has {} entries - expected {}", row_width, width)
                        }
                }
                row_width = 0;

                // If there are multiple line jumps, then add repeat empty lines:
                for _ in 0..width*(repeat-1) {
                    func(0);
                }

            } else if c == '!' {

                // Add missing padding
                let total = width*row_count + row_width;
                let n_pad =  width*width - total;
                
                if n_pad==0{
                    break;
                } else {
                    for _ in 0..n_pad {
                        func(0);
                    }
                }
                row_width=0;
            } else if c == '\n' || c == '\r' {
                // do nothing on line break
                continue;
            } else {
                panic!("RLE file incorrectly formatted, only 'b' and 'o' allowed.")
            }
            prefixset = false;
            prefixnum = 0;
        }
        // Verify that format is correct
        if row_count>width{
            panic!("RLE file incorrectly formatted, too many rows in file");
        }
    }
}

/// Read .rle file and return problem parameters
pub fn init_from_file(file_path: &str, width: usize) -> Vec<u8> {
    // Reads the file and saves as a string
    let f = BufReader::new(File::open(file_path).unwrap());
    let mut line_iter = f.lines();
    
    // Skip comments
    let mut header_line = String::new();
    while let Some(Ok(line)) = line_iter.next() {
        if !line.starts_with('#') {
            header_line = line;
            break;
        }
    }

    // Parse the grid size from header
    let dims = parse_header(&header_line);

    // Convert RLE to binary vector to represent initial grid
    let mut rle_str = String::new();
    while let Some(line) = line_iter.next() {
        rle_str.push_str(&line.unwrap());
    }

    let mut init_state: Vec<u8> = Vec::new();
    
    // Return initial grid as a N^2 sized vector
    iter_coords(rle_str.as_str(), dims, &mut |p| {
        init_state.push(p);
    });

    // Embed initial state in the middle of NxN grid
    let grid_size = (init_state.len() as f64).sqrt().floor() as usize;

    let n_offset = calc_padding(width, grid_size);

    let mut output_mat = vec![0; width*width];

    // Copy the input matrix to the center of the result matrix
    for i in 0..grid_size {
        for j in 0..grid_size {
            let source_idx = i * grid_size + j;
            let target_idx = (i + n_offset) * width + (j + n_offset);
            output_mat[target_idx] = init_state[source_idx];
        }
    }

    output_mat
}

pub fn calc_padding(big_n: usize, grid_size: usize) -> usize{

    assert!(big_n>grid_size, "Pattern {}sq is too big for grid of size {}", grid_size, big_n);

    let diff = big_n-grid_size;

    match diff%2 {
        0 => diff/2,
        1 => (diff+1)/2,
        _ => panic!("INTEGER DIVISION BY 2 YIELDED SMTH WEIRDD!!!!")
    }
}

pub fn vec_to_matrix<T: Clone>(vec: &[T], n: usize) -> Vec<Vec<T>> {
    vec.chunks(n)
        .map(|chunk| chunk.to_vec())
        .collect()
}

pub fn display(mat: &Vec<Vec<u8>>)
{
    for row in mat{
            println!("{:?}", row);
        }
    }

pub fn get_memory_usage() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.used_memory() // Returns memory usage in KB
}
    
pub fn write_results_to_csv(
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
        "", "", "" ] // Iterations
        )?;
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
        
    

