use std::ops::{Add, Mul};

use cosmwasm_std::{
    coin, from_binary, Addr, Attribute, Binary, ContractResult, OwnedDeps, Response, Uint128,
};
use cosmwasm_vm::testing::{
    execute, instantiate, migrate, mock_backend, mock_env, mock_info, mock_instance_options, query,
};
use cosmwasm_vm::Instance;
use delorean_distributer::errors::ContractError;
use delorean_distributer::msg::{
    ClaimStatusResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use delorean_distributer::state::{Config, FundShare, CLAIM_STATE, CONFIG, FUND_STATE};

static WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_distributer.wasm");

#[test]
fn delorean_distributer_test() {
    let backend = mock_backend(&[]);
    let admin = String::from("admin");
    let user1 = String::from("user1");
    let user2 = String::from("user2");

    let admin_info = mock_info(&admin, &&[coin(150000u128, "uusd")].to_vec());
    let user1_info = mock_info(&String::from("user1"), &[coin(10000u128, "uusd")].to_vec());

    let shares = [
        FundShare {
            address: user1.clone(),
            note: "".to_string(),
            share: 2000u32, //20.00
        },
        FundShare {
            address: user2.clone(),
            note: "".to_string(),
            share: 4000u32, //40.00
        },
    ]
    .to_vec();

    let config = Config {
        admin: admin.clone(),
        shares: shares.clone(),
    };

    let instatiate_msg = InstantiateMsg {
        config: config.clone(),
    };

    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();
    // make sure we can instantiate with this
    let res: ContractResult<Response> =
        instantiate(&mut deps, mock_env(), admin_info.clone(), instatiate_msg);
    let rsp = query(&mut deps, mock_env(), QueryMsg::Config {}).unwrap();
    let config_rsp: ConfigResponse = from_binary(&rsp).unwrap();

    assert_eq!(config_rsp.config, config);

    let update_config_msg = ExecuteMsg::ConfigUpdate {
        config: config.clone(),
    };
    let rsp: ContractResult<Response> =
        execute(&mut deps, mock_env(), user1_info.clone(), update_config_msg);
    assert_eq!(rsp.is_err(), true);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Deposit {},
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = migrate(&mut deps, mock_env(), MigrateMsg {});
    assert_eq!(rsp.is_err(), false);

    let rsp: Binary = query(
        &mut deps,
        mock_env(),
        QueryMsg::ClaimStatus {
            wallet: user1.clone(),
        },
    )
    .unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected = ClaimStatusResponse {
        claimable_ust: Uint128::from(2000u32),
        claimed_ust: None,
        total_ust: Uint128::from(10000u32),
        share: 2000u32,
    };
    assert_eq!(
        rsp, expected,
        "Claimable UST of user 1 should match rsp={:?}",
        expected
    );

    let rsp: Binary = query(
        &mut deps,
        mock_env(),
        QueryMsg::ClaimStatus {
            wallet: user2.clone(),
        },
    )
    .unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected = ClaimStatusResponse {
        claimable_ust: Uint128::from(4000u32),
        claimed_ust: None,
        total_ust: Uint128::from(10000u32),
        share: 4000u32,
    };
    assert_eq!(
        rsp, expected,
        "Claimable UST of user 2 should match rsp={:?}",
        expected
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Claim {},
    );
    assert_eq!(rsp.is_err(), false);

    assert_eq!(
        rsp.unwrap().attributes.clone(),
        [
            Attribute {
                key: "action".to_string(),
                value: "claim".to_string()
            },
            Attribute {
                key: "ust".to_string(),
                value: "2000".to_string()
            },
            Attribute {
                key: "wallet".to_string(),
                value: "user1".to_string()
            }
        ]
        .to_vec(),
        "User 1 should have 2000 ust in the wallet after claim"
    );

    let rsp: Binary = query(
        &mut deps,
        mock_env(),
        QueryMsg::ClaimStatus {
            wallet: user1.clone(),
        },
    )
    .unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected = ClaimStatusResponse {
        claimable_ust: Uint128::from(0u32),
        claimed_ust: Some(Uint128::from(10000u32)),
        total_ust: Uint128::from(10000u32),
        share: 2000u32,
    };
    assert_eq!(
        rsp, expected,
        "Claimable UST of user 1 after first claim should match of rsp={:?}",
        expected
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::Deposit {},
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: Binary = query(
        &mut deps,
        mock_env(),
        QueryMsg::ClaimStatus {
            wallet: user1.clone(),
        },
    )
    .unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected = ClaimStatusResponse {
        claimable_ust: Uint128::from(30000u32),
        claimed_ust: Some(Uint128::from(10000u32)),
        total_ust: Uint128::from(160000u32),
        share: 2000u32,
    };
    assert_eq!(
        rsp, expected,
        "Claimable UST of user 1 after 2nd deposit should match rsp={:?}",
        expected
    );

    let rsp: Binary = query(
        &mut deps,
        mock_env(),
        QueryMsg::ClaimStatus {
            wallet: user2.clone(),
        },
    )
    .unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected = ClaimStatusResponse {
        claimable_ust: Uint128::from(64000u32),
        claimed_ust: None,
        total_ust: Uint128::from(160000u32),
        share: 4000u32,
    };
    assert_eq!(
        rsp, expected,
        "Claimable UST of user 2 after 2nd deposit should match rsp={:?}",
        expected
    );
}
