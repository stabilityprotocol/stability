#[derive(Clone, Debug, clap::Parser)]
pub struct StabilityConfiguration {

    /// HTTP URL of the private pool from which the node will retrieve zero-gas transactions
    #[arg(long, value_name = "URL")]
	pub zero_gas_tx_pool: Option<String>,

    /// Timeout in milliseconds for the zero-gas transaction pool
    /// (default: 1000)
    #[arg(long, value_name = "MILLISECONDS", default_value = "1000")]
    pub zero_gas_tx_pool_timeout: u64,
}