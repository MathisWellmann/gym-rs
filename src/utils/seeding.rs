use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64;

/// Generates a PRNG using the PCG64 algorithm.
///
/// Returns the PRNG instance along with the seed number used to initiate the
/// generator.
///
/// # Examples
///
/// ```rust
/// use gym_rs::utils::seeding::rand_random;
///
/// // Generates a PRNG using a random seed derived from the OS.
/// rand_random(None);
///
/// // Generates a PRNG using the given seed.
/// let (generator, seed_no) = rand_random(Some(64));
/// assert_eq!(seed_no, 64);
/// ```
pub fn rand_random(seed: Option<u64>) -> (Pcg64, u64) {
    let seed_no = seed.unwrap_or(thread_rng().gen());
    let generator = Pcg64::seed_from_u64(seed_no);

    (generator, seed_no)
}

#[cfg(test)]
mod tests {
    use super::rand_random;

    // NOTE: The negative case cannot be tested.
    #[test]
    fn given_seed_when_rand_random_then_generator_is_created_using_seed() {
        let seed_no = 42;
        let (_generator, generator_seed) = rand_random(Some(seed_no));

        assert_eq!(seed_no, generator_seed);
    }
}
