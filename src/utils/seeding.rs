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
/// // Generates a PRNG using a random seed derived from the OS.
/// rand_random(None)
///
/// // Generates a PRNG using the given seed.
/// let (generator, seed_no) = rand_random(64)
/// assert_eq!(seed_no, 64)
/// ```
pub fn rand_random(seed: Option<u64>) -> (Pcg64, u64) {
    let seed_no = seed.unwrap_or((&mut thread_rng()).gen());
    let generator = Pcg64::seed_from_u64(seed_no);

    return (generator, seed_no);
}
