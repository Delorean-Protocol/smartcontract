use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Config, RoundInfo, WinnerItem, Metadata};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
   pub config : Config
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ConfigUpdate {
        config : Config
    },
    WinnerUpdate {
        winners : Vec<WinnerItem>
    },
    RoundUpdate {
        round_info : RoundInfo
    },
    Mint {
        nft_type : u32
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FundDepositMsg {
    Deposit {
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecureMintMsg {
    SecureMint {
        owner: String,
        token_uri: Option<String>,
        extension: Metadata,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {
    },
    Winners {
    },
    RoundInfo {
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnersResponse {
    pub winners: Vec<WinnerItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoundInfoResponse {
    pub round: RoundInfo,
}


