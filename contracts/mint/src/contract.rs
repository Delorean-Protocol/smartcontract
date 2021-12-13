

use std::ops::Div;

use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    StatusResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, NftExecuteMsg, FundDepositExecuteMsg
};
use crate::state::{Config, Metadata, MintStatus, config_read, config_update, status_read, status_update};
use cosmwasm_std::{
    to_binary, Addr, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse,
    Response, StdResult, SubMsg, WasmMsg, coins
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let status = MintStatus{
        mint_count:0
    };

    config_update(deps.storage).save(&msg.config)?;
    status_update(deps.storage).save(&status)?;
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
     
        ExecuteMsg::ConfigUpdate {
           config
        } => try_config_update(deps, env, info, config),

        ExecuteMsg::Mint {
         } => try_mint(deps, env, info),

        ExecuteMsg::SecureMint {
            owner,
            token_uri,
            extension
        } => try_secure_mint(deps, env, info, owner, token_uri, extension),
    
    }
}


fn deposit_funds(contract: &Addr, cns: Vec<Coin>) -> Result<SubMsg, ContractError> {
    let msg = FundDepositExecuteMsg::Deposit {
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        msg: to_binary(&msg)?,
        funds: cns,
    });
    Ok(exec)
}

fn mint_nft(to: String,token_id: String, nft_contract: &Addr, extension: &Metadata, token_uri : Option<String>) -> Result<SubMsg, ContractError> {
    let msg = NftExecuteMsg::Mint {
        owner: to,
        token_id: token_id,
        extension : extension.clone(),
        token_uri : token_uri
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: nft_contract.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    });
    Ok(exec)
}


pub fn try_config_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_config : Config
) -> Result<Response, ContractError> {
    let mut _config = config_read(deps.storage).load()?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    config_update(deps.storage).save(&new_config)?;

    Ok(Response::default())
}

pub fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;
    let sent_funds = info.funds.clone();
    let mint_count:u32;
    if sent_funds.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    if config.price.denom != sent_funds[0].denom {
        return Err(ContractError::InsufficientFund {});
    }
    mint_count = sent_funds[0].amount.div(config.price.amount).u128() as u32;
    if mint_count != 1{
        return Err(ContractError::InsufficientFund {});
    }
    let mut mintstatus = status_read(deps.storage).load()?;

      // limit check
    if mintstatus.mint_count + mint_count > config.mint_limit {
        return Err(ContractError::MintLimitReached {});
    }
    let token_id = mintstatus.mint_count + mint_count;
    mintstatus.mint_count = token_id;

    mint_nft(info.sender.clone().to_string(), token_id.clone().to_string(), &config.nft_contract, &config.nft_metadata, None)?;


    for fund_share in config.shares {
       let amount = fund_share.get_share(sent_funds[0].clone().amount).u128();
       deposit_funds(&fund_share.address, coins(amount, sent_funds[0].clone().denom ) )?;
    };


    status_update(deps.storage).save(&mintstatus)?;
    Ok(Response::default())
}

pub fn try_secure_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: String,
    token_uri : Option<String>,
    extension : Metadata
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;
    if !(info.sender == config.admin || info.sender == config.minter) {
        return Err(Unauthorized {}.build());
    }

    let mut mintstatus = status_read(deps.storage).load()?;    
    let token_id = mintstatus.mint_count + 1;
    mintstatus.mint_count = token_id;

    mint_nft(owner.clone(), token_id.clone().to_string(), &config.nft_contract, &extension, token_uri)?;

    status_update(deps.storage).save(&mintstatus)?;
    
    Ok(Response::default())
}


pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Status  {} => get_status(deps, env),
        QueryMsg::Config {} => get_config(deps, env),
    }
}

fn get_status(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let st = status_read(deps.storage).load()?;
    let rsp = StatusResponse { mint_status: st };
    to_binary(&rsp)
}

fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = config_read(deps.storage).load()?;
    let rsp = ConfigResponse { config: state };
    to_binary(&rsp)
}

