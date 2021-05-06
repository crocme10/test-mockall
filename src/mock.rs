#[cfg(test)]
use std::marker::PhantomData;

#[cfg(test)]
pub struct Mock<'a, C: 'a, R> {
    retval: R,
    mock_fn: Option<fn(C) -> R>,
    calls: Vec<C>,
    phantom: PhantomData<&'a C>,
}

#[cfg(test)]
impl<'a, C, R> Mock<'a, C, R> {
    pub fn new<T: Into<R>>(retval: T) -> Self {
        Mock {
            retval: retval.into(),
            mock_fn: None,
            calls: vec![],
            phantom: PhantomData,
        }
    }

    pub fn return_value<T: Into<R>>(&mut self, retval: T) {
        self.retval = retval.into();
    }

    pub fn call(&self, args: C) -> R {
        self.calls.push(args);

        if let Some(mock_fn) = self.mock_fn {
            return mock_fn(args);
        }

        self.retval
    }
}
