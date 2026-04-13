use rand::RngExt;
use rand::distr::Alphanumeric;

pub fn string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
