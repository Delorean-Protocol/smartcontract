use cosmwasm_std::{coin, from_binary, ContractResult, Response};
use cosmwasm_vm::testing::{
    execute, instantiate, migrate, mock_env, mock_info, mock_instance_options, query,
};
use cosmwasm_vm::Instance;
use delorean_app::msg::{
    ConfigResponse, DegenInfoResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use delorean_app::state::{Config, FundShare, Metadata, NftMetaInfo, RoundInfo, WinnerInfo};
use mock_tax::mock_dependencies::mock_dependencies_with_custom_querier;

static WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_app.wasm");

#[test]
fn delorean_distributer_test() {
    let backend = mock_dependencies_with_custom_querier(&[coin(900000000u128, "uusd".to_string())]);
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
                        share: 6000u32, //60.00
                    },
                    FundShare {
                        address: user2.clone(),
                        note: "team".to_string(),
                        share: 4000u32, //40.00
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

    assert_eq!(config_rsp.config, config, "Config updated successfully");

    let update_config_msg = ExecuteMsg::ConfigUpdate {
        config: config.clone(),
    };
    let rsp: ContractResult<Response> =
        execute(&mut deps, mock_env(), user1_info.clone(), update_config_msg);
    assert_eq!(
        rsp.unwrap_err(),
        "Unauthorized",
        "Config update should faill on unauthorized access"
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::ConfigUpdate {
            config: config.clone(),
        },
    );
    assert_eq!(
        rsp.is_err(),
        false,
        "Config update should pass on authorized access"
    );

    let rsp: ContractResult<Response> = migrate(&mut deps, mock_env(), MigrateMsg {});
    assert_eq!(rsp.is_err(), false, "Migrate should work");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Mint { nft_type: 1 },
    );
    assert_eq!(
        rsp.unwrap_err(),
        "InsufficientFund",
        "InsufficientFund test on mint of nft 1"
    );

    user1_info.funds = [coin(150000000u128, "uusd")].to_vec();
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Mint { nft_type: 1 },
    );
    assert_eq!(
        rsp.is_err(),
        false,
        "Mint with sufficient fund passes for nft 1"
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::Mint { nft_type: 2 },
    );
    assert_eq!(rsp.is_err(), true, "InsufficientFund test on mint of nft 2");

    user2_info.funds = [coin(75000000u128, "uusd")].to_vec();
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::Mint { nft_type: 2 },
    );
    assert_eq!(
        rsp.is_err(),
        false,
        "Mint with sufficient fund passes for nft 2"
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(
        rsp.unwrap_err(),
        "NotFound",
        "Claim prize should not work if winner not set"
    );

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
    assert_eq!(
        rsp.is_err(),
        true,
        "Set winner should fail for not authrozied wallet"
    );

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
    assert_eq!(
        rsp.unwrap_err(),
        "Unauthorized",
        "Round update should fail for not authrozied wallet"
    );

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
    assert_eq!(
        rsp.is_err(),
        false,
        "Winner update should work for authrozied wallet"
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(
        rsp.unwrap_err(),
        "Unauthorized",
        "Claim prize should fail for expired time"
    );

    let _rsp: ContractResult<Response> = execute(
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
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(
        rsp.unwrap_err(),
        "Unauthorized",
        "Claim prize should fail for already claimed"
    );

    let _rsp: ContractResult<Response> = execute(
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
    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::ClaimPrize {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.is_err(), false, "Claim prize should work");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user1_info.clone(),
        ExecuteMsg::Degen {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.is_err(), false, "Degen should work 1");

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        user2_info.clone(),
        ExecuteMsg::Degen {
            burn_nft_id: "1".to_string(),
        },
    );
    assert_eq!(rsp.is_err(), false, "Degen should work 2");

    let rsp = query(&mut deps, mock_env(), QueryMsg::DegenInfo {}).unwrap();
    let rsp: DegenInfoResponse = from_binary(&rsp).unwrap();
    assert_eq!(
        rsp,
        DegenInfoResponse {
            wallets: vec!["user1".to_string(), "user2".to_string()],
        },
        "Degen info response should work with proper response"
    );

    let rsp: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::RoundUpdate {
            round_info: RoundInfo {
                active: true,
                start_date: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
                name: "Day 1".to_string(),
                end_date: (mock_env().block.time.nanos() / 1_000_000_000) - 100000,
            },
        },
    );
    assert_eq!(
        rsp.is_err(),
        false,
        "Round update should pass with funds moving to different wallets"
    );

    let rsp = query(&mut deps, mock_env(), QueryMsg::DegenInfo {}).unwrap();
    let rsp: DegenInfoResponse = from_binary(&rsp).unwrap();
    assert_eq!(
        rsp,
        DegenInfoResponse { wallets: vec![] },
        "Degen info response should be blank after round is updated"
    );
}
