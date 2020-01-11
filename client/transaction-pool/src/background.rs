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

//! Background tasks for transaction queue

use std::{
	sync::Arc,
	time::Duration,
	collections::{HashMap, BTreeMap},
};

use futures::{prelude::*, channel::mpsc};

use sc_transaction_graph::{self, ChainApi, TransactionFor, ExtrinsicFor, ExHash, NumberFor};
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Extrinsic, Header, SimpleArithmetic},
};

struct RevalidationQueue<A: ChainApi> {
	block_ordered: BTreeMap<NumberFor<A>, HashMap<ExHash<A>, TransactionFor<A>>>,
	members: HashMap<ExHash<A>, NumberFor<A>>,
}

struct BackgroundTasksState<A: ChainApi> {
	revalidation_queue: RevalidationQueue<A>,
	best_block_seen: NumberFor<A>,
}

pub enum MaintainerUpdate<A: ChainApi>
{
	Block(BlockId<<A as ChainApi>::Block>)
}

pub struct BackgroundTasks<A: ChainApi> {
	pool: Arc<sc_transaction_graph::Pool<A>>,
	state: BackgroundTasksState<A>,
}

impl<A: ChainApi> BackgroundTasks<A> {

	pub fn new(
		pool: Arc<sc_transaction_graph::Pool<A>>,
	) -> BackgroundTasks<A>
	{
		Self {
			pool,
			state: BackgroundTasksState::<A>::default(),
		}
	}

	pub async fn main(&mut self, mut notifications: mpsc::UnboundedReceiver<MaintainerUpdate<A>>)
	{
		let current_state = &mut self.state;

		// TODO: Interval can be adaptive (if it is something left in queues, it decreases,
		//       if it always exhausted, then it increases.
		let mut interval = interval(std::time::Duration::from_millis(200)).fuse();

		loop {
			futures::select! {
				notification = notifications.next() => {
					log::trace!(target: "txpool", "Background worker: notification received");

					match notification {
						Some(MaintainerUpdate::Block(id)) => {
							let block_number = match self.pool.resolve_block_number(&id) {
								Ok(val) => val,
								Err(e) => {
									log::warn!(target: "txpool", "Error receiving unknown block in notifications");
									continue;
								}
							};

							current_state.best_block_seen = block_number;

							let ready_txes = self.pool.ready().collect::<Vec<TransactionFor<A>>>();
							current_state.revalidation_queue.push(block_number, ready_txes);

							if let Err(err) = current_state.revalidation_queue
								.process(block_number, 20, &self.pool).await
							{
								log::warn!(target: "txpool", "Error revalidating transactions: {}", err);
							}
						},
						None => {},
					}
				},
				_ = interval.next() => {
					if current_state.revalidation_queue.is_empty() {
						continue;
					}

					log::trace!(target: "txpool", "Background worker: waking");

					if let Err(err) = current_state.revalidation_queue
						.process(current_state.best_block_seen, 20, &self.pool).await
					{
						log::warn!(target: "txpool", "Error revalidating transactions: {}", err);
					}
				},
			};
		}
	}
}


fn interval(duration: Duration) -> impl Stream<Item = ()> + Unpin {
	futures::stream::unfold((),
		move |_| futures_timer::Delay::new(duration).map(|_| Some(((), ())))
	)
}

impl<A: ChainApi> Default for RevalidationQueue<A> {
	fn default() -> Self {
		RevalidationQueue {
			block_ordered: Default::default(),
			members: Default::default(),
		}
	}
}

impl<A: ChainApi> RevalidationQueue<A> {
	pub fn push(&mut self, block_number: NumberFor<A>, transactions: Vec<TransactionFor<A>>) {
		for ext in transactions.into_iter() {

			// we don't add something that already scheduled for revalidation
			if self.members.contains_key(&ext.hash) { continue; }

			self.block_ordered.entry(block_number)
				.and_modify(|value| { value.insert(ext.hash.clone(), ext.clone()); })
				.or_insert_with(|| {
					let mut bt = HashMap::new();
					bt.insert(ext.hash.clone(), ext.clone());
					bt
				});
			self.members.insert(ext.hash.clone(), block_number);
		}
	}

	pub fn is_empty(&self) -> bool {
		return self.block_ordered.is_empty()
	}

	fn process(&mut self, block_number: NumberFor<A>, count: usize, pool: &sc_transaction_graph::Pool<A>)
		-> impl Future<Output=Result<(), A::Error>>
	{
		let mut queued_exts = Vec::new();

		let mut empty = Vec::<NumberFor<A>>::new();
		// Take maximum of count transaction by order
		// which they got into the pool
		for (block_number, mut ext_map) in self.block_ordered.iter_mut() {
			if queued_exts.len() >= count { break; }

			loop {
				let next_key = match ext_map.keys().nth(0) {
					Some(k) => k.clone(),
					None => { break; }
				};

				let ext = ext_map.remove(&next_key).expect("Just checked the key");
				self.members.remove(&next_key);

				queued_exts.push(ext);

				if ext_map.len() == 0 { empty.push(*block_number); }

				if queued_exts.len() >= count { break; }
			}
		}

		// retain only non-empty
		for empty_block_number in empty.into_iter() { self.block_ordered.remove(&empty_block_number); }

		pool.revalidate(
			&BlockId::number(block_number),
			queued_exts.into_iter().map(|x| x.data.clone())
		)
	}
}

impl<A: ChainApi> Default for BackgroundTasksState<A> {
	fn default() -> Self {
		use num_traits::identities::Zero;
		BackgroundTasksState {
			revalidation_queue: Default::default(),
			best_block_seen: NumberFor::<A>::zero(),
		}
	}
}

