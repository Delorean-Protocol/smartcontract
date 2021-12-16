


use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, AnchorExecuteMsg, MigrateMsg
};
use crate::state::{Config, config_read, config_update};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse,
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
         
        ExecuteMsg::Deposit {} => try_deposit(deps,env,info),
        ExecuteMsg::WithdrawFund {} => try_withdraw_fund(deps, env, info),
   
    }
}


fn transfer_funds(to: &Addr, cns: Vec<Coin>) -> BankMsg {
    return BankMsg::Send {
        to_address: to.to_string(),
        amount: cns,
    };
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

pub fn try_withdraw_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config_state = config_read(deps.storage).load()?;
    if info.sender != config_state.admin {
        return Err(Unauthorized {}.build());
    }
    let balance = deps.querier.query_all_balances(&_env.contract.address)?;

    Ok(Response::new()
    .add_message(transfer_funds(&info.sender, balance)))
}

fn acnchor_deposit(contract_addr : String, coins: Vec<Coin>) -> Result<SubMsg, ContractError> {
    let msg = AnchorExecuteMsg::DepositStable {
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_addr,
        msg: to_binary(&msg)?,
        funds: coins,
    });
    Ok(exec)
}


pub fn try_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sent_funds = info.funds.clone();
    let config = config_read(deps.storage).load()?; 
    
    Ok(Response::new().add_submessage(acnchor_deposit(config.anchor_smart_contract.clone().to_string(), sent_funds.clone())?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg ) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => get_config(deps, env),
    }
}

fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = config_read(deps.storage).load()?;
    let rsp = ConfigResponse { config: state };
    to_binary(&rsp)
}

