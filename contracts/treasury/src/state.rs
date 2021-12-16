use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{ Item };

const  CONFIG_KEY: &str = "config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
    pub anchor_smart_contract : String
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);