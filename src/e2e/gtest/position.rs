use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use s::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
    MIN_SQRT_PRICE,
};

#[test]
fn test_create_position() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500, 500));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, 500).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, 500).assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500,
        token_y,
        500,
        None::<&str>,
    )
    .unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: -10,
            upper_tick: 10,
            liquidity_delta: Liquidity::new(10),
            slippage_limit_lower: pool.sqrt_price,
            slippage_limit_upper: pool.sqrt_price,
        },
    );

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta: Liquidity::new(10),
                block_timestamp: sys.block_timestamp(),
                lower_tick: -10,
                upper_tick: 10,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: Liquidity::new(10),
                lower_tick_index: -10,
                upper_tick_index: 10,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));
}

#[test]
fn test_position_same_upper_and_lower_tick() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500, 500));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, 500).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, 500).assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500,
        token_y,
        500,
        None::<&str>,
    )
    .unwrap();

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: 10,
            upper_tick: 10,
            liquidity_delta: Liquidity::new(10),
            slippage_limit_lower: pool.sqrt_price,
            slippage_limit_upper: pool.sqrt_price,
        },
        InvariantError::InvalidTickIndex,
    );

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();
}

#[test]
fn test_position_below_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let initial_balance = 10_000_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -46080;
    let upper_tick_index = -23040;
    let liquidity_delta = Liquidity::from_integer(10_000);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_balance,
        token_y,
        initial_balance,
        None::<&str>,
    )
    .unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
    );
    let expected_x_increase = 0;
    let expected_y_increase = 2162;

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta,
                block_timestamp: sys.block_timestamp(),
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: liquidity_delta,
                lower_tick_index,
                upper_tick_index,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    let zero_fee = FeeGrowth::new(0);

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check pool
    assert_eq!(pool_state.liquidity, pool_state_before.liquidity);
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(
        (invariant_x, invariant_y),
        (expected_x_increase, expected_y_increase)
    );
}

#[test]
fn test_position_within_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let max_tick_test = 177_450;
    let min_tick_test = -max_tick_test;
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = min_tick_test + 10;
    let upper_tick_index = max_tick_test - 10;
    let liquidity_delta = Liquidity::from_integer(100);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_balance,
        token_y,
        initial_balance,
        None::<&str>,
    )
    .unwrap();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state_before.sqrt_price,
                slippage_limit_upper: pool_state_before.sqrt_price,
            },
        )
        .assert_success();

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());
    let zero_fee = FeeGrowth::new(0);
    let expected_x_increase = 317;
    let expected_y_increase = 32;

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert_eq!(pool_state.liquidity, liquidity_delta);
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(
        (invariant_x, invariant_y),
        (expected_x_increase, expected_y_increase)
    );
}

#[test]
fn test_position_above_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -22980;
    let upper_tick_index = 0;
    let liquidity_delta = Liquidity::from_integer(10_000);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_balance,
        token_y,
        initial_balance,
        None::<&str>,
    )
    .unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
    );

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta,
                block_timestamp: sys.block_timestamp(),
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: liquidity_delta,
                lower_tick_index,
                upper_tick_index,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    let zero_fee = FeeGrowth::new(0);
    let expected_x_increase = 21549;
    let expected_y_increase = 0;

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check pool
    assert_eq!(pool_state.liquidity, Liquidity::new(0));
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(invariant_x, expected_x_increase);
    assert_eq!(invariant_y, expected_y_increase);
}

#[test]
fn test_create_position_not_enough_token_x() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (1, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -8;
    let upper_tick_index = 8;
    let liquidity_delta = Liquidity::from_integer(10_000);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        1,
        token_y,
        initial_balance,
        None::<&str>,
    )
    .unwrap();

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
        InvariantError::FailedToChangeTokenBalance,
    );

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    get_tick(&invariant, pool_key, lower_tick_index).unwrap_err();
    get_tick(&invariant, pool_key, upper_tick_index).unwrap_err();
    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();

    assert_eq!(
        vec![
            (ActorId::from(TOKEN_X_ID), TokenAmount(1)),
            (ActorId::from(TOKEN_Y_ID), TokenAmount(initial_balance)),
        ],
        get_user_balances(&invariant, REGULAR_USER_1.into())
    );

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_1,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        ),
        Some((TokenAmount(1), TokenAmount(initial_balance),))
    );

    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    assert_eq!(invariant_x, 0);
    assert_eq!(invariant_y, 0);
    assert_eq!(user_1_x, 1);
    assert_eq!(user_1_y, initial_balance);
    assert_eq!(&pool_state, &pool_state_before);
}

#[test]
fn test_create_position_not_enough_token_y() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (initial_balance, 1));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_balance,
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -8;
    let upper_tick_index = 8;
    let liquidity_delta = Liquidity::from_integer(10_000);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_balance,
        token_y,
        1,
        None::<&str>,
    )
    .unwrap();

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
        InvariantError::FailedToChangeTokenBalance,
    );

    assert_eq!(
        vec![
            (ActorId::from(TOKEN_X_ID), TokenAmount(initial_balance)),
            (ActorId::from(TOKEN_Y_ID), TokenAmount(1))
        ],
        get_user_balances(&invariant, REGULAR_USER_1.into())
    );

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_1,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        ),
        Some((TokenAmount(initial_balance), TokenAmount(1)))
    );

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    get_tick(&invariant, pool_key, lower_tick_index).unwrap_err();
    get_tick(&invariant, pool_key, upper_tick_index).unwrap_err();
    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();

    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    assert_eq!(invariant_x, 0);
    assert_eq!(invariant_y, 0);
    assert_eq!(user_1_x, initial_balance);
    assert_eq!(user_1_y, 1);
    assert_eq!(&pool_state, &pool_state_before);
}

#[test]
fn test_remove_position() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let initial_mint = 10u128.pow(10);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let remove_position_index = 0;

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePool {
                token_0: token_x,
                token_1: token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            },
        )
        .assert_success();

    let lower_tick_index = -20;
    let upper_tick_index = 10;
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    mint(&token_x_program, REGULAR_USER_1, initial_mint).assert_success();
    mint(&token_y_program, REGULAR_USER_1, initial_mint).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_mint)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_mint)
        .assert_success();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_mint,
        token_y,
        initial_mint,
        None::<&str>,
    )
    .unwrap();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_state.liquidity, liquidity_delta);

    let liquidity_delta = Liquidity::new(liquidity_delta.get() * 1_000_000);
    let incorrect_lower_tick_index = lower_tick_index - 50;
    let incorrect_upper_tick_index = upper_tick_index + 50;

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        liquidity_delta.get(),
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        liquidity_delta.get(),
    )
    .assert_success();

    let deposit_x = balance_of(&token_x_program, REGULAR_USER_1);
    let deposit_y = balance_of(&token_y_program, REGULAR_USER_1);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        deposit_x,
        token_y,
        deposit_y,
        None::<&str>,
    )
    .unwrap();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: incorrect_lower_tick_index,
                upper_tick: incorrect_upper_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 1).unwrap();

    // Check position
    assert!(position_state.lower_tick_index == incorrect_lower_tick_index);
    assert!(position_state.upper_tick_index == incorrect_upper_tick_index);

    let amount = 1000;
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();
    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
    deposit_single_token(&invariant, REGULAR_USER_2, token_x, amount, None::<&str>).unwrap();

    invariant
        .send(
            REGULAR_USER_2,
            InvariantAction::Swap {
                pool_key,
                x_to_y: true,
                amount: swap_amount,
                by_amount_in: true,
                sqrt_price_limit: slippage,
            },
        )
        .assert_success();

    withdraw_single_token(&invariant, REGULAR_USER_2, token_y, None, None::<&str>).unwrap();

    let pool_state_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(
        pool_state_after.fee_growth_global_x,
        FeeGrowth::new(49999950000049999)
    );
    assert_eq!(pool_state_after.fee_protocol_token_x, TokenAmount(1));
    assert_eq!(pool_state_after.fee_protocol_token_y, TokenAmount(0));

    assert!(pool_state_after
        .sqrt_price
        .lt(&pool_state_before.sqrt_price));

    assert_eq!(pool_state_after.liquidity, pool_state_before.liquidity);
    assert_eq!(pool_state_after.current_tick_index, -10);
    assert_ne!(pool_state_after.sqrt_price, pool_state_before.sqrt_price);

    let amount_x = balance_of(&token_x_program, REGULAR_USER_2);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_2);
    assert_eq!(amount_x, 0);
    assert_eq!(amount_y, 993);

    // pre load dex balances
    let invariant_x_before_remove = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y_before_remove = balance_of(&token_y_program, INVARIANT_ID);

    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);

    // Remove position
    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::RemovePosition {
                position_id: remove_position_index,
            },
        )
        .assert_success();

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_1,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        ),
        Some((TokenAmount(499), TokenAmount(999)))
    );

    // Load states
    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index);
    let upper_tick = get_tick(&invariant, pool_key, lower_tick_index);
    let lower_tick_bit = is_tick_initialized(&invariant, pool_key, lower_tick_index);
    let upper_tick_bit = is_tick_initialized(&invariant, pool_key, upper_tick_index);
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    let expected_withdrawn_x = 499;
    let expected_withdrawn_y = 999;
    let expected_fee_x = 0;

    assert_eq!(
        invariant_x_before_remove - invariant_x,
        expected_withdrawn_x + expected_fee_x
    );
    assert_eq!(
        invariant_y_before_remove - invariant_y,
        expected_withdrawn_y
    );

    // Check ticks
    assert_eq!(lower_tick, Err(InvariantError::TickNotFound));
    assert_eq!(upper_tick, Err(InvariantError::TickNotFound));

    // Check tickmap
    assert!(!lower_tick_bit);
    assert!(!upper_tick_bit);

    // Check pool
    assert!(pool_state.liquidity == liquidity_delta);
    assert!(pool_state.current_tick_index == -10);
}
