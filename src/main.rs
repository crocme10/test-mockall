use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};

#[cfg(test)]
mod mock;

// This trait is not object safe.
#[async_trait]
trait Generic {
    async fn generic_fn<'a, S: 'a>(&self, mut stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin + 'a;
}

// Implement the trait for boxed pointers to some type `T` which
// implements the trait.
#[async_trait]
impl<'a, T: ?Sized> Generic for Box<T>
where
    T: Generic + Send + Sync,
{
    async fn generic_fn<'b, S: 'b>(&self, stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin + 'b,
    {
        println!("Generic::generic for Box<T> T: Generic");
        (**self).generic_fn(stream).await
    }
}

/////////////////////////////////////////////////////////////////////
// This is an object-safe equivalent that interoperates seamlessly.

#[async_trait]
trait ErasedGeneric {
    // Replace the generic parameter with a trait object.
    async fn erased_fn<'a>(
        &self,
        stream: &'a mut (dyn Stream<Item = i32> + Send + Sync + Unpin + 'a),
    ) -> Result<i32, std::io::Error>;
}

// Impl the not-object-safe trait for a trait object of the
// object-safe trait.
#[async_trait]
impl Generic for (dyn ErasedGeneric + Send + Sync) {
    async fn generic_fn<'a, S>(&self, mut stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin + 'a,
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
    async fn erased_fn<'a>(
        &self,
        stream: &'a mut (dyn Stream<Item = i32> + Send + Sync + Unpin + 'a),
    ) -> Result<i32, std::io::Error> {
        println!("Erased::erased for T: Generic");
        self.generic_fn(stream).await
    }
}

#[tokio::main]
async fn main() {
    let stream = futures::stream::iter(vec![1, 3, 5, 7]);

    struct S;
    #[async_trait]
    impl Generic for S {
        async fn generic_fn<'a, S: 'a>(&self, stream: S) -> Result<i32, std::io::Error>
        where
            S: Stream<Item = i32> + Send + Sync + Unpin + 'a,
        {
            let sum: i32 = stream.collect::<Vec<i32>>().await.iter().sum();
            Ok(sum)
        }
    }

    // Construct a trait object.
    let trait_object: Box<dyn ErasedGeneric + Send + Sync> = Box::new(S);

    // Seamlessly invoke the generic method on the trait object.
    //
    // THIS LINE LOOKS LIKE MAGIC. We have a value of type trait
    // object and we are invoking a generic method on it.
    let res = trait_object.generic_fn(stream).await;
    println!("res: {}", res.expect("res"));
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MockGeneric<'a, T> {
        pub generate_index: mock::Mock<'a, (T,), Result<i32, std::io::Error>>,
    }

    #[cfg(test)]
    impl<'a, T> MockGeneric<'a, T> {
        fn new() -> Self {
            MockGeneric {
                generate_index: mock::Mock::new(Ok(43)),
            }
        }
    }

    #[cfg(test)]
    #[async_trait]
    impl<'a, T> Generic for MockGeneric<'a, T>
    where
        T: From<&'a (dyn Stream<Item = i32> + Send + Sync + Unpin + 'a)> + Sync,
    {
        async fn generic_fn<'b, S: 'b>(&self, stream: S) -> Result<i32, std::io::Error>
        where
            S: Stream<Item = i32> + Send + Sync + Unpin + 'b,
        {
            self.generate_index.call((T::from(&stream),))
        }
    }
}
