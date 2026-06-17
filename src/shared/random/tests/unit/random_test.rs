use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

#[test]
fn same_seed_produces_same_sequence() {
    let mut a = StdRng::seed_from_u64(42);
    let mut b = StdRng::seed_from_u64(42);
    for _ in 0..100 {
        assert_eq!(a.random::<u64>(), b.random::<u64>());
    }
}

#[test]
fn different_seeds_produce_different_sequences() {
    let mut a = StdRng::seed_from_u64(42);
    let mut b = StdRng::seed_from_u64(43);
    assert_ne!(a.random::<u64>(), b.random::<u64>());
}

#[test]
fn gen_range_works_correctly() {
    let mut rng = StdRng::seed_from_u64(99);
    for _ in 0..100 {
        let val = rng.random_range(1..=20);
        assert!((1..=20).contains(&val));
    }
}

#[test]
fn gen_bool_works_correctly() {
    let mut rng = StdRng::seed_from_u64(7);
    let mut had_true = false;
    let mut had_false = false;
    for _ in 0..1000 {
        if rng.random_bool(0.5) {
            had_true = true;
        } else {
            had_false = true;
        }
    }
    assert!(had_true && had_false);
}

#[test]
fn fill_bytes_deterministic() {
    let mut a = StdRng::seed_from_u64(0);
    let mut b = StdRng::seed_from_u64(0);
    let mut buf_a = [0u8; 32];
    let mut buf_b = [0u8; 32];
    a.fill(&mut buf_a);
    b.fill(&mut buf_b);
    assert_eq!(buf_a, buf_b);
}
