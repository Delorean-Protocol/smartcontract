use cosmwasm_std::{ContractResult, Attribute, Response, coin, from_binary};
use cosmwasm_vm::testing::{
    instantiate, mock_backend, mock_env, mock_info, mock_instance_options, query, execute
};
use cosmwasm_vm::Instance;
use delorean_mint::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use delorean_mint::state::{
    Config, FundShare, Metadata
};
// const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
static WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/delorean_mint.wasm");
 
#[test]
fn delorean_distributer_test() {
    let backend = mock_backend(&[]);
    let admin = String::from("admin");
    let treasury = String::from("tressury");
    let team_fund = String::from("team_fund");
    let user1 = String::from("user1");
   

    let admin_info = mock_info(&admin, &[]);
    let user1_info = mock_info(&String::from("user1"), & [coin(150000000u128, "uusd")].to_vec());

    let shares = [ FundShare{
                address: team_fund.clone(),
                note: "".to_string(),
                share: 2000u32,  //20.00
            },
            FundShare{
                address: treasury.clone(),
                note: "".to_string(),
                share: 8000u32, //40.00
            }
    ].to_vec();

    let config =  Config{
        admin: admin,
        minter: String::from("minter"),
        nft_contract: String::from("nft_contract"),
        nft_metadata: Metadata { 
            name : Some(String::from("NFT 1")) ,
            animation_url : None,
            attributes : None,
            background_color : None,
            youtube_url : None,
            description : None,
            image : None,
            image_data : None ,
            external_url : None
        },
        shares: shares.to_vec(),
        price : coin(150000000u128, "uusd"),
        mint_limit: 4u32,
    };
    
    let instatiate_msg = InstantiateMsg {
        config : config.clone()
    };

    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();
    // make sure we can instantiate with this
    let res: ContractResult<Response> = instantiate(&mut deps, mock_env(), admin_info.clone(), instatiate_msg);
    let rsp = query(&mut deps, mock_env(), QueryMsg::Config{}).unwrap();
    let config_rsp: ConfigResponse = from_binary(&rsp).unwrap();
    
    assert_eq!(
        config_rsp.config,
        config
    );

    let update_config_msg = ExecuteMsg::ConfigUpdate {
        config : config.clone()
    };
    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), update_config_msg.clone());
    assert_eq!(rsp.is_err(), true);

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), admin_info.clone(), update_config_msg.clone());
    assert_eq!(rsp.is_err(), false);

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Mint{});
    assert_eq!(rsp.is_err(), false);
    assert_eq!(rsp.clone().unwrap().attributes, [ Attribute { key: String::from("action"), value: String::from("mint_nft_1"), }, Attribute { key: String::from("token_id"), value: String::from("1"),}]);
       

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Mint{});
    assert_eq!(rsp.is_err(), false);
    assert_eq!(rsp.clone().unwrap().attributes, [ Attribute { key: String::from("action"), value: String::from("mint_nft_1"), }, Attribute { key: String::from("token_id"), value: String::from("2"),}]);
       

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Mint{});
    assert_eq!(rsp.is_err(), false);
    assert_eq!(rsp.clone().unwrap().attributes, [ Attribute { key: String::from("action"), value: String::from("mint_nft_1"), }, Attribute { key: String::from("token_id"), value: String::from("3"),}]);
       

    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Mint{});
    assert_eq!(rsp.is_err(), false);
    assert_eq!(rsp.clone().unwrap().attributes, [ Attribute { key: String::from("action"), value: String::from("mint_nft_1"), }, Attribute { key: String::from("token_id"), value: String::from("4"),}]);
       
    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::Mint{});
    assert_eq!(rsp.is_err(), true);


    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), admin_info.clone(), ExecuteMsg::SecureMint{ owner: "user1".to_string(), token_uri : "".to_string(), extension :  config.nft_metadata.clone() });
    assert_eq!(rsp.is_err(), false);
    assert_eq!(rsp.clone().unwrap().attributes, [ Attribute { key: String::from("action"), value: String::from("secure_mint_nft"), }, Attribute { key: String::from("token_id"), value: String::from("5"),}]);
       
       
    let rsp:ContractResult<Response> = execute(&mut deps, mock_env(), user1_info.clone(), ExecuteMsg::SecureMint{ owner: "user1".to_string(), token_uri : "".to_string(), extension :  config.nft_metadata.clone() });
    assert_eq!(rsp.is_err(), true);
       
}
