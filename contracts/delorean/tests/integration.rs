use cosmwasm_std::{coin, from_binary, ContractResult, Response};
use cosmwasm_vm::testing::{
    execute, instantiate, migrate, mock_backend, mock_env, mock_info, mock_instance_options, query,
};
use cosmwasm_vm::Instance;
use delorean_app::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use delorean_app::state::{Config, FundShare, Metadata, NftMetaInfo, RoundInfo, WinnerInfo};

static WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_app.wasm");

#[test]
fn delorean_distributer_test() {
    let backend = mock_backend(&[]);
    let admin = String::from("admin");
    let user1 = String::from("user1");
    let user2 = String::from("user2");

    let admin_info = mock_info(&admin, &&[coin(150000u128, "uusd")].to_vec());
    let mut user1_info = mock_info(&String::from("user1"), &[coin(10000u128, "uusd")].to_vec());
    let mut user2_info = mock_info(&String::from("user2"), &[coin(10000u128, "uusd")].to_vec());

    let config = Config {
        admin: admin.clone(),
        mint_contract: "xx1".to_string(),
        nft_contract: "xx2".to_string(),
        nfts: vec![
            NftMetaInfo {
                nft_metadata: Metadata {
                    name: Some(String::from("NFT 1")),
                    animation_url: None,
                    attributes: None,
                    background_color: None,
                    youtube_url: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                },
                price: coin(150000000u128, "uusd".to_string()),
                shares: vec![
                    FundShare {
                        address: user1.clone(),
                        note: "treasury".to_string(),
                        share: 4000u32, //20.00
                    },
                    FundShare {
                        address: user2.clone(),
                        note: "team".to_string(),
                        share: 6000u32, //40.00
                    },
                ],
            },
            NftMetaInfo {
                nft_metadata: Metadata {
                    name: Some(String::from("NFT 2")),
                    animation_url: None,
                    attributes: None,
                    background_color: None,
                    youtube_url: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                },
                price: coin(75000000u128, "uusd".to_string()),
                shares: vec![
                    FundShare {
                        address: user1.clone(),
                        note: "treasury".to_string(),
                        share: 4000u32, //20.00
                    },
                    FundShare {
                        address: user2.clone(),
                        note: "team".to_string(),
                        share: 000u32, //40.00
                    },
                ],
            },
        ],
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
    assert_eq!(rsp.unwrap_err(), "Unauthorized");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::ConfigUpdate {
            config: config.clone(),
        },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = migrate(&mut deps, mock_env(), MigrateMsg {});
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Mint { nft_type: 1 },
    );
    assert_eq!(rsp.unwrap_err(), "InsufficientFund");

    user1_info.funds = [coin(150000000u128, "uusd")].to_vec();
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Mint { nft_type: 1 },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::Mint { nft_type: 2 },
    );
    assert_eq!(rsp.is_err(), true);

    user2_info.funds = [coin(75000000u128, "uusd")].to_vec();
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::Mint { nft_type: 2 },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.unwrap_err(), "NotFound");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::WinnerUpdate {
            winner: WinnerInfo {
                winner_address: "user1".to_string(),
                winner_amount: coin(700000000u128, "uusd".to_string()),
                claimed: false,
                claim_end_time: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
            },
        },
    );
    assert_eq!(rsp.is_err(), true);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::RoundUpdate {
            round_info: RoundInfo {
                active: true,
                start_date: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
                name: "Day 1".to_string(),
                end_date: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
            },
        },
    );
    assert_eq!(rsp.unwrap_err(), "Unauthorized");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WinnerUpdate {
            winner: WinnerInfo {
                winner_address: "user1".to_string(),
                winner_amount: coin(700000000u128, "uusd".to_string()),
                claimed: false,
                claim_end_time: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
            },
        },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.unwrap_err(), "Unauthorized");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WinnerUpdate {
            winner: WinnerInfo {
                winner_address: "user1".to_string(),
                winner_amount: coin(700000000u128, "uusd".to_string()),
                claimed: true,
                claim_end_time: (mock_env().block.time.nanos() / 1_000_000_000) + 1000,
            },
        },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.unwrap_err(), "Unauthorized");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WinnerUpdate {
            winner: WinnerInfo {
                winner_address: "user1".to_string(),
                winner_amount: coin(700000000u128, "uusd".to_string()),
                claimed: false,
                claim_end_time: (mock_env().block.time.nanos() / 1_000_000_000) + 1000,
            },
        },
    );
    assert_eq!(rsp.is_err(), false);

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.is_err(), false);
}
