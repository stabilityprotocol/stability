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

//! Prometheus basic proposer metrics.

use prometheus_endpoint::{
	prometheus::CounterVec, register, Gauge, Histogram, HistogramOpts, Opts, PrometheusError,
	Registry, U64,
};

/// Optional shareable link to basic authorship metrics.
#[derive(Clone, Default)]
pub struct MetricsLink(Option<Metrics>);

impl MetricsLink {
	pub fn new(registry: Option<&Registry>) -> Self {
		Self(registry.and_then(|registry| {
			Metrics::register(registry)
				.map_err(|err| {
					log::warn!("Failed to register proposer prometheus metrics: {}", err)
				})
				.ok()
		}))
	}

	pub fn report<O>(&self, do_this: impl FnOnce(&Metrics) -> O) -> Option<O> {
		self.0.as_ref().map(do_this)
	}
}

/// The reason why proposing a block ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndProposingReason {
	NoMoreTransactions,
	HitDeadline,
	HitBlockSizeLimit,
	HitBlockWeightLimit,
	/// No transactions are allowed in the block.
	TransactionForbidden,
}

/// Authorship metrics.
#[derive(Clone)]
pub struct Metrics {
	pub block_constructed: Histogram,
	pub number_of_transactions: Gauge<U64>,
	pub end_proposing_reason: CounterVec,
	pub create_inherents_time: Histogram,
	pub create_block_proposal_time: Histogram,
	pub zgt_response_time: Histogram,
	pub zgt_inclusion_in_block_time: Histogram,
	pub normal_extrinsic_inclusion_in_block_time: Histogram,
}

impl Metrics {
	pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
		Ok(Self {
			block_constructed: register(
				Histogram::with_opts(HistogramOpts::new(
					"substrate_proposer_block_constructed",
					"Histogram of time taken to construct new block",
				))?,
				registry,
			)?,
			number_of_transactions: register(
				Gauge::new(
					"substrate_proposer_number_of_transactions",
					"Number of transactions included in block",
				)?,
				registry,
			)?,
			create_inherents_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"substrate_proposer_create_inherents_time",
					"Histogram of time taken to execute create inherents",
				))?,
				registry,
			)?,
			create_block_proposal_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"substrate_proposer_block_proposal_time",
					"Histogram of time taken to construct a block and prepare it for proposal",
				))?,
				registry,
			)?,
			end_proposing_reason: register(
				CounterVec::new(
					Opts::new(
						"substrate_proposer_end_proposal_reason",
						"The reason why the block proposing was ended. This doesn't include errors.",
					),
					&["reason"],
				)?,
				registry,
			)?,
			zgt_response_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"stability_proposer_zgt_response_time",
					"Histogram of time taken to get ZGT api response",
				))?,
				registry,
			)?,
			zgt_inclusion_in_block_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"stability_proposer_zgt_inclusion_in_block_time",
					"Histogram of time taken to include ZGT transactions in block",
				))?,
				registry,
			)?,
			normal_extrinsic_inclusion_in_block_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"stability_proposer_normal_extrinsic_inclusion_in_block_time",
					"Histogram of time taken to include normal extrinsics in block",
				))?,
				registry,
			)?,
		})
	}

	/// Report the reason why the proposing ended.
	pub fn report_end_proposing_reason(&self, reason: EndProposingReason) {
		let reason = match reason {
			EndProposingReason::HitDeadline => "hit_deadline",
			EndProposingReason::NoMoreTransactions => "no_more_transactions",
			EndProposingReason::HitBlockSizeLimit => "hit_block_size_limit",
			EndProposingReason::HitBlockWeightLimit => "hit_block_weight_limit",
			EndProposingReason::TransactionForbidden => "transactions_forbidden",
		};

		self.end_proposing_reason.with_label_values(&[reason]).inc();
	}
}
