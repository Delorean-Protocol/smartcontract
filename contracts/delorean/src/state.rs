use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, Coin, Uint128};
use cosmwasm_storage::{
    singleton, singleton_read, ReadonlySingleton,
    Singleton,
};
const CONFIG_KEY: &str = "config_1sd&23";
const WINNERS_KEY: &str = "winners_1gasd2";
const ROUNDINFO_KEY: &str = "round_info_12fas";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub mint_contract: Addr,
    pub nfts : Vec<NftMetaInfo>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftMetaInfo {
   pub nft_metadata: Metadata,
   pub price : Coin,
   pub shares: Vec<FundShare>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoundInfo {
   pub active: bool,
   pub start_date : u64,
   pub end_date : u64,
   pub name : String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundShare {
    pub address: Addr,
    pub note: String,
    pub share: u32 // in decimal of 100
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
pub struct WinnerItem {
    pub winner : String,
    pub winner_amount : Coin,
    pub claim_end_time : u64,
    pub claimed : bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnerInfo {
    pub winners : Vec<WinnerItem>
}



pub fn config_update(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn round_read(storage: &dyn Storage) -> ReadonlySingleton<RoundInfo> {
    singleton_read(storage, ROUNDINFO_KEY)
}

pub fn round_update(storage: &mut dyn Storage) -> Singleton<RoundInfo> {
    singleton(storage, ROUNDINFO_KEY)
}

pub fn winner_read(storage: &dyn Storage) -> ReadonlySingleton<WinnerInfo> {
    singleton_read(storage, WINNERS_KEY)
}

pub fn winner_update(storage: &mut dyn Storage) -> Singleton<WinnerInfo> {
    singleton(storage, WINNERS_KEY)
}
