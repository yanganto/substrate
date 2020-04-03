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

//! token crypto types.

use crate::{RuntimePublic, KeyTypeId};

use sp_std::vec::Vec;
pub use sp_core::token::Public;


impl RuntimePublic for Public {
	type Signature = Vec<u8>;

	fn all(key_type: KeyTypeId) -> crate::Vec<Self> {
		// TOOD
		crate::Vec::<Self>::new()
	}

	fn generate_pair(key_type: KeyTypeId, _seed: Option<Vec<u8>>) -> Self {
		sp_io::crypto::token_generate(key_type)
	}

	fn sign<M: AsRef<[u8]>>(&self, key_type: KeyTypeId, _msg: &M) -> Option<Self::Signature> {
		Some(sp_io::crypto::token_generate(key_type).to_vec()) //TODO: hacky
	}

	fn verify<M: AsRef<[u8]>>(&self, _msg: &M, _signature: &Self::Signature) -> bool {
		true
	}

	fn to_raw_vec(&self) -> Vec<u8> {
		sp_core::crypto::Public::to_raw_vec(self)
	}
}
