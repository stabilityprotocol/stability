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

use futures::TryFutureExt;
// Substrate
use sc_cli::{ChainSpec, SubstrateCli};
use sc_service::DatabaseSource;
// Frontier
use fc_db::kv::frontier_database_dir;

use crate::{
	chain_spec,
	cli::{Cli, Subcommand},
	service::{self, db_config_dir},
};

#[cfg(feature = "runtime-benchmarks")]
use crate::chain_spec::get_account_id_from_seed;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Frontier Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2021
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
		Ok(match id {
			"alphanet" => Box::new(chain_spec::alphanet::alphanet_config()),
			"betanet" => Box::new(chain_spec::betanet::betanet_config()),
			"testnet" => Box::new(chain_spec::testnet::testnet_config()),
			"" | "local" | "dev" => {
				let enable_manual_seal = self.sealing.map(|_| true).unwrap_or_default();
				Box::new(chain_spec::dev::development_config(enable_manual_seal))
			}
			path => Box::new(chain_spec::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// Remove Frontier offchain db
				let db_config_dir = db_config_dir(&config);
				match cli.eth.frontier_backend_type {
					crate::eth::BackendType::KeyValue => {
						let frontier_database_config = match config.database {
							DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
								path: frontier_database_dir(&db_config_dir, "db"),
								cache_size: 0,
							},
							DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
								path: frontier_database_dir(&db_config_dir, "paritydb"),
							},
							_ => {
								return Err(format!(
									"Cannot purge `{:?}` database",
									config.database
								)
								.into())
							}
						};
						cmd.run(frontier_database_config)?;
					}
					crate::eth::BackendType::Sql => {
						let db_path = db_config_dir.join("sql");
						match std::fs::remove_dir_all(&db_path) {
							Ok(_) => {
								println!("{:?} removed.", &db_path);
							}
							Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
								eprintln!("{:?} did not exist.", &db_path);
							}
							Err(err) => {
								return Err(format!(
									"Cannot purge `{:?}` database: {:?}",
									db_path, err,
								)
								.into())
							}
						};
					}
				};
				cmd.run(config.database)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let aux_revert = Box::new(move |client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		}
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			use crate::benchmarking::{
				inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder,
			};
			use frame_benchmarking_cli::{
				BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE,
			};
			use stability_runtime::{Hashing, EXISTENTIAL_DEPOSIT};

			let runner = cli.create_runner(cmd)?;
			match cmd {
				BenchmarkCmd::Pallet(cmd) => runner
					.sync_run(|config| cmd.run_with_spec::<Hashing, ()>(Some(config.chain_spec))),
				BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					cmd.run(client)
				}),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
					let (client, backend, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					let db = backend.expose_db();
					let storage = backend.expose_storage();
					cmd.run(config, client, db, storage)
				}),
				BenchmarkCmd::Overhead(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					let ext_builder = RemarkBuilder::new(client.clone());
					cmd.run(
						config,
						client,
						inherent_benchmark_data()?,
						Vec::new(),
						&ext_builder,
					)
				}),
				BenchmarkCmd::Extrinsic(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					// Register the *Remark* and *TKA* builders.
					let ext_factory = ExtrinsicFactory(vec![
						Box::new(RemarkBuilder::new(client.clone())),
						Box::new(TransferKeepAliveBuilder::new(
							client.clone(),
							get_account_id_from_seed::<sp_core::ecdsa::Public>("Alice"),
							EXISTENTIAL_DEPOSIT,
						)),
					]);

					cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
				}),
				BenchmarkCmd::Machine(cmd) => {
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
				}
			}
		}
		#[cfg(not(feature = "runtime-benchmarks"))]
		Some(Subcommand::Benchmark) => Err("Benchmarking wasn't enabled when building the node. \
			You can enable it with `--features runtime-benchmarks`."
			.into()),
		Some(Subcommand::FrontierDb(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|mut config| {
				let (client, _, _, _, frontier_backend) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let frontier_backend = match frontier_backend {
					fc_db::Backend::KeyValue(kv) => kv,
					_ => panic!("Only fc_db::Backend::KeyValue supported"),
				};
				cmd.run(client, frontier_backend)
			})
		}
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::build_full(config, cli.eth, cli.sealing, cli.stability)
					.map_err(Into::into)
					.await
			})
		}
	}
}
