use std::vec;

use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    AnchorExecuteMsg, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use moneymarket::market::Cw20HookMsg;
use moneymarket::querier::deduct_tax;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ConfigUpdate { config } => try_config_update(deps, env, info, config),

        ExecuteMsg::Deposit {} => try_deposit(deps, env, info),
        ExecuteMsg::AnchorWithdraw { amount } => try_anchor_withdraw(deps, env, info, amount),
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
    new_config: Config,
) -> Result<Response, ContractError> {
    let mut _config = CONFIG.load(deps.storage)?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::default())
}

pub fn try_anchor_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(Unauthorized {}.build());
    }
    // let balance = query_token_balance(deps.as_ref(), Addr::unchecked(config.clone().aust_contract), _env.clone().contract.address )?;

    let msg = Cw20ExecuteMsg::Send {
        contract: config.clone().anchor_smart_contract,
        amount: amount,
        msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: config.clone().aust_contract,
        msg: to_binary(&msg)?,
        funds: vec![],
    });

    Ok(Response::default()
        .add_submessage(exec)
        .add_attribute("action", "anchor_withdraw"))
}

pub fn try_withdraw_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config_state = CONFIG.load(deps.storage)?;
    if info.sender != config_state.admin {
        return Err(Unauthorized {}.build());
    }
    let balance = deps.querier.query_balance(&_env.contract.address, "uusd")?;

    Ok(Response::new().add_message(transfer_funds(
        &info.sender,
        vec![deduct_tax(deps.as_ref(), balance)?],
    )))
}

fn acnchor_deposit(contract_addr: String, coins: Vec<Coin>) -> Result<SubMsg, ContractError> {
    let msg = AnchorExecuteMsg::DepositStable {};
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_addr,
        msg: to_binary(&msg)?,
        funds: coins,
    });
    Ok(exec)
}

pub fn try_deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sent_funds = info.funds.clone();
    let config = CONFIG.load(deps.storage)?;

    Ok(Response::new()
        .add_submessage(acnchor_deposit(
            config.anchor_smart_contract.clone().to_string(),
            vec![deduct_tax(
                deps.as_ref(),
                deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: "uusd".to_string(),
                        amount: sent_funds[0].amount,
                    },
                )?,
            )?],
        )?)
        .add_attribute("action", "treasury_deposit"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => get_config(deps, env),
    }
}

fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = CONFIG.load(deps.storage)?;
    let rsp = ConfigResponse { config: state };
    to_binary(&rsp)
}
