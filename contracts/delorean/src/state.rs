use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Uint128};
use cw_storage_plus::Item;

const CONFIG_KEY: &str = "config_1sd&23";
const WINNERS_KEY: &str = "winners_1gasd2";
const ROUNDINFO_KEY: &str = "round_info_12fas";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
    pub mint_contract: String,
    pub nft_contract: String,
    pub nfts: Vec<NftMetaInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftMetaInfo {
    pub nft_metadata: Metadata,
    pub price: Coin,
    pub shares: Vec<FundShare>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoundInfo {
    pub active: bool,
    pub start_date: u64,
    pub end_date: u64,
    pub name: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnerInfo {
    pub winner_address: String,
    pub winner_amount: Coin,
    pub claim_end_time: u64,
    pub claimed: bool,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const ROUND_INFO: Item<RoundInfo> = Item::new(ROUNDINFO_KEY);
pub const WINNER_INFO: Item<WinnerInfo> = Item::new(WINNERS_KEY);
