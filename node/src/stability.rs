// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use fc_rpc::pending::ConsensusDataProvider;
use sc_client_api::{AuxStore, UsageProvider};
use sp_api::ProvideRuntimeApi;
use sp_consensus_aura::Slot;
use sp_consensus_aura::SlotDuration;
use sp_consensus_aura::{digests::CompatibleDigestItem, AuraApi};
use sp_inherents::InherentData;
use sp_runtime::{traits::Block as BlockT, Digest, DigestItem};
use sp_timestamp::TimestampInherentData;
use std::{marker::PhantomData, sync::Arc};

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

/// StbleAuraConsensusDataProvider provides the data required for the Aura consensus engine.
/// This is a workaround for making work the unified accounts feature with Aura consensus.
pub struct StbleAuraConsensusDataProvider<B, C> {
	slot_duration: SlotDuration,
	#[allow(dead_code)]
	client: Arc<C>,
	phantom_data: PhantomData<B>,
}

impl<B, C> StbleAuraConsensusDataProvider<B, C>
where
	B: BlockT,
	C: AuxStore + ProvideRuntimeApi<B> + UsageProvider<B> + Send + Sync,
	C::Api: AuraApi<B, stbl_core_primitives::aura::Public>,
{
	pub fn new(client: Arc<C>) -> Self {
		let slot_duration = sc_consensus_aura::slot_duration(&*client)
			.expect("slot_duration is always present; qed.");
		Self {
			slot_duration,
			client,
			phantom_data: Default::default(),
		}
	}
}

impl<B, C> ConsensusDataProvider<B> for StbleAuraConsensusDataProvider<B, C>
where
	B: BlockT,
	C: AuxStore + ProvideRuntimeApi<B> + UsageProvider<B> + Send + Sync,
	C::Api: AuraApi<B, stbl_core_primitives::aura::Public>,
{
	fn create_digest(
		&self,
		_parent: &B::Header,
		data: &InherentData,
	) -> Result<Digest, sp_inherents::Error> {
		let timestamp = data
			.timestamp_inherent_data()?
			.expect("Timestamp is always present; qed");

		let digest_item = <DigestItem as CompatibleDigestItem<
			stbl_core_primitives::aura::Signature,
		>>::aura_pre_digest(Slot::from_timestamp(timestamp, self.slot_duration));

		Ok(Digest {
			logs: vec![digest_item],
		})
	}
}
