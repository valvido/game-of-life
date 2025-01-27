
use std::{env, time::Instant};
use png::{Encoder,ColorType};


use std::fs;
use hashlife_fast::{TreeData,Point, parse_fle_file, write_rle};

fn dump_points_to_str(points: &Vec<Point>)->String{
    let mut sorted_points = points.clone();
    sorted_points.sort();
    let mut mystr = String::new();
    for p in sorted_points.iter(){
        let pstr = format!("{x}\t{y}\n", x=p.x,y=p.y);
        mystr.push_str(&pstr);
    }
    mystr
}

fn save_png(fname:&str, xsize: usize, ysize: usize, data: &[u8]){
    let mut f = fs::File::create(fname).unwrap();
    let mut enc = Encoder::new(f, xsize as u32, ysize as u32);
    enc.set_color(ColorType::Grayscale);
    let mut writer = enc.write_header().unwrap();
    writer.write_image_data(data);
    writer.finish();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 4);
    let in_filename = &args[1];
    let n_steps = args[2].parse::<u64>().unwrap();
    // let n_steps = args[2].parse::<u64>().unwrap();
    let out_filename = &args[3];

    let contents = fs::read_to_string(in_filename).unwrap();
    let points = parse_fle_file(&contents);
    let start_time = Instant::now();
    let mut tree = TreeData::gather_all_points(&points);
    println!("finished gathering");
    let MAX_STEPS = 1<<9;
    let mut step_n = 0;
    let mut frame = 0;
    let xsize = 800;
    let ysize = 800;
    while step_n < n_steps{
        let cur_steps = std::cmp::min(n_steps - step_n, MAX_STEPS);
        tree.step_forward(cur_steps);
        if tree.hash_count() > 15000000{
            let bef_garbage_tree_size = tree.hash_count();
            tree = tree.pruned_tree();
            let aft_garbage_tree_size = tree.hash_count();
            println!("Garbage collected, Bef: {},\t Aft: {}",bef_garbage_tree_size,aft_garbage_tree_size);
        }
        step_n += cur_steps;
        let t = start_time.elapsed().as_secs_f64();
        println!("reached step {} at time {} (avg {}) hash size {}",step_n,t,t/step_n as f64, tree.hash_count());
        let fname = format!("frames/step{:03}.png", frame);
        save_png(fname.as_str(),xsize,ysize,&tree.make_grayscale_map(Point{x:0,y:0}, xsize, ysize, 7, 2006.)[..]);
        frame += 1;
    }
    println!("finished stepping");
    let out_points = tree.dump_all_points();
    println!("finished dumping");
    let rle_tot_str = write_rle(&out_points);

    // let orig_p_str = dump_points_to_str(&points);
    // let new_p_str = dump_points_to_str(&out_points);
    // fs::write("orig_points.txt", orig_p_str)
    //     .expect("failed to open points.txt file for writing");
    // fs::write("new_points.txt", new_p_str)
    //     .expect("failed to open points.txt file for writing");

    fs::write(out_filename, rle_tot_str)
        .expect("failed to open output file for writing");
}