//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use std::time::Duration;
use sc_client::LongestChain;
use node_template_runtime::{self, GenesisConfig, opaque::Block, RuntimeApi};
use sc_service::{error::Error as ServiceError, AbstractService, Configuration, ServiceBuilder};
use sp_inherents::InherentDataProviders;
use sc_network::construct_simple_protocol;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
use sc_basic_authority;
use sc_consensus_manual_seal::{run_manual_seal, rpc, import_queue};
use sp_core::H256;
use futures::{TryFutureExt, FutureExt};


// Our native executor instance.
native_executor_instance!(
	pub Executor,
	node_template_runtime::api::dispatch,
	node_template_runtime::native_version,
);

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.

/// Builds a new service for a full client.
pub fn new_full<C: Send + Default + 'static>(config: Configuration<C, GenesisConfig>)
	-> Result<impl AbstractService, ServiceError>
{
	let mut pool_ = None;
	let mut backend_ = None;
	let mut client_ = None;
	let mut select_chain_ = None;
	let mut stream = None;
	let (builder, inherent_data_providers) = {
		let inherent_data_providers = sp_inherents::InherentDataProviders::new();

		let builder = sc_service::ServiceBuilder::new_full::<
			node_template_runtime::opaque::Block, node_template_runtime::RuntimeApi, crate::service::Executor
		>(config)?.with_select_chain(|_config, backend| {
			backend_ = Some(backend.clone());
			let select_chain = sc_client::LongestChain::new(backend.clone());
			select_chain_ = Some(select_chain.clone());
			Ok(select_chain)
		})?
			.with_transaction_pool(|config, client, _fetcher| {
				client_ = Some(client.clone());
				let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
				let pool = sc_transaction_pool::BasicPool::new(config, pool_api);
				pool_ = Some(pool.pool().clone());

				let maintainer = sc_transaction_pool::FullBasicPoolMaintainer::new(pool.pool().clone(), client);
				let maintainable_pool = sp_transaction_pool::MaintainableTransactionPool::new(pool, maintainer);
				Ok(maintainable_pool)
			})?
			.with_import_queue(|_, client, _a, _b| {
				Ok(import_queue(Box::new(client)))
			})?
			.with_network_protocol(|_| Ok(NodeProtocol::new()))?
			.with_rpc_extensions(|_, _a, _b, _c, _d| {
				let mut io = jsonrpc_core::IoHandler::default();
				let (sender, receiver) = futures::channel::mpsc::channel(100);
				stream = Some(receiver);
				io.extend_with(
					rpc::ManualSealApi::<H256>::to_delegate(rpc::ManualSeal::new(sender))
				);

				Ok(io)
			})?;

		(builder, inherent_data_providers)
	};

	let service = builder.build()?;

	let proposer = sc_basic_authority::ProposerFactory {
		client: service.client(),
		transaction_pool: service.transaction_pool(),
	};
	let future = run_manual_seal(
		Box::new(client_.unwrap().clone()),
		proposer,
		backend_.unwrap().clone(),
		pool_.unwrap().clone(),
		stream.unwrap(),
		select_chain_.unwrap(),
		inherent_data_providers,
	).boxed();

	service.spawn_essential_task(future.map(|()| Ok::<(), ()>(())).compat());

	Ok(service)
}

// Builds a new service for a light client.
//pub fn new_light<C: Send + Default + 'static>(config: Configuration<C, GenesisConfig>)
//                                              -> Result<impl AbstractService, ServiceError>
//{
//	let inherent_data_providers = InherentDataProviders::new();
//
//	ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
//		.with_select_chain(|_config, backend| {
//			Ok(LongestChain::new(backend.clone()))
//		})?
//		.with_transaction_pool(|config, client, fetcher| {
//			let fetcher = fetcher
//				.ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;
//			let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
//			let pool = sc_transaction_pool::BasicPool::new(config, pool_api);
//			let maintainer = sc_transaction_pool::LightBasicPoolMaintainer::with_defaults(pool.pool().clone(), client, fetcher);
//			let maintainable_pool = sp_transaction_pool::MaintainableTransactionPool::new(pool, maintainer);
//			Ok(maintainable_pool)
//		})?
//		.with_import_queue_and_fprb(|_config, client, backend, fetcher, _select_chain, _tx_pool| {
//			let fetch_checker = fetcher
//				.map(|fetcher| fetcher.checker().clone())
//				.ok_or_else(|| "Trying to start light import queue without active fetch checker")?;
//			let grandpa_block_import = grandpa::light_block_import::<_, _, _, RuntimeApi>(
//				client.clone(), backend, &*client.clone(), Arc::new(fetch_checker),
//			)?;
//			let finality_proof_import = grandpa_block_import.clone();
//			let finality_proof_request_builder =
//				finality_proof_import.create_finality_proof_request_builder();
//
//			let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, ()>(
//				sc_consensus_aura::SlotDuration::get_or_compute(&*client)?,
//				grandpa_block_import,
//				None,
//				Some(Box::new(finality_proof_import)),
//				client,
//				inherent_data_providers.clone(),
//				None,
//			)?;
//
//			Ok((import_queue, finality_proof_request_builder))
//		})?
//		.with_network_protocol(|_| Ok(NodeProtocol::new()))?
//		.with_finality_proof_provider(|client, backend|
//			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
//		)?
//		.build()
//}
