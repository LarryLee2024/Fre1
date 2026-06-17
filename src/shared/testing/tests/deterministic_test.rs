use crate::shared::testing::deterministic::DeterministicRng;

#[test]
fn same_seed_produces_same_sequence() {
    let mut a = DeterministicRng::with_seed(42);
    let mut b = DeterministicRng::with_seed(42);
    for _ in 0..100 {
        assert_eq!(a.gen_range(1, 21), b.gen_range(1, 21));
    }
}

#[test]
fn different_seeds_produce_different_sequences() {
    let mut a = DeterministicRng::with_seed(42);
    let mut b = DeterministicRng::with_seed(43);
    assert_ne!(a.gen_range(1, 1001), b.gen_range(1, 1001));
}

#[test]
fn default_seed_is_42() {
    let mut a = DeterministicRng::new();
    let mut b = DeterministicRng::with_seed(42);
    assert_eq!(a.gen_range(0, 100), b.gen_range(0, 100));
}
