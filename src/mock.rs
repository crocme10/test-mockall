// use std::fmt;
// use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

type OptionalRef<T> = Arc<RwLock<Option<T>>>;

/// C: Function
/// R: Return value
pub struct Mock<C, R> {
    return_value: Arc<RwLock<R>>,
    mock_fn: OptionalRef<fn(C) -> R>,
    mock_closure: OptionalRef<Box<Fn(C) -> R + Send + Sync>>,
    calls: Arc<RwLock<Vec<C>>>,
}

impl<C, R> Mock<C, R> {
    pub fn new<T: Into<R>>(return_value: T) -> Self {
        Mock {
            return_value: Arc::new(RwLock::new(return_value.into())),
            mock_fn: Arc::new(RwLock::new(None)),
            mock_closure: Arc::new(RwLock::new(None)),
            calls: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn call(&self, args: C) -> R {
        self.calls.write().unwrap().push(args);

        if let Some(ref mock_fn) = *self.mock_fn.read().unwrap() {
            return mock_fn(args);
        }

        if let Some(ref mock_closure) = *self.mock_closure.read().unwrap() {
            return mock_closure(args);
        }

        Arc::clone(&self.return_value)
    }
}
