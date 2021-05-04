use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};

// This trait is not object safe.
#[async_trait]
pub trait Generic {
    async fn generic_fn<S>(&self, mut stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin;
}

// Implement the trait for boxed pointers to some type `T` which
// implements the trait.
#[async_trait]
impl<'a, T: ?Sized> Generic for Box<T>
where
    T: Generic + Send + Sync,
{
    async fn generic_fn<S>(&self, stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin,
    {
        println!("Generic::generic for Box<T> T: Generic");
        (**self).generic_fn(stream).await
    }
}

/////////////////////////////////////////////////////////////////////
// This is an object-safe equivalent that interoperates seamlessly.

#[async_trait]
pub trait ErasedGeneric {
    // Replace the generic parameter with a trait object.
    async fn erased_fn(
        &self,
        stream: &mut (dyn Stream<Item = i32> + Send + Sync + Unpin),
    ) -> Result<i32, std::io::Error>;
}

// Impl the not-object-safe trait for a trait object of the
// object-safe trait.
#[async_trait]
impl Generic for (dyn ErasedGeneric + Send + Sync) {
    async fn generic_fn<S>(&self, mut stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin,
    {
        println!("Generic::generic for dyn Erased");
        self.erased_fn(&mut stream).await
    }
}

// If `T` impls the not-object-safe trait, it impls the
// object-safe trait too.
#[async_trait]
impl<T> ErasedGeneric for T
where
    T: Generic + Send + Sync,
{
    async fn erased_fn(
        &self,
        stream: &mut (dyn Stream<Item = i32> + Send + Sync + Unpin),
    ) -> Result<i32, std::io::Error> {
        println!("Erased::erased for T: Generic");
        self.generic_fn(stream).await
    }
}

#[cfg(test)]
mod tests {
    use crate::mock;
    use futures::stream::{Stream, StreamExt};

    struct MockGeneric<S: Stream<Item = i32> + Send + Sync + Unpin> {
        pub generic_fn: mock::Mock<S, Result<i32, std::io::Error>>,
    }
}
