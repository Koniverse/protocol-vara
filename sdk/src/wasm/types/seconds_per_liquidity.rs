use crate::liquidity::Liquidity;
use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(25, U256)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SecondsPerLiquidity(#[tsify(type = "bigint")] pub u128);

decimal_ops!(SecondsPerLiquidity);

impl SecondsPerLiquidity {
    pub fn unchecked_add(self, other: SecondsPerLiquidity) -> SecondsPerLiquidity {
        if other.get() > SecondsPerLiquidity::max_instance().get() - self.get() {
            SecondsPerLiquidity::new(
                (other.get() - (SecondsPerLiquidity::max_instance().get() - self.get())) - 1,
            )
        } else {
            SecondsPerLiquidity::new(self.get() + other.get())
        }
    }

    pub fn unchecked_sub(self, other: SecondsPerLiquidity) -> SecondsPerLiquidity {
        if other.get() > self.get() {
            SecondsPerLiquidity::new(
                SecondsPerLiquidity::max_instance().get() - (other.get() - self.get()) + 1,
            )
        } else {
            SecondsPerLiquidity::new(self.get() - other.get())
        }
    }
    pub fn calculate_seconds_per_liquidity_global(
        liquidity: Liquidity,
        current_timestamp: u64,
        last_timestamp: u64,
    ) -> TrackableResult<Self> {
        if current_timestamp <= last_timestamp {
            return Err(err!("current_timestamp > last_timestamp failed"));
        }
        let delta_time = current_timestamp - last_timestamp;

        Ok(Self::new(
            Self::checked_from_value(
                Self::from_value::<U256, u128>(
                    u128::from(delta_time)
                        .checked_mul(Self::one().cast())
                        .ok_or_else(|| err!(TrackableError::MUL))?
                        .checked_mul(Liquidity::one().cast())
                        .ok_or_else(|| err!(TrackableError::MUL))?,
                )
                .checked_div(liquidity.get())
                .ok_or_else(|| err!(TrackableError::DIV))?,
            )
            .map_err(|_| err!(TrackableError::cast::<u128>().as_str()))?,
        ))
    }
}

pub fn calculate_seconds_per_liquidity_inside(
    tick_lower: i32,
    tick_upper: i32,
    tick_current: i32,
    tick_lower_seconds_per_liquidity_outside: SecondsPerLiquidity,
    tick_upper_seconds_per_liquidity_outside: SecondsPerLiquidity,
    pool_seconds_per_liquidity_global: SecondsPerLiquidity,
) -> TrackableResult<SecondsPerLiquidity> {
    let current_above_lower = tick_current >= tick_lower;
    let current_below_upper = tick_current < tick_upper;

    let seconds_per_liquidity_below = if current_above_lower {
        tick_lower_seconds_per_liquidity_outside
    } else {
        pool_seconds_per_liquidity_global.unchecked_sub(tick_lower_seconds_per_liquidity_outside)
    };

    let seconds_per_liquidity_above = if current_below_upper {
        tick_upper_seconds_per_liquidity_outside
    } else {
        pool_seconds_per_liquidity_global.unchecked_sub(tick_upper_seconds_per_liquidity_outside)
    };

    Ok(pool_seconds_per_liquidity_global
        .unchecked_sub(seconds_per_liquidity_below)
        .unchecked_sub(seconds_per_liquidity_above))
}
