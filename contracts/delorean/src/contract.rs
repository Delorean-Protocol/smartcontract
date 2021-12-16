use crate::errors::{ContractError, Unauthorized};
use crate::msg::{
    ConfigResponse, Cw721ExecuteMsg, ExecuteMsg, FundDepositMsg, InstantiateMsg, MigrateMsg,
    QueryMsg, RoundInfoResponse, SecureMintMsg, WinnersResponse,
};
use crate::state::{Config, Metadata, RoundInfo, WinnerInfo, CONFIG, ROUND_INFO, WINNER_INFO};
use cosmwasm_std::{
    coin, entry_point, to_binary, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse,
    Response, StdResult, SubMsg, WasmMsg,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
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

        ExecuteMsg::WinnerUpdate { winner } => try_winners_update(deps, env, info, winner),

        ExecuteMsg::ClaimPrize { burn_nft_id } => try_claim_prize(deps, env, info, burn_nft_id),

        ExecuteMsg::RoundUpdate { round_info } => try_round_update(deps, env, info, round_info),

        ExecuteMsg::Mint { nft_type } => try_mint(deps, env, info, nft_type),
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
    new_config: Config,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

pub fn try_claim_prize(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    burn_nft_id: String,
) -> Result<Response, ContractError> {
    let _winner = WINNER_INFO.may_load(deps.storage)?;
    match _winner {
        None => {
            return Err(ContractError::NotFound {});
        }
        Some(mut _winner) => {
            let time = _env.block.time.nanos() / 1_000_000_000;

            if _winner.winner_address != info.sender.to_string()
                || _winner.claimed == true
                || _winner.claim_end_time < time
            {
                return Err(Unauthorized {}.build());
            }

            let _config = CONFIG.load(deps.storage)?;

            let msg = Cw721ExecuteMsg::Burn {
                token_id: burn_nft_id,
            };
            let burn_nft_submsg = SubMsg::new(WasmMsg::Execute {
                contract_addr: _config.nft_contract,
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            _winner.claimed = true;
            WINNER_INFO.save(deps.storage, &_winner)?;

            //Burn nft and send winner prize
            Ok(Response::default()
                .add_message(BankMsg::Send {
                    to_address: _winner.winner_address,
                    amount: vec![_winner.winner_amount].to_vec(),
                })
                .add_submessage(burn_nft_submsg))
        }
    }
}

pub fn try_winners_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    winner: WinnerInfo,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    WINNER_INFO.save(deps.storage, &winner)?;
    Ok(Response::default())
}

pub fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut nft_type: u32,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
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
    let mut msgs: Vec<SubMsg> = vec![];

    msgs.push(secure_mint_nft(
        _config.mint_contract.clone(),
        info.sender.clone().to_string(),
        nft_info.nft_metadata.clone(),
        "".to_string(),
    )?);

    for fund_share in nft_info.shares {
        let amount = fund_share.get_share(sent_funds[0].clone().amount).u128();
        msgs.push(deposit_funds(
            fund_share.address.clone().to_string(),
            coin(amount, sent_funds[0].clone().denom),
        )?);
    }

    Ok(Response::default().add_submessages(msgs))
}

pub fn try_round_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    round: RoundInfo,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
    if info.sender != _config.admin {
        return Err(Unauthorized {}.build());
    }
    ROUND_INFO.save(deps.storage, &round)?;

    Ok(Response::default())
}

fn deposit_funds(contract_addr: String, coin: Coin) -> Result<SubMsg, ContractError> {
    let msg = FundDepositMsg::Deposit {};
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_addr,
        msg: to_binary(&msg)?,
        funds: vec![coin],
    });
    Ok(exec)
}

fn secure_mint_nft(
    contract_address: String,
    to: String,
    extension: Metadata,
    token_uri: String,
) -> Result<SubMsg, ContractError> {
    let msg = SecureMintMsg::SecureMint {
        owner: to,
        extension: extension,
        token_uri: token_uri,
    };
    let exec = SubMsg::new(WasmMsg::Execute {
        contract_addr: contract_address,
        msg: to_binary(&msg)?,
        funds: vec![],
    });
    Ok(exec)
}

fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = CONFIG.load(deps.storage)?;
    let rsp = ConfigResponse { config: state };
    to_binary(&rsp)
}

fn get_round_info(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let round_info = ROUND_INFO.may_load(deps.storage)?;
    let rsp = RoundInfoResponse { round: round_info };
    to_binary(&rsp)
}

fn get_winners(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let winner_info = WINNER_INFO.may_load(deps.storage)?;
    let rsp = WinnersResponse {
        winner: winner_info,
    };
    to_binary(&rsp)
}
