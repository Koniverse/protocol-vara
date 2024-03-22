#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
#[cfg(test)]
mod test_helpers;


use gstd::{
    msg::{self, reply},
    prelude::*,
};
use io::*;
use contracts::{errors::InvariantError, FeeTier, FeeTiers};
use math::percentage::Percentage;
use decimal::*;
#[derive(Default, Clone)]
pub struct Invariant {
    pub config: InvariantConfig,
    pub fee_tiers: FeeTiers,
}

impl Invariant {
    pub fn change_protocol_fee(&mut self, protocol_fee: u128) -> Result<u128, InvariantError> {
        if !self.caller_is_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.config.protocol_fee = protocol_fee;
        Ok(self.config.protocol_fee)
    }

    pub fn get_protocol_fee(&self) -> u128 {
        self.config.protocol_fee
    }

    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<FeeTier, InvariantError> {
        if fee_tier.tick_spacing == 0 || fee_tier.tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee_tier.fee >= Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        if !self.caller_is_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.add(fee_tier)?;
        Ok(fee_tier)
    }

    pub fn fee_tier_exists(&self, fee_tier: FeeTier) -> bool {
        self.fee_tiers.contains(fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<FeeTier, InvariantError> {
        if !self.caller_is_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.remove(fee_tier)?;
        Ok(fee_tier)
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        self.fee_tiers.get_all()
    }

    fn caller_is_admin(&self) -> bool {
        msg::source() == self.config.admin
    }
}

static mut INVARIANT: Option<Invariant> = None;

#[no_mangle]
extern "C" fn init() {
    let init: InitInvariant = msg::load().expect("Unable to decode InitInvariant");

    let invariant = Invariant {
        config: init.config,
        fee_tiers: Default::default()
    };

    unsafe {
        INVARIANT = Some(invariant);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: InvariantAction = msg::load().expect("Unable to decode InvariantAction");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };

    match action {
        InvariantAction::ChangeProtocolFee(protocol_fee) => {
            match invariant.change_protocol_fee(protocol_fee) {
                Ok(protocol_fee) => {
                    reply(InvariantEvent::ProtocolFeeChanged(protocol_fee), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
        InvariantAction::AddFeeTier(fee_tier) => {
            match invariant.add_fee_tier(fee_tier) {
                Ok(fee_tier) => {
                    reply(InvariantEvent::FeeTierAdded(fee_tier), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
        InvariantAction::RemoveFeeTier(fee_tier) => {
            match invariant.remove_fee_tier(fee_tier) {
                Ok(fee_tier) => {
                    reply(InvariantEvent::FeeTierRemoved(fee_tier), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
    }
}
#[no_mangle]
extern "C" fn state() {
    let query: InvariantStateQuery = msg::load().expect("Unable to decode InvariantStateQuery");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };
    match query {
        InvariantStateQuery::FeeTierExist(fee_tier) => {
            let exists = invariant.fee_tier_exists(fee_tier);
            reply(InvariantState::FeeTierExist(exists), 0).expect("Unable to reply");
        }
        InvariantStateQuery::GetFeeTiers => {
            let fee_tiers = invariant.get_fee_tiers();
            reply(InvariantState::QueriedFeeTiers(fee_tiers), 0).expect("Unable to reply");
        }
        InvariantStateQuery::GetProtocolFee => {
            let protocol_fee = invariant.get_protocol_fee();
            reply(InvariantState::ProtocolFee(protocol_fee), 0).expect("Unable to reply");
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use gtest::{Program, System};
    const USERS: [u64; 3] = [1, 2, 3];
    const ADMIN: u64 = USERS[0];
    const USER: u64 = USERS[1];
    const PROGRAM_OWNER: u64 = USERS[2];
    const PROGRAM_ID: u64 = 105;
    const PATH: &str = "./target/wasm32-unknown-unknown/release/invariant.wasm";

    pub fn init_invariant(sys: &System, protocol_fee: u128) -> Program<'_> {
        let program = Program::from_file_with_id(sys, PROGRAM_ID, PATH);
    
        assert!(!program
            .send(
                PROGRAM_OWNER,
                InitInvariant {
                    config: InvariantConfig {
                        admin: ADMIN.into(),
                        protocol_fee,
                    },
                },
            )
            .main_failed());
        program
    }

    #[test]
    fn test_init() {
        let sys = System::new();
        sys.init_logger();

        let _invariant = init_invariant(&sys, 100);
    }

    #[test]
    fn test_fee_tiers() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 100);
        let fee_tier = FeeTier::new(Percentage::new(1), 10u16).unwrap();
        let fee_tier_value = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 10u16,
        };

        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.contains(&(ADMIN, InvariantEvent::FeeTierAdded(fee_tier_value).encode())));
        
        let state: InvariantState = invariant.read_state(InvariantStateQuery::GetFeeTiers).expect("Failed to read state");
        assert_eq!(state, InvariantState::QueriedFeeTiers(vec![fee_tier_value]));

        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.contains(&(ADMIN, InvariantEvent::ActionFailed(InvariantError::FeeTierAlreadyExist).encode())));

        let state: InvariantState = invariant.read_state(InvariantStateQuery::GetFeeTiers).expect("Failed to read state");
        assert_eq!(state, InvariantState::QueriedFeeTiers(vec![fee_tier_value]));

        let res = invariant.send(ADMIN, InvariantAction::RemoveFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.contains(&(ADMIN, InvariantEvent::FeeTierRemoved(fee_tier_value).encode())));

        let state: InvariantState = invariant.read_state(InvariantStateQuery::GetFeeTiers).expect("Failed to read state");
        assert_eq!(state, InvariantState::QueriedFeeTiers(vec![]));
    }
}
