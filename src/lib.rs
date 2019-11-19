use std::sync::Arc;
use treiber_stack::TreiberStack as Stack;

mod treiber_stack;

struct Internal<T> {
    stack: Stack<T>,
    create: Box<dyn Fn() -> T + Send + Sync>,
    clear: Box<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> Internal<T> {
    fn new<C, D>(create: C, clear: D) -> Self
    where
        C: Fn() -> T + Send + Sync + 'static,
        D: Fn(&mut T) -> () + Send + Sync + 'static,
    {
        Internal {
            stack: Stack::new(),
            create: Box::new(create),
            clear: Box::new(clear),
        }
    }
}

/// A pool of reusable memory.
pub struct Pool<T> {
    internal: Arc<Internal<T>>,
}

impl<T> Pool<T> {
    /// Create a new Pool from an initializer function.
    pub fn new<C>(create: C) -> Pool<T>
    where
        C: Fn() -> T + Send + Sync + 'static,
    {
        Pool {
            internal: Arc::new(Internal::new(create, |_| {})),
        }
    }

    /// Create a new Pool from an initializer function and a clear function.
    pub fn with_clear<C, D>(create: C, clear: D) -> Pool<T>
    where
        C: Fn() -> T + Send + Sync + 'static,
        D: Fn(&mut T) -> () + Send + Sync + 'static,
    {
        Pool {
            internal: Arc::new(Internal::new(create, clear)),
        }
    }

    /// Get an item from the pool.
    pub fn get<'a>(&'a self) -> ItemGuard<'a, T> {
        let pool = &self.internal;
        let item = pool.stack.pop();
        ItemGuard {
            item: Some(item.unwrap_or_else(|| (*self.internal.create)())),
            pool: self,
        }
    }

    /// Store an item back inside the pool.
    fn reintroduce(&self, mut item: T) {
        (*self.internal.clear)(&mut item);
        self.internal.stack.push(item);
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            internal: self.internal.clone(),
        }
    }
}

/// RAII structure used to reintroduce an item into the pool when dropped.
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
