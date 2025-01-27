
pub struct Arena<T>{
    data: Vec<Vec<T>>,
}

impl<T> Arena<T>{
    pub fn new()->Arena<T>{
        Arena{
            data:vec![Vec::with_capacity(1)],
        }
    }
    pub fn has_capacity(&self)->bool{
        let last = self.data.last().unwrap();
        last.capacity() > last.len()
    }
    pub fn alloc(&mut self, initval: T)->&mut T{
        if self.has_capacity(){
            let last = self.data.last_mut().unwrap();
            last.push(initval);
            last.last_mut().unwrap()
        }
        else{
            let old_capacity = self.data.last().unwrap().capacity();
            let new_capacity = ((old_capacity * 4) / 3) + 5;
            self.data.push(Vec::with_capacity(new_capacity));
            let newlast = self.data.last_mut().unwrap();
            newlast.push(initval);
            newlast.last_mut().unwrap()
        }
    }
    pub fn iter(&self)-> core::iter::Flatten<std::slice::Iter<'_, Vec<T>>>{
        self.data.iter().into_iter().flatten()
    }
    pub fn iter_mut(&mut self)->core::iter::Flatten<std::slice::IterMut<'_, Vec<T>>>{
        self.data.iter_mut().into_iter().flatten()
    }
}