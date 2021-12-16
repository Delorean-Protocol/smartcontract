use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, FundDepositMsg, WinnersResponse, RoundInfoResponse, SecureMintMsg, MigrateMsg
};
use crate::state::{Config,Metadata, config_read, config_update, round_read, round_update, winner_read, winner_update, WinnerItem, WinnerInfo, RoundInfo};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Coin, coin, Deps, DepsMut, Env, MessageInfo, QueryResponse,
    Response, StdResult, SubMsg, WasmMsg
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    config_update(deps.storage).save(&msg.config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg ) -> Result<Response, ContractError> {
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

        ExecuteMsg::WinnerUpdate {
            winners
        } => try_winners_update(deps, env, info, winners),

        ExecuteMsg::RoundUpdate {
            round_info
        } => try_round_update(deps, env, info, round_info),

        ExecuteMsg::Mint {
            nft_type
        } => try_mint(deps, env, info, nft_type),
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => get_config(deps, env),
        QueryMsg::Winners {} => get_winners(deps, env),
        QueryMsg::RoundInfo {} => get_round_info(deps, env),
    }
}

pub fn try_config_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_config : Config
) -> Result<Response, ContractError> {
    let _config = config_read(deps.storage).load()?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    config_update(deps.storage).save(&new_config)?;

    Ok(Response::default())
}


pub fn try_winners_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    winners : Vec<WinnerItem>
) -> Result<Response, ContractError> {
    let _config = config_read(deps.storage).load()?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    let winner_info = WinnerInfo{ winners : winners };
    winner_update(deps.storage).save(&winner_info)?;

    Ok(Response::default())
}


pub fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut nft_type : u32
) -> Result<Response, ContractError> {
    let _config = config_read(deps.storage).load()?;
    nft_type = nft_type - 1;
    if nft_type >= _config.nfts.len() as u32 {
        return Err(Unauthorized {}.build());
    }
    let sent_funds = info.funds.clone();
    let nft_info = _config.nfts[nft_type as usize].clone();

    if sent_funds.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    if nft_info.price.denom != sent_funds[0].denom {
        return Err(ContractError::InsufficientFund {});
    }
    if sent_funds[0].amount != nft_info.price.amount {
        return Err(ContractError::InsufficientFund {});
    }
    secure_mint_nft(&_config.mint_contract, info.sender.clone().to_string(),&nft_info.nft_metadata, None)?;
   
    for fund_share in nft_info.shares {
        let amount = fund_share.get_share(sent_funds[0].clone().amount).u128();
        deposit_funds(fund_share.address.clone().to_string(), coin(amount, sent_funds[0].clone().denom ) )?;
     };
 
    Ok(Response::default())
}


pub fn try_round_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    round : RoundInfo
) -> Result<Response, ContractError> {
    let _config = config_read(deps.storage).load()?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    round_update(deps.storage).save(&round)?;

    Ok(Response::default())
}


fn deposit_funds(contract_addr : String, coin: Coin) -> Result<SubMsg, ContractError> {
    let msg = FundDepositMsg::Deposit {
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_addr,
        msg: to_binary(&msg)?,
        funds: vec![coin],
    });
    Ok(exec)
}

fn secure_mint_nft(contract_address: &Addr, to: String, extension: &Metadata, token_uri : Option<String>) -> Result<SubMsg, ContractError> {
    let msg = SecureMintMsg::SecureMint {
        owner: to,
        extension : extension.clone(),
        token_uri : token_uri
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    });
    Ok(exec)
}


fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = config_read(deps.storage).load()?;
    let rsp = ConfigResponse { config: state };
    to_binary(&rsp)
}

fn get_round_info(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let round_info = round_read(deps.storage).load()?;
    let rsp = RoundInfoResponse { round: round_info };
    to_binary(&rsp)
}

fn get_winners(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let winners_info = winner_read(deps.storage).load()?;
    let rsp = WinnersResponse { winners: winners_info.winners };
    to_binary(&rsp)
}



