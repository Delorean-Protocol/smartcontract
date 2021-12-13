
use std::ops::{Add, Mul};

use cosmwasm_std::{Addr, Binary, ContractResult, Decimal, OwnedDeps, Response, Uint128, coin, from_binary};
use cosmwasm_vm::testing::{
    instantiate, mock_backend, mock_env, mock_info, mock_instance_options, query, execute
};
use cosmwasm_vm::Instance;
use delorean_distributer::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, ClaimStatusResponse, QueryMsg};
use delorean_distributer::state::{
    Config, FUND_STATE, CONFIG, CLAIM_STATE, FundShare
};
use delorean_distributer::errors::{ ContractError };
use serde;

static WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_distributer.wasm");
 
#[test]
fn delorean_distributer_test() {
    let mut backend = mock_backend(&[]);
    let admin = String::from("admin");
    let user1 = String::from("user1");
    let user2 = String::from("user2");

   

    let admin_info = mock_info(&admin, &[]);
    let mut user1_info = mock_info(&String::from("user1"), &[]);

    let shares = [ FundShare{
                address: user1.clone(),
                note: "".to_string(),
                share: 2000u32,  //20.00
            },
            FundShare{
                address: user2.clone(),
                note: "".to_string(),
                share: 4000u32, //40.00
            }
    ].to_vec();

    let config =  Config{
        admin : admin.clone(), 
        shares : shares.clone()
    };
    
    let instatiate_msg = InstantiateMsg {
        config : config.clone()
    };

    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();
    // make sure we can instantiate with this
    let res: ContractResult<Response> = instantiate(&mut deps, mock_env(), admin_info, instatiate_msg);
    let rsp = query(&mut deps, mock_env(), QueryMsg::Config{}).unwrap();
    let config_rsp: ConfigResponse = from_binary(&rsp).unwrap();
    
    assert_eq!(
        config_rsp.config,
        config
    );

    let update_config_msg = ExecuteMsg::ConfigUpdate {
        config : config.clone()
    };
    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), update_config_msg);
    assert_eq!(rsp.is_err(), true);

    user1_info.funds = [coin(10000u128, "uusd")].to_vec();
    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Deposit {});
    assert_eq!(rsp.is_err(), false);

    let rsp:Binary = query(&mut deps, mock_env(),QueryMsg::ClaimStatus{ wallet : user1.clone() }).unwrap();
    let rsp:ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected =  ClaimStatusResponse { 
        claimable_ust: Uint128::from(2000u32),
        claimed_ust: None,
        total_ust : Uint128::from(10000u32),
        share : 2000u32 
    };
    assert_eq!(rsp, expected, "Claimable UST of user 1 should match rsp={:?}", expected);

    let rsp:Binary = query(&mut deps, mock_env(),QueryMsg::ClaimStatus{ wallet : user2.clone() }).unwrap();
    let rsp: ClaimStatusResponse = from_binary(&rsp).unwrap();
    let expected =  ClaimStatusResponse { 
        claimable_ust: Uint128::from(4000u32),
        claimed_ust: None,
        total_ust : Uint128::from(10000u32),
        share : 4000u32 
    };
    assert_eq!(rsp, expected, "Claimable UST of user 2 should match rsp={:?}", expected);
    println!("{}",user1_info.clone().funds[0].amount);

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Claim{});
    assert_eq!(rsp.is_err(), false);
    println!("{}",user1_info.clone().funds[0].amount);
    assert_eq!(user1_info.clone().funds[0].amount, Uint128::from(2000u32), "User 1 should have 2000 ust in the wallet after claim");

}
