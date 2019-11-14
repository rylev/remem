use std::sync::{Arc, Mutex};

pub struct Pool<T> {
    internal: Arc<Mutex<InternalPool<T>>>,
}

impl<T> Pool<T> {
    pub fn new<C, D>(creation: C, clearance: D) -> Pool<T>
    where
        C: Fn() -> T + 'static,
        D: Fn(&mut T) -> () + 'static,
    {
        Pool {
            internal: Arc::new(Mutex::new(InternalPool::new(creation, clearance))),
        }
    }

    pub fn get<'a>(&'a self) -> ItemGuard<'a, T> {
        let mut pool = self.internal.lock().unwrap();
        let item = if pool.free.is_empty() {
            (*pool.creation)()
        } else {
            pool.free.pop().unwrap()
        };
        ItemGuard {
            item: Some(item),
            pool: self,
        }
    }

    pub fn reintroduce(&self, mut item: T) {
        let mut pool = self.internal.lock().unwrap();
        (*pool.clearance)(&mut item);
        pool.free.push(item);
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            internal: self.internal.clone(),
        }
    }
}

struct InternalPool<T> {
    free: Vec<T>,
    creation: Box<dyn Fn() -> T>,
    clearance: Box<dyn Fn(&mut T)>,
}

impl<T> InternalPool<T> {
    pub fn new<C, D>(creation: C, clearance: D) -> Self
    where
        C: Fn() -> T + 'static,
        D: Fn(&mut T) -> () + 'static,
    {
        InternalPool {
            free: Vec::new(),
            creation: Box::new(creation),
            clearance: Box::new(clearance),
        }
    }
}

pub struct ItemGuard<'a, T> {
    item: Option<T>,
    pool: &'a Pool<T>,
}

impl<'a, T> Drop for ItemGuard<'a, T> {
    fn drop(&mut self) {
        self.pool.reintroduce(self.item.take().unwrap())
    }
}

impl<'a, T> std::ops::Deref for ItemGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for ItemGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let pool = Pool::<Vec<u8>>::new(
            || {
                println!("Allocating new memory");
                Vec::new()
            },
            |v| v.clear(),
        );
        let mut item = pool.get();
        let mut _item2 = pool.get();

        item.push(1);
        item.push(2);
        item.push(3);

        drop(item);

        let mut item = pool.get();

        item.push(1);
        item.push(2);
        item.push(3);
    }
}
