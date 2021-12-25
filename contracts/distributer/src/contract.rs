use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    ClaimStatusResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::state::{Config, CLAIM_STATE, CONFIG, FUND_STATE};
use cosmwasm_std::{
    coin, entry_point, to_binary, Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo,
    QueryResponse, Response, StdResult, Uint128,
};
use moneymarket::querier::deduct_tax;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
    let i: Uint128 = Uint128::from(0u32);
    FUND_STATE.save(deps.storage, &i)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
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
        ExecuteMsg::Claim {} => try_claim(deps, env, info),
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

pub fn try_claim(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let wallet = info.sender.clone();

    let config = CONFIG.load(deps.storage)?;
    let mut found = None;
    for fund_share in config.shares {
        if fund_share.address == wallet {
            found = Some(fund_share);
            break;
        }
    }

    if found == None {
        return Err(ContractError::NotFound {});
    } else {
        let funds = FUND_STATE.load(deps.storage)?;
        let claimable_ust;
        let claimed_ust = CLAIM_STATE.may_load(deps.storage, &wallet.to_string())?;
        let mut claimable: Uint128 = funds.clone();
        match claimed_ust {
            None => {}
            Some(claimed_ust) => claimable = claimable - claimed_ust,
        }
        let share = found.unwrap();
        claimable_ust = share.get_share(claimable);
        CLAIM_STATE.save(deps.storage, &wallet.to_string(), &claimable)?;

        Ok(Response::new()
            .add_attribute("action", "claim")
            .add_attribute("ust", claimable_ust.to_string())
            .add_attribute("wallet", wallet.clone().to_string())
            .add_message(transfer_funds(
                &wallet,
                vec![deduct_tax(
                    deps.as_ref(),
                    coin(claimable_ust.u128(), "uusd"),
                )?],
            )))
    }
}

pub fn try_deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sent_funds = info.funds.clone();

    if sent_funds.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    if &sent_funds[0].denom != "uusd" {
        return Err(ContractError::EmptyBalance {});
    }
    let mut amnt = FUND_STATE.load(deps.storage)?;
    amnt = amnt + &sent_funds[0].amount;
    FUND_STATE.save(deps.storage, &amnt)?;

    Ok(Response::default())
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => get_config(deps, env),
        QueryMsg::ClaimStatus { wallet } => get_claim_status(deps, env, wallet),
    }
}

fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let config = CONFIG.load(deps.storage)?;
    let rsp = ConfigResponse { config: config };
    to_binary(&rsp)
}

fn get_claim_status(deps: Deps, _env: Env, wallet: String) -> StdResult<QueryResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut found = None;
    for fund_share in config.shares {
        if fund_share.address == wallet {
            found = Some(fund_share);
            break;
        }
    }
    let rsp: ClaimStatusResponse;
    if found == None {
        rsp = ClaimStatusResponse {
            claimable_ust: Uint128::from(0u32),
            claimed_ust: None,
            total_ust: Uint128::from(0u32),
            share: 0u32,
        };
    } else {
        let funds = FUND_STATE.load(deps.storage)?;
        let claimed_ust = CLAIM_STATE.may_load(deps.storage, &wallet)?;
        let mut claimable: Uint128 = funds.clone();
        match claimed_ust {
            None => {}
            Some(claimed_ust) => claimable = claimable - claimed_ust,
        }
        let share = found.unwrap();
        rsp = ClaimStatusResponse {
            claimable_ust: share.get_share(claimable),
            claimed_ust: claimed_ust,
            total_ust: funds.clone(),
            share: share.share,
        };
    }

    to_binary(&rsp)
}
