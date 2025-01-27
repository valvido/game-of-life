mod point;
mod rle;
mod quadtree;
mod largekey_table;
mod raw_ops;
mod serialize;
mod typedarena;

pub use crate::point::{Point};
pub use crate::quadtree::{TreeData};
pub use crate::rle::*;

pub fn tile_bytes(arr:&[u8],xsize:usize,tile:usize)->Vec<u8>{
    //use to zoom up the grayscale map
    assert!(arr.len()%xsize == 0);
    (0..arr.len()/xsize)
    .map(|y|&arr[(y*xsize)..((y+1)*xsize)])
    .map(|it|std::iter::repeat(it).take(tile))
    .into_iter().flatten()
    .into_iter().flatten()
    .map(|v|std::iter::repeat(*v).take(tile))
    .into_iter().flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;    
    use std::fs;
 
    fn life_forward_fn(
        sum:u8,
        curval:u8
    ) -> u8 {
        let other_val = sum - curval;
        if other_val == 3 {
            1
        }
        else if other_val == 2 || other_val == 3 {
            curval
        }
        else{
            0
        }
    }
    
    fn step_forward_automata(prevmap: &[u8], nextmap: &mut [u8], xsize:usize, ysize: usize){
        for y in 1..(ysize-1){
            let ymaps = [
                &prevmap[((y-1)*xsize)..((y+0)*xsize)],
                &prevmap[((y+0)*xsize)..((y+1)*xsize)],
                &prevmap[((y+1)*xsize)..((y+2)*xsize)],
            ];
            //sums the first elements
            for x in 1..(xsize-1){
                let csum:u8 = ymaps.iter().map(|v|{let sr:u8 = v[(x-1)..(x+1+1)].iter().sum();sr}).sum();
                let nextval = life_forward_fn(csum, ymaps[1][x]);
                nextmap[y*xsize+x] = nextval;
            }
        }
    }
    fn set_map(points: &Vec<Point>, map: &mut[u8], xsize:usize){
        for p in points.iter(){
            map[(p.y as usize)*xsize+p.x as usize] = 1;
        }
    }
    fn map_to_points(map: &[u8], xsize:usize, ysize: usize)->Vec<Point>{
        let mut res: Vec<Point> = Vec::new();
        for y in 0..ysize{
            for x in 0..xsize{
                if map[y*xsize+x] != 0{
                    res.push(Point{
                        x:x as i64,
                        y:y as i64,
                    });
                }
            }
        }
        res
    }
    fn points_equal(v1: &Vec<Point>, v2: &Vec<Point>)->bool{
        v1.len() == v2.len() && sort_points(v1).iter().zip(sort_points(v2).iter()).all(|(x1,x2)|*x1 == *x2)
    }
    fn step_forward_gold(points: &Vec<Point>, n_steps: u64)->Vec<Point>{
        const SIZE: usize = 300;
        let centered_points = points.iter().map(|x|*x+Point{x:(SIZE/2) as i64,y:(SIZE/2) as i64}).collect();
        let mut map = vec![0 as u8; SIZE*SIZE];
        set_map(&centered_points, &mut map, SIZE);

        for _ in 0..n_steps{
            let mut newmap = vec![0 as u8; SIZE*SIZE];
            step_forward_automata(&map, &mut newmap, SIZE, SIZE);
            map.clone_from_slice(&newmap);
        }
        // let mut mutdata
        map_to_points(&map, SIZE, SIZE).iter().map(|x|*x+Point{x:-((SIZE/2) as i64),y:-((SIZE/2) as i64)}).collect()
    }
    fn step_forward_actual(points: &Vec<Point>, n_steps: u64)->Vec<Point>{
        let mut tree = TreeData::gather_all_points(&points);
        tree.step_forward(n_steps);
        tree.dump_all_points()
    }
    fn sort_points(points: &Vec<Point>)->Vec<Point>{
        let mut sps = points.clone();
        sps.sort();
        sps
    }
    
    fn dump_points_to_str(points: &Vec<Point>)->String{
        let mut mystr = String::new();
        for p in sort_points(points).iter(){
            let pstr = format!("{x}\t{y}\n", x=p.x,y=p.y);
            mystr.push_str(&pstr);
        }
        mystr
    }

    #[test]
    fn test_load_dump_points() {
        let contents = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
        );
        let expected = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
        );

        let points = parse_fle_file(contents);
        let mut tree = TreeData::gather_all_points(&points);
        let out_points = tree.dump_all_points();
        let rle_tot_str = write_rle(&out_points);
        assert_eq!(expected, rle_tot_str);
    }
    
    #[test]
    fn test_load_dump_points_large() {
        let contents = concat!(
            "x = 49, y = 22, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8b\no5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o\n4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$\n19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b\n3o3b3o$21bo5bo!\n"
        );
        let expected = concat!(
            "x = 49, y = 22, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8b\no5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o\n4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$\n19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b\n3o3b3o$21bo5bo!\n"
        );

        let points = parse_fle_file(contents);
        let tree = TreeData::gather_all_points(&points);
        let out_points = tree.dump_all_points();
        let rle_tot_str = write_rle(&out_points);
        assert_eq!(expected, rle_tot_str);
    }
    
    #[test]
    fn test_serailize_deserialize() {
        let contents = concat!(
            "x = 49, y = 22, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8b\no5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o\n4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$\n19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b\n3o3b3o$21bo5bo!\n"
        );
        let expected = concat!(
            "x = 49, y = 22, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8b\no5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o\n4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$\n19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b\n3o3b3o$21bo5bo!\n"
        );

        let points = parse_fle_file(contents);
        let oldtree = TreeData::gather_all_points(&points);
        let treeser = oldtree.serialize_treerepr();
        let newtree = TreeData::deserialize_treerepr(&treeser[..]);
        let out_points = newtree.dump_all_points();
        
        let rle_tot_str = write_rle(&out_points);
        assert_eq!(expected, rle_tot_str);
    }

    #[test]
    fn test_quadtree_against_gold() {
        let contents = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8bo5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b3o3b3o$21bo5bo!\n"
        );
        let points = parse_fle_file(contents);
        let n_steps = 5;
        let actual_points = step_forward_actual(&points, n_steps);
        // println!("done with actual");
        let gold_points = step_forward_gold(&points, n_steps);
    //     fs::write("gold_points.txt", dump_points_to_str(&sort_points(&gold_points)))
    //        .expect("failed to open points.txt file for writing");
    //    fs::write("actual_points.txt", dump_points_to_str(&sort_points(&actual_points)))
    //        .expect("failed to open points.txt file for writing");
   
        assert!(points_equal(&gold_points, &actual_points));
    }
    
    fn maps_eq(v1: &[u8], v2: &[u8])->bool{
        v1.len() == v2.len() && v1.iter().zip(v2.iter()).all(|(x1,x2)|*x1 == *x2)
    }
    #[test]
    fn test_grayscale_map() {
        let contents = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8bo5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b3o3b3o$21bo5bo!\n"
        );
        let points = parse_fle_file(contents);
        let tree = TreeData::gather_all_points(&points);
        // let xsize = 4;
        // let ysize = 4;
        // let map = tree.make_grayscale_map(Point{x: -5, y:-5}, xsize, ysize, 4, 1.5);
        // for y in 0..ysize{
        //     for x in 0..xsize{
        //         print!("{:#04x}, ",map[y*xsize+x]);
        //     }
        //     print!("\n");
        // }
        assert!(
            maps_eq(
                &tree.make_grayscale_map(Point{x: -5, y:-5}, 4, 4, 4, 1.)[..],
                &[
                    0x1f, 0x4e, 0x1d, 0x02, 
                    0x00, 0x19, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00
                ]
            )
        );
        assert!(
            maps_eq(
                &tree.make_grayscale_map(Point{x: -5, y:-5}, 4, 4, 4, 1.5)[..],
                &[
                    0x2f, 0x76, 0x2c, 0x04, 
                    0x00, 0x26, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00,
                ]
            )
        );
        
        assert!(
            maps_eq(
                &tree.make_grayscale_map(Point{x: -5, y:-5}, 12, 8, 2, 1.)[..],
                &[
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                    0x00, 0x0f, 0x3f, 0x1f, 0x2f, 0x00, 0x4f, 0x2f, 0x0f, 0x00, 0x2f, 0x3f,
                    0x00, 0x2f, 0x3f, 0x2f, 0x6f, 0x0f, 0x7f, 0x7f, 0x2f, 0x5f, 0x2f, 0x2f,
                    0x00, 0x3f, 0x00, 0x00, 0x0f, 0x7f, 0x7f, 0x7f, 0x7f, 0x2f, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x7f, 0x1f, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8f, 0x7f, 0x0f, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x2f, 0x0f, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]
            )
        );
        assert!(
            maps_eq(
                &tree.make_grayscale_map(Point{x: 5, y:0}, 8, 8, 0, 1.)[..],
                &[
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 
                    0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0x00,
                    0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
                    0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
                    0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
                    0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]
            )
        );
        // assert!(false);
    }
}