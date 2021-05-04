use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
// This code taken from David Tolnay's erased-serde library,

// This trait is not object safe.
#[async_trait]
#[cfg_attr(test, mockall::automock)]
trait Generic {
    async fn generic_fn<S>(&self, mut stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin + 'static;
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
        S: Stream<Item = i32> + Send + Sync + Unpin + 'static,
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
        S: Stream<Item = i32> + Send + Sync + Unpin + 'static,
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

#[tokio::main]
async fn main() {
    let stream = futures::stream::iter(vec![1, 3, 5, 7]);

    struct S;
    #[async_trait]
    impl Generic for S {
        async fn generic_fn<S>(&self, stream: S) -> Result<i32, std::io::Error>
        where
            S: Stream<Item = i32> + Send + Sync + Unpin + 'static,
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
