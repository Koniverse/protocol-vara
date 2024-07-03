use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};
use sails_rtl::{prelude::*, ActorId};

#[test]
fn test_create_pool() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pools(&invariant, u8::MAX, 0).unwrap(),
        vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()]
    );

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).expect("Pool doesn't exist");
    let expected_pool = Pool {
        sqrt_price: init_sqrt_price,
        current_tick_index: init_tick,
        fee_receiver: ActorId::from(ADMIN),
        last_timestamp: sys.block_timestamp(),
        start_timestamp: sys.block_timestamp(),
        ..Pool::default()
    };
    assert_eq!(pool, expected_pool);
}

#[test]
fn test_create_pool_x_to_y_and_y_to_x() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::PoolAlreadyExist);

    assert_eq!(
        get_pools(&invariant, u8::MAX, 0).unwrap(),
        vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()]
    );
}
#[test]
fn test_create_pool_with_same_tokens() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_0,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::TokensAreSame);

    assert_eq!(get_pools(&invariant, u8::MAX, 0).unwrap(), vec![]);
}

#[test]
fn test_create_pool_fee_tier_not_added() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::FeeTierNotFound);

    assert_eq!(get_pools(&invariant, u8::MAX, 0).unwrap(), vec![]);
}

#[test]
fn test_create_pool_init_tick_not_divisible_by_tick_spacing() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 2;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitTick);

    assert_eq!(get_pools(&invariant, u8::MAX, 0).unwrap(), vec![]);
}

#[test]
fn test_create_pool_init_sqrt_price_minimal_difference_from_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap() + SqrtPrice::new(U128::from(1));

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        init_tick
    );
}

#[test]
fn test_create_pool_init_sqrt_price_has_closer_init_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 2;
    let init_sqrt_price = SqrtPrice::new(U128::from(1000175003749000000000000u128));

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitSqrtPrice);

    let correct_tick_index = 3;
    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_tick_index,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        correct_tick_index
    );
}

#[test]
fn test_create_pool_init_sqrt_price_has_closer_init_tick_spacing_over_one() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 0;
    let init_sqrt_price = SqrtPrice::new(U128::from(1000225003749000000000000u128));

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitSqrtPrice);

    let correct_tick_index = 3;
    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_tick_index,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        correct_tick_index
    );
}
