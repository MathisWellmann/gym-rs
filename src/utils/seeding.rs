use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64;

pub fn random(seed: Option<u64>) -> (Pcg64, u64) {
    let seed_no = seed.unwrap_or((&mut thread_rng()).gen());
    let mut generator = Pcg64::seed_from_u64(seed_no);

    return (generator, seed_no);
}
