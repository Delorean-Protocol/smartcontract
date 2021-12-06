use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Storage, Uint128};
use cosmwasm_storage::{
    singleton, singleton_read, ReadonlySingleton,
    Singleton,
};
pub static CONFIG_KEY: &[u8] = b"config";
pub static MINT_STATUS_KEY: &[u8] = b"mint_status";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub minter: Addr,
    pub nft_contract: Addr,
    pub nft_metadata: Metadata,
    pub shares: Vec<FundShare>,
    pub price : Coin,
    pub mint_limit: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintStatus {
   pub mint_count: u32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundShare {
    pub address: Addr,
    pub note: String,
    pub share: u32 // in decimal of 100
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



pub fn config_update(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn status_update(storage: &mut dyn Storage) -> Singleton<MintStatus> {
    singleton(storage, MINT_STATUS_KEY)
}

pub fn status_read(storage: &dyn Storage) -> ReadonlySingleton<MintStatus> {
    singleton_read(storage, MINT_STATUS_KEY)
}
