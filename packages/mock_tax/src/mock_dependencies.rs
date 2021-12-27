use cosmwasm_std::{to_binary, Binary, Coin, ContractResult, Decimal, SystemResult, Uint128};
use cosmwasm_vm::{
    testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    Backend,
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

pub fn custom_query_execute(request: &TerraQueryWrapper) -> ContractResult<Binary> {
    match &request {
        TerraQueryWrapper { route, query_data } => {
            if &TerraRoute::Treasury == route {
                match query_data {
                    TerraQuery::TaxRate {} => {
                        let res = TaxRateResponse {
                            rate: Decimal::zero(),
                        };
                        return ContractResult::from(to_binary(&res));
                    }
                    TerraQuery::TaxCap { denom: _ } => {
                        let res = TaxCapResponse {
                            cap: Uint128::from(0u32),
                        };
                        return ContractResult::from(to_binary(&res));
                    }
                    _ => panic!("DO NOT ENTER HERE"),
                }
            } else {
                panic!("DO NOT ENTER HERE")
            }
        }
    }
}

pub fn mock_dependencies_with_custom_querier(
    contract_balance: &[Coin],
) -> Backend<MockApi, MockStorage, MockQuerier<TerraQueryWrapper>> {
    let custom_querier: MockQuerier<TerraQueryWrapper> =
        MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)])
            .with_custom_handler(|query| SystemResult::Ok(custom_query_execute(query)));

    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}
