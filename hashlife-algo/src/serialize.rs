use std::mem::{transmute, size_of};

pub fn serialize_transmutable<T:Copy>(vec: &mut Vec<u8>, x:T){
    unsafe{
        let N_BYTES:usize = size_of::<T>();
        let x_ptr: *const T = &x;
        let byte_ptr: *const u8 = transmute(x_ptr);
        
        for i in 0..N_BYTES{
            vec.push(*byte_ptr.add(i));
        }
    }
}
pub fn deserialize_transmutable<'a,T:Copy + Default>(dataiter: &mut core::slice::Iter<'a,u8>)->Option<T>{
    unsafe{
        let mut res: T = Default::default();
        let N_BYTES:usize = size_of::<T>();
        let x_ptr: *mut T = &mut res;
        let byte_ptr: *mut u8 = transmute(x_ptr);
        for i in 0..N_BYTES{
            match dataiter.next(){
                Some(v)=>{
                    *byte_ptr.add(i) = *v;
                },
                None=>{
                    return None
                }
            }
        }
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    
    fn make_vec_from<T:Copy>(x:T)->Vec<u8>{
        let mut res: Vec<u8> = Vec::new();
        serialize_transmutable(&mut res, x);
        res
    }
    #[test]
    fn test_serialize() {
        let val: u64 = 0xfc;
        assert_eq!(deserialize_transmutable::<u64>(&mut make_vec_from(val).iter()).unwrap(), val);
    }
    #[test]
    fn test_serialize_tuple() {
        let val: (u64,u8,u64) = (0xffc,0xf,0xccc);
        assert_eq!(deserialize_transmutable::<(u64,u8,u64)>(&mut make_vec_from(val).iter()).unwrap(), val);
    }
}