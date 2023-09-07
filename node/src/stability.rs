#[derive(Clone, Debug, clap::Parser)]
pub struct StabilityConfiguration {

    /// HTTP URL of the private pool from which the node will retrieve zero-gas transactions
    #[arg(long, value_name = "URL")]
	pub zero_gas_tx_pool: Option<String>,
}