// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! RPC interface for the transaction payment module.

use std::sync::Arc;
use sp_blockchain::HeaderBackend;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, ProvideRuntimeApi, UniqueSaturatedInto},
};
use sp_core::Bytes;
//use pallet_transaction_payment_rpc_runtime_api::CappedDispatchInfo;
//pub use pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi as TransactionPaymentRuntimeApi;
// pub use self::gen_client::Client as TransactionPaymentClient;

#[rpc]
pub trait SumStorageApi<BlockHash> {
	#[rpc(name = "sumStorage_getSum")]
	fn get_sum(
		&self,
		at: Option<BlockHash>
	) -> Result<u32>;
}

/// A struct that implements the `SumStorageApi`.
pub struct SumStorage<C> {
	client: Arc<C>,
	// Maybe add a phantom marker for block here?
	// Or maybe just delete the block and at stuff entirely?
}

impl<C> SumStorage<C> {
	/// Create new `SumStorage` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client }
	}
}

/// Error type of this RPC api.
// pub enum Error {
// 	/// The transaction was not decodable.
// 	DecodeError,
// 	/// The call to runtime failed.
// 	RuntimeError,
// }
//
// impl From<Error> for i64 {
// 	fn from(e: Error) -> i64 {
// 		match e {
// 			Error::RuntimeError => 1,
// 			Error::DecodeError => 2,
// 		}
// 	}
// }

impl<C, Block> SumStorageApi<<Block as BlockT>::Hash>
	for SumStorage<C>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi,
	C: HeaderBackend<Block>,
	// C::Api: TransactionPaymentRuntimeApi<Block, Balance, Extrinsic>,
{
	fn get_sum(
		&self,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<u32> {
		// let api = self.client.runtime_api();
		// let at = BlockId::hash(at.unwrap_or_else(||
		// 	// If the block hash is not supplied assume the best block.
		// 	self.client.info().best_hash
		// ));
		//
		// let encoded_len = encoded_xt.len() as u32;
		//
		// let uxt: Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| RpcError {
		// 	code: ErrorCode::ServerError(Error::DecodeError.into()),
		// 	message: "Unable to query dispatch info.".into(),
		// 	data: Some(format!("{:?}", e).into()),
		// })?;
		// api.query_info(&at, uxt, encoded_len).map_err(|e| RpcError {
		// 	code: ErrorCode::ServerError(Error::RuntimeError.into()),
		// 	message: "Unable to query dispatch info.".into(),
		// 	data: Some(format!("{:?}", e).into()),
		// }).map(CappedDispatchInfo::new)

		Ok(1337)
	}
}
