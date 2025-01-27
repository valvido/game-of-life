/*
Fast storage and lookup table for 
well distributed keys of size u128.

Perfect for lookups where the key
is a hash.
*/

use std::boxed::Box;
use crate::typedarena::Arena;


#[derive(Clone,Copy, Default)]
struct HashNodeData<T:Copy + Default>{
    key: u128,
    value: T,
}

pub struct LargeKeyTable<T: Copy + Default>{
    table: Vec<Option<std::ptr::NonNull<HashNodeData<T>>>>,
    arena: Arena<HashNodeData<T>>,
    n_elements: usize,
    lookup_mask: usize, 
    pub table_size_log2: u8,
}
enum PossibleIdx {
    Found(usize),
    Empty(usize),
}

fn get_idx<T:Copy + Default>(table: &Vec<Option<std::ptr::NonNull<HashNodeData<T>>>>, lookup_mask: usize, key: u128) -> PossibleIdx{
    let upkey = (key >> 64) as u64;
    let mut curkey = key >> 24;
    //quadratic probing for now
    let mut curoffset: usize = 0;
    loop {
        let idx = ((key as usize) + curoffset) & lookup_mask;
        let entry = &table[idx];
        match entry{
            None=>{
                return PossibleIdx::Empty(idx);
            },
            Some(ptr)=>{
                let nodekey = unsafe{ptr.as_ref().key};
                if (key == nodekey){
                    return PossibleIdx::Found(idx);
                }
            }
        }
        curoffset += (curkey as usize) & 0xff;
        curkey >>= 1;
    }
}
impl<T: Copy + Default> LargeKeyTable<T>{
    pub fn new(initial_capacity_log2:u8) -> LargeKeyTable<T>{
        let next_size = 1 << initial_capacity_log2;
        // twos-compliment masking
        let next_mask = next_size - 1;
        LargeKeyTable{
            table: vec![None;next_size],
            arena: Arena::new(),
            n_elements: 0,
            lookup_mask: next_mask,
            table_size_log2: initial_capacity_log2,
        }
    }
    pub fn len(&self)->usize{
        self.n_elements
    }
    pub fn get_idx(&self, key:u128)->PossibleIdx{
        get_idx(&self.table, self.lookup_mask, key)
    }
    pub fn get(&self, key: u128) -> Option<T>{
        match self.get_idx(key){
            PossibleIdx::Found(idx)=>Some(unsafe{self.table[idx].unwrap().as_ref().value}),
            PossibleIdx::Empty(_)=>None
        }
    }
    fn _grow(&mut self){
        self.table_size_log2 += 1;
        let next_size = 1<<self.table_size_log2;
        self.table = vec![None;next_size];
        self.lookup_mask = next_size - 1;
        for entry in self.arena.iter_mut(){
            match get_idx(&self.table, self.lookup_mask, entry.key){
                PossibleIdx::Found(idx)=>{
                    panic!("should not have found index when growing!");
                },
                PossibleIdx::Empty(idx)=>{
                    self.table[idx] = std::ptr::NonNull::new(entry);
                }
            }
        }
    }
    pub fn add(&mut self, key: u128, value: T){
        match self.get_idx(key){
            PossibleIdx::Found(idx)=>{
                unsafe{self.table[idx].unwrap().as_mut().value = value;};
            },
            PossibleIdx::Empty(idx)=>{
                self.n_elements += 1;
                self.table[idx] = std::ptr::NonNull::new(self.arena.alloc(HashNodeData{
                    key: key,
                    value: value,
                }));
                if self.n_elements >= self.table.len()/2{
                    self._grow();
                }
            }
        }
    }
    // pub fn iter_mut<F>(&mut self, func: &mut F)
    // where
    //     F: FnMut(&u128, &mut T)
    // {
    //     for item in self.table.iter_mut(){
    //         if item.val.is_some(){
    //             let val = item.val.as_mut().unwrap();
    //             func(&val.key, &mut val.value);
    //         }
    //     }
    // }
    
    pub fn iter<F>(& self, func: &mut F)
    where
        F: FnMut(&u128, &T)->bool
    {
        for item in self.arena.iter(){
            if !func(&item.key, &item.value) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_hash_insertions(){
        let basekey:u128 = 0x8fab04dd8336fe8b33e4424a0d9e3e97;
        let mut table: LargeKeyTable<i32> = LargeKeyTable::new(1);
        const MAX_CHECK: usize = 5;
        for i in 0..MAX_CHECK{
            table.add(basekey.wrapping_mul((i*i) as u128), i as i32);
        }
        let mut x = 0;
        for j in 0..MAX_CHECK*MAX_CHECK{
            assert_eq!(table.get(basekey.wrapping_mul(j as u128)).is_some(), x*x == j);
            if x*x == j{
                x += 1;
            }
        }
    }
}