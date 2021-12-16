use cosmwasm_std::{
    coin, from_binary, ContractResult, Response,
};
use cosmwasm_vm::testing::{
    execute, instantiate, migrate, mock_backend, mock_env, mock_info, mock_instance_options, query,
};
use cosmwasm_vm::Instance;
use delorean_treasury::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use delorean_treasury::state::Config;

static WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_treasury.wasm");

#[test]
fn delorean_distributer_test() {
    let backend = mock_backend(&[]);
    let admin = String::from("admin");

    let admin_info = mock_info(&admin, &&[coin(150000u128, "uusd")].to_vec());
    let user1_info = mock_info(&String::from("user1"), &[coin(10000u128, "uusd")].to_vec());

    let config = Config {
        admin: admin.clone(),
        anchor_smart_contract: "anchor_smart_contract".to_string(),
    };

    let instatiate_msg = InstantiateMsg {
        config: config.clone(),
    };

    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();
    // make sure we can instantiate with this
    let _res: ContractResult<Response> =
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
}
