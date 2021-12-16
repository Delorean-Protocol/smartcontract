use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};
pub const CONFIG_KEY: &str = "config";
pub const FUNDSTATE_KEY: &str = "fund_state";
pub const CLAIMED_STATE_KEY: &str = "claim_state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
    pub shares: Vec<FundShare>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundShare {
    pub address: String,
    pub note: String,
    pub share: u32, // in decimal of 100
}

impl FundShare {
    pub fn get_share(&self, fund: Uint128) -> Uint128 {
        let d = fund.clone();
        return d.multiply_ratio(self.share, 10000u32);
    }
}

pub const CLAIM_STATE: Map<&str, Uint128> = Map::new(CLAIMED_STATE_KEY);
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const FUND_STATE: Item<Uint128> = Item::new(FUNDSTATE_KEY);
