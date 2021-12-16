use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Uint128};
use cw_storage_plus::Item;
const CONFIG_KEY: &str = "config";
const MINT_STATUS_KEY: &str = "mint_status";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
    pub minter: String,
    pub nft_contract: String,
    pub nft_metadata: Metadata,
    pub shares: Vec<FundShare>,
    pub price: Coin,
    pub mint_limit: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintStatus {
    pub mint_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundShare {
    pub address: String,
    pub note: String,
    pub share: u32, // in decimal of 100
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

impl FundShare {
    pub fn get_share(&self, fund: Uint128) -> Uint128 {
        let d = fund.clone();
        return d.multiply_ratio(self.share, 10000u32);
    }
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const MINTSTATUS: Item<MintStatus> = Item::new(MINT_STATUS_KEY);
