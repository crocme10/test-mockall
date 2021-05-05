use async_trait::async_trait;
use futures::stream::{Stream, StreamEx};

mod gen;
mod mock;


use crate::gen::{ErasedGeneric, Generic};

struct S;

#[async_trait]

impl Generic for S {
    async fn generic_fn<S>(&self, stream: S) -> Result<i32, std::io::Error>
    where
        S: Stream<Item = i32> + Send + Sync + Unpin,
    {
        let sum: i32 = stream.collect::<Vec<i32>>().await.iter().sum();
        Ok(sum)
    }
}

#[tokio::main]
async fn main() {
    let stream = futures::stream::iter(vec![1, 3, 5, 7]);

    // Construct a trait object.
    let trait_object: Box<dyn ErasedGeneric + Send + Sync> = Box::new(S);

    // Seamlessly invoke the generic method on the trait object.
    //
    // THIS LINE LOOKS LIKE MAGIC. We have a value of type trait
    // object and we are invoking a generic method on it.
    let res = trait_object.generic_fn(&mut stream).await;
    println!("res: {}", res.expect("res"));
}
