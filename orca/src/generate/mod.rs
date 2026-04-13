use rand::distr::Alphanumeric;
use rand::RngExt;

pub fn string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
