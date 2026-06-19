use crate::core::capabilities::runtime::replay::foundation::{
    DeterministicRng as CoreDeterministicRng, RngSeeds, RngStream,
};

/// 不变量：相同种子 + 相同调用次数 → 相同结果。
#[test]
fn rng_output_is_deterministic_across_instances() {
    let seeds = RngSeeds::new(12345, 67890, 11111, 22222);

    let mut rng_a = CoreDeterministicRng::new(seeds);
    let mut rng_b = CoreDeterministicRng::new(seeds);

    for stream in RngStream::all() {
        for _ in 0..100 {
            let a = rng_a.next_u64(stream);
            let b = rng_b.next_u64(stream);
            assert_eq!(a, b, "RNG output must be deterministic for {:?}", stream);
        }
    }
}

/// 不变量：不同 RNG 流产生不同输出序列。
#[test]
fn rng_streams_produce_different_sequences() {
    let seeds = RngSeeds::uniform(42);
    let mut rng = CoreDeterministicRng::new(seeds);

    let combat_val = rng.next_u64(RngStream::Combat);
    let drop_val = rng.next_u64(RngStream::Drop);
    let ai_val = rng.next_u64(RngStream::AI);
    let world_val = rng.next_u64(RngStream::World);

    // 各流使用不同偏置，输出应大概率不同
    // 虽不能保证 100% 唯一但应为真
    assert!(
        combat_val != drop_val || combat_val != ai_val || combat_val != world_val,
        "streams should produce different values (unlikely collision)"
    );
}

/// 不变量：gen_range 返回的值在指定范围内。
#[test]
fn rng_gen_range_within_bounds() {
    let seeds = RngSeeds::uniform(42);
    let mut rng = CoreDeterministicRng::new(seeds);

    for _ in 0..1000 {
        let val = rng.gen_range(RngStream::Combat, 5, 10);
        assert!(
            (5..10).contains(&val),
            "gen_range(5,10) = {} should be in [5,10)",
            val
        );
    }
}

/// 不变量：gen_bool 返回 true 的概率语义正确（种子已知时结果确定）。
#[test]
fn rng_gen_bool_deterministic() {
    let seeds = RngSeeds::uniform(42);
    let mut rng_a = CoreDeterministicRng::new(seeds);
    let mut rng_b = CoreDeterministicRng::new(seeds);

    for _ in 0..100 {
        assert_eq!(
            rng_a.gen_bool(RngStream::Combat, 0.5),
            rng_b.gen_bool(RngStream::Combat, 0.5),
            "gen_bool must be deterministic"
        );
    }
}

/// 不变量：seed 变更后 RNG 输出完全不同。
#[test]
fn rng_different_seed_different_output() {
    let seeds_a = RngSeeds::uniform(1);
    let seeds_b = RngSeeds::uniform(2);

    let mut rng_a = CoreDeterministicRng::new(seeds_a);
    let mut rng_b = CoreDeterministicRng::new(seeds_b);

    let mut any_different = false;
    for _ in 0..10 {
        if rng_a.next_u64(RngStream::Combat) != rng_b.next_u64(RngStream::Combat) {
            any_different = true;
            break;
        }
    }
    assert!(
        any_different,
        "different seeds should produce different output"
    );
}

/// 不变量：set_all_seeds 重置计数器后与新建实例一致。
#[test]
fn rng_set_all_seeds_resets_to_initial_state() {
    let seeds = RngSeeds::uniform(42);
    let mut rng = CoreDeterministicRng::new(seeds);

    // 消耗一些随机数
    for _ in 0..50 {
        rng.next_u64(RngStream::Combat);
    }

    // 重置种子
    rng.set_all_seeds(seeds);

    // 应该与新建实例的第 1 个输出一致
    let mut fresh = CoreDeterministicRng::new(seeds);
    assert_eq!(
        rng.next_u64(RngStream::Combat),
        fresh.next_u64(RngStream::Combat),
        "set_all_seeds should reset RNG state"
    );
}
