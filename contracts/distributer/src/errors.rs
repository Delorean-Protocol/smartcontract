use cosmwasm_std::StdError;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility = "pub(crate)")]
pub enum ContractError {
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    #[snafu(display("StdError: {}", original))]
    Std { original: StdError },
    #[snafu(display("Unauthorized"))]
    Unauthorized { backtrace: Option<snafu::Backtrace> },

    #[snafu(display("EmptyBalance"))]
    EmptyBalance {},

    #[snafu(display("InsufficientFund"))]
    InsufficientFund {},

    #[snafu(display("MintLimitReached"))]
    MintLimitReached {},

    #[snafu(display("NotFound"))]
    NotFound {},

    #[snafu(display("PriceMismatch"))]
    PriceMismatch {},
}

impl From<StdError> for ContractError {
    fn from(original: StdError) -> Self {
        Std { original }.build()
    }
}
