use rand::{SeedableRng, Rng};

fn main() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let _ = rng.gen_range(0..10);
}
