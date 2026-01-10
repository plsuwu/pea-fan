use std::sync::LazyLock;

use tinyrand::{Rand, RandRange, StdRand, Wyrand};
use tokio::sync::OnceCell;

pub fn next() -> u32 {
    let mut rand = StdRand::default();
    let val = (rand.next_u32() % 500) as f32 * 0.01;

    println!("val: {}", val);

    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rand_jitter() {
        let val = next();
    }
}
