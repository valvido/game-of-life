use std::hash::Hasher;
use std::intrinsics::transmute;
use std::mem::size_of;

use crate::largekey_table::LargeKeyTable;
use crate::serialize;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use metrohash::MetroHash128;
use crate::point::Point;
use crate::raw_ops::*;
use crate::serialize::*;


#[derive(Copy, Clone, Default)]
pub struct QuadTreeValue{
    lt: u128,
    rt: u128,
    lb: u128,
    rb: u128,
}
impl QuadTreeValue{
    fn to_array(&self)->[u128;4]{
        [self.lt,self.rt,self.lb,self.rb]
    }
    fn from_array(arr: &[u128;4])->QuadTreeValue{
        QuadTreeValue{
            lt: arr[0],
            rt: arr[1],
            lb: arr[2],
            rb: arr[3],
        }
    }
}
impl QuadTreeValue{
    fn key(&self) -> u128 {
        let mut hasher = MetroHash128::new();
        let res: u128;
        unsafe{
        hasher.write( &std::mem::transmute::<QuadTreeValue, [u8;64]>(*self));
        res = std::mem::transmute::<(u64,u64), u128>( hasher.finish128());
        }
        res
    }
    fn is_raw(&self)->bool{
        node_is_raw(self.lt)
    }
}
#[derive(Copy, Clone, Default)]
struct QuadTreeNode{
    v: QuadTreeValue,
    forward_key: u128,
    set_count: u64,
    forward_steps: u64,
}
const NULL_KEY: u128 = 0xcccccccccccccccccccccccccccccccc;
pub struct TreeData{
    map: LargeKeyTable<QuadTreeNode>,
    black_keys: Vec<u128>,
    root: u128,
    depth: u64,
    offset: Point,
    age: u64,
}

const BLACK_BASE: u128 = 0;
impl TreeData{
    pub fn new() -> TreeData{
        const INIT_SIZE_POW2: u8 = 1;
        let mut tree_data = TreeData{
            map: LargeKeyTable::new(INIT_SIZE_POW2),
            black_keys: vec![BLACK_BASE],
            root: BLACK_BASE,
            depth: 0,
            offset: Point{x:0,y:0},
            age: 0,
        };
        // extend the tree so that increase_size() method can be called
        tree_data.root = tree_data.black_key(1);
        tree_data.depth = 1;
        // call increase depth so that the tree is at very least depth 2, useful for proper recursion
        tree_data.increase_depth();
        tree_data
    }
    fn black_key(&mut self, depth:usize) -> u128{
        //cached method of retreiving the black key for a particular tree level
        match self.black_keys.get(depth){
            Some(x)=>*x,
            None=>{
                let prev_key = self.black_key(depth-1);
                let cur_value = QuadTreeValue::from_array(&[prev_key;4]);
                let cur_key = cur_value.key();
                self.map.add(cur_key, QuadTreeNode{
                    v: cur_value,
                    forward_steps: 0,
                    forward_key: NULL_KEY,
                    set_count: 0,
                });
                self.black_keys.push(cur_key);
                cur_key
            },
        }
    }
    fn get_set_count(&self, d: &QuadTreeValue)->u64{
        if d.is_raw(){
            d.to_array().iter().map(|x|(*x as u64).count_ones() as u64).sum()
        }
        else{
            d.to_array().iter().map(|x|self.map.get(*x).unwrap().set_count).sum()
        }
    }
    fn add_array(&mut self, arr: [u128;4])->u128{
        let val = QuadTreeValue::from_array(&arr);
        let key = val.key();
        match self.map.get(key){
            None=>{
                self.map.add(key, QuadTreeNode{
                    v: val,
                    forward_key: NULL_KEY,
                    forward_steps: 0,
                    set_count: self.get_set_count(&val),
                });
            },
            Some(oldval)=>{}
        };
        key
    }
    fn increase_depth(&mut self){
        let l1m = self.map.get(self.root).unwrap().v.to_array();
        let bkeyd1 = self.black_key((self.depth-1) as usize);
        let smap = [
            bkeyd1, bkeyd1, bkeyd1, bkeyd1,
            bkeyd1, l1m[0], l1m[1], bkeyd1,
            bkeyd1, l1m[2], l1m[3], bkeyd1,
            bkeyd1, bkeyd1, bkeyd1, bkeyd1,
        ];
        let depth0map = [
            self.add_array(slice(&smap, 0, 0)), self.add_array(slice(&smap, 2, 0)),
            self.add_array(slice(&smap, 0, 2)), self.add_array(slice(&smap, 2, 2)),
        ];
        let newkey = self.add_array(depth0map);
        self.root = newkey;
        self.depth += 1;
        let magnitude = (8<<(self.depth-2)) as i64;
        self.offset = self.offset + Point{x:-magnitude,y:-magnitude};
    }
    fn is_black(&self, key: u128)->bool{
        key == 0 || self.map.get(key).unwrap().set_count == 0
    }
    pub fn step_forward(&mut self, n_steps: u64){
        while self.depth < 3{
            self.increase_depth();
        }
        let max_steps = 4 << (self.depth-1);
        let cur_steps = std::cmp::min(max_steps, n_steps);
        let steps_left = n_steps - cur_steps;
        let init_map = self.map.get(self.root).unwrap().v.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
        let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
        let transposed_map = transpose_quad(&arg_map);
        let has_white_on_border: bool = transposed_map.iter()
            .enumerate()
            .filter(|(i,_)|is_on_4x4_border(*i))
            .any(|(_,key)|!self.is_black(*key));
        if has_white_on_border{
            self.increase_depth();
            self.step_forward(n_steps);
        }
        else{
            self.increase_depth();
            let newkey = self.step_forward_rec(self.root, self.depth-1, cur_steps);
            self.root = newkey;
            self.depth -= 1;
            self.age += cur_steps;
            let magnitude = (8<<(self.depth-1)) as i64;
            self.offset = self.offset + Point{x:magnitude,y:magnitude};
            if steps_left != 0{
                self.step_forward(steps_left);
            }
        }
    }
    fn step_forward_rec(&mut self,key: u128, depth: u64, n_steps: u64) -> u128{
        let full_steps = 4<<depth;
        assert!(n_steps <= full_steps, "num steps requested greater than full step, logic inaccurate");
        let item = self.map.get(key).unwrap();
        if n_steps == item.forward_steps as u64 && item.forward_key != NULL_KEY{
            item.forward_key
        }
        else{
            let newkey = self.step_forward_compute_recursive(key, depth, n_steps);
            // update the forward_key with the new key
            if n_steps != 0{
                self.map.add(key, QuadTreeNode{
                    v: item.v,
                    forward_key: newkey,
                    forward_steps: n_steps,
                    set_count: item.set_count,
                });
                // self.sync_age(key);
            }
            newkey
        }
    }
    fn step_forward_compute_recursive(&mut self, key: u128, depth: u64, n_steps: u64) -> u128{
        let node = self.map.get(key).unwrap();
        let d = node.v;
        if d.is_raw(){
            assert_eq!(depth, 0);
            step_forward_raw(d.to_array(), n_steps)
        }
        else if node.set_count == 0{
            //if it is black, return a black key
            //TODO: check if there is a better way to do this....
            self.black_key((depth) as usize)
        }
        else{
            assert_ne!(depth, 0);
            assert!(n_steps <= (4<<depth));
            let init_map = d.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
            let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
            let mut transposed_map = transpose_quad(&arg_map);
            let finalarr = if n_steps == 0{
                slice(&transposed_map, 1, 1)
            }
            else{
                let next_iter_full_steps = 4<<(depth-1);
                for bt in 0..2{
                    let dt = std::cmp::min(next_iter_full_steps as i64,std::cmp::max(0, n_steps as i64-next_iter_full_steps*bt)) as u64;
                    let mut result = [NULL_KEY;16];
                    for x in 0..(3-bt){
                        for y in 0..(3-bt){
                            let k1 = self.add_array(slice(&transposed_map, x as usize,y as usize));
                            result[(y*4+x) as usize] = self.step_forward_rec(k1,depth-1,dt);
                        }
                    }
                    transposed_map = result;
                }
                slice(&transposed_map, 0, 0)
            };
            // need to add finald to the table so that downstream users can look up its children
            self.add_array(finalarr)
        }
    }
    fn add_deps_to_tree(orig_table:&LargeKeyTable<QuadTreeNode>, new_table: &mut LargeKeyTable<QuadTreeNode>, root: u128){
        // if not raw value
        if !node_is_raw(root) && new_table.get(root).is_none(){
            let mut node = orig_table.get(root).unwrap();
            for newroot in node.v.to_array().iter(){
                TreeData::add_deps_to_tree(orig_table, new_table, *newroot);
            }
            if node.forward_key != NULL_KEY && !node_is_raw( node.forward_key){
                TreeData::add_deps_to_tree(orig_table, new_table, node.forward_key);
            }
            new_table.add(root,node);
        }
    }
    pub fn pruned_tree(&self)->TreeData{
        let mut next_map = LargeKeyTable::new(self.map.table_size_log2);
        //make sure black keys are in new map
        TreeData::add_deps_to_tree(&self.map, &mut next_map, self.root);
        TreeData{
            map: next_map,
            black_keys: vec![BLACK_BASE],
            root: self.root,
            depth: self.depth,
            offset: self.offset,
            age: self.age,
        }
    }
    pub fn serialize_treerepr(&self)->Vec<u8>{    
        const SERIAL_SIZE:usize = std::mem::size_of::<(u128,QuadTreeNode)>();
        const HEADER_SIZE:usize = 8*8;
        let mut res: Vec<u8> = Vec::with_capacity(self.map.len()*SERIAL_SIZE+HEADER_SIZE);
        serialize::serialize_transmutable::<u128>(&mut res, self.root);
        serialize::serialize_transmutable::<Point>(&mut res, self.offset);
        serialize::serialize_transmutable::<u64>(&mut res, self.depth);
        serialize::serialize_transmutable::<u64>(&mut res, self.map.len() as u64);
        serialize::serialize_transmutable::<u64>(&mut res, self.age);
        self.map.iter(&mut|key,value|{
            serialize::serialize_transmutable::<(u128,QuadTreeNode)>(&mut res, (*key, *value));
            true
        });
        //no need to serialize black keys, easy enough to recompute, already in tree.
        res
    }
    pub fn deserialize_treerepr(data: &[u8])->TreeData{
        let mut dataiter = data.iter();
        let root = serialize::deserialize_transmutable::<u128>(&mut dataiter).unwrap();
        let offset = serialize::deserialize_transmutable::<Point>(&mut dataiter).unwrap();
        let depth = serialize::deserialize_transmutable::<u64>(&mut dataiter).unwrap();
        let length = serialize::deserialize_transmutable::<u64>(&mut dataiter).unwrap();
        let age = serialize::deserialize_transmutable::<u64>(&mut dataiter).unwrap();
        let capacity_log2 = 64 - (length+1).leading_zeros() + 1;
        let mut new_map = LargeKeyTable::new(capacity_log2 as u8);
        for _ in 0..length{
            let (key, value) = serialize::deserialize_transmutable::<(u128,QuadTreeNode)>(&mut dataiter).unwrap();
            new_map.add(key, value);
        }
        TreeData{
            map: new_map,
            black_keys: vec![BLACK_BASE],
            root: root,
            depth: depth,
            offset: offset,
            age: age,
        }
    }   

    fn gather_points_recurive(&mut self, prev_map: &HashMap<Point, u128>, depth: usize) -> HashMap<Point, u128>{
        let mut map: HashMap<Point, u128> = HashMap::new();
        for oldp in prev_map.keys(){
            let newp = parent_point(*oldp);
            match map.entry(newp){
                //ignore the occupied case, as it means the value has already been filled
                Entry::Occupied(_)=>{},
                //if the entry is vacant, fill it entirely
                Entry::Vacant(entry)=>{
                    let child_keys = child_points(newp).map(|childp|
                        match prev_map.get(&childp) {
                            None=>self.black_key(depth-1),
                            Some(key)=>*key,
                        }
                    );
                    let value = QuadTreeValue::from_array(&child_keys);
                    let key = value.key();
                    self.map.add(key,QuadTreeNode{
                        v: value,
                        forward_key: NULL_KEY,
                        forward_steps: 0,
                        set_count: self.get_set_count(&value),
                    });
                    entry.insert(key);
                }
            }
        }
        map
    }

    pub fn gather_all_points(points: &Vec<Point>)->TreeData{
        let mut cur_map = gather_raw_points(&points);
        let mut tree = TreeData::new();
        let mut depth:u64 = 0;
        while cur_map.len() > 1 || depth < 3{
            depth += 1;
            cur_map = tree.gather_points_recurive(&cur_map, depth as usize);
        }
        let magnitude = (8<<(depth-1)) as i64;
        let rootp = *cur_map.keys().next().unwrap();
        tree.root = *cur_map.values().next().unwrap();
        tree.depth = depth;
        tree.offset = rootp.times(magnitude);
        tree
    }
    pub fn num_live_cells(&self)->u64{
        self.map.get(self.root).unwrap().set_count
    }
    pub fn hash_count(&self)->usize{
        self.map.len()
    }
    pub fn get_age(&self)->u64{
        self.age
    }
    
        
    fn iter_grayscale_points<F>(&self, root: u128, depth: i64, cur_loc: Point, fun:&mut F)
    where
        F: FnMut(i64,Point,u64)->bool
    {
        // let area =  (1 as u64)<<(2*(depth+3));
        if depth <= -3{
            let count = (root as u64) & 1;
            fun(depth, cur_loc, count);
        }
        else if depth <= 0{
            assert!(node_is_raw(root));
            let min_depth = -3;
            let val = root as u64;
            let magnitude = 1<<(depth+2);
            // dsize*dsize, but the compiler optimizes the division better
            if fun(depth, cur_loc, val.count_ones() as u64) && depth > min_depth {
                for y in 0..2{
                    for x in 0..2{
                        let offset = Point{x:x, y:y}.times(magnitude); 
                        self.iter_grayscale_points(get_subchunk(val, depth, x as u8, y as u8) as u128, depth-1,cur_loc+offset, fun);
                    }
                }
            }
        }
        else{
            assert!(!node_is_raw(root));
            let magnitude = 1<<(depth+2);
            let subvalue = self.map.get(root).unwrap();
            if fun(depth, cur_loc, subvalue.set_count){
                for (i, subnode) in subvalue.v.to_array().iter().enumerate(){
                    let offset = Point{
                        x:((i%2) as i64),
                        y:((i/2) as i64),
                    }.times(magnitude);
                    self.iter_grayscale_points(*subnode, depth-1,cur_loc+offset, fun);
                }
            }
        }
    }
    pub fn dump_all_points(&self) -> Vec<Point>{
        let mut res: Vec<Point> = Vec::new();
        self.iter_grayscale_points(self.root, self.depth as i64, self.offset, &mut|depth,p,count|{
            if count == 0{
                return false;
            }
            if depth == -3{
                res.push(p);
            }
            return true;
        });
        res
    }
    
    pub fn make_grayscale_map(&self, offset:Point, xsize: usize, ysize: usize, zoom: u8, brightness: f64) -> Vec<u8> {
        assert!(zoom >= 0);
        let mut res: Vec<u8> = Vec::new();
        res.resize(xsize*ysize, 0);
        const B2: u8 = 16;
        let brightness_int = (brightness * (1<<B2) as f64) as u64;
        self.iter_grayscale_points(self.root, self.depth as i64, offset.neg() + self.offset, &mut|depth,p,count|{
            let relmag:i64 = 1<<(depth+3 - zoom as i64);
            let t = p.div(1<<zoom);
            if count == 0{
                false
            }
            else if t.x >= xsize as i64 || t.y >= ysize as i64 || t.x + relmag <= 0 || t.y + relmag <= 0{
                false
            }
            else if zoom as i64 >= depth+3{
                let area_log2 = zoom*2;
                res[(t.y*(xsize as i64)+t.x) as usize] = std::cmp::min(255, (255*brightness_int*count) >> (B2 + area_log2)) as u8;
                false
            }
            else{
                true
            }
        });
        res
    }
}

fn point_8x8_loc(p: Point) -> u8{
    ((p.y % 8)*8 + (p.x % 8)) as u8
}
fn set_bit(bitidx: u8) -> u64{
    (1 as u64) << bitidx
}
fn gather_raw_points(points: &Vec<Point>) -> HashMap<Point, u128>{
    let mut map: HashMap<Point, u128> = HashMap::new();
    for p in points.iter(){
        let ploc = Point{x:p.x/8,y:p.y/8};
        *map.entry(ploc).or_insert(0) |= set_bit(point_8x8_loc(*p)) as u128;
    }
    map
}
fn parent_point(p:Point) -> Point {
    Point{x:p.x/2,y:p.y/2}
}
fn child_points(p:Point) -> [Point;4] {
    [
        Point{x:p.x*2+0, y:p.y*2+0},
        Point{x:p.x*2+1, y:p.y*2+0},
        Point{x:p.x*2+0, y:p.y*2+1},
        Point{x:p.x*2+1, y:p.y*2+1},
    ]
}
