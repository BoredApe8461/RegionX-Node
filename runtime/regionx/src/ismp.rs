// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(not(feature = "std"))]
use crate::alloc::string::ToString;
use crate::{AccountId, Ismp, IsmpParachain, ParachainInfo, Runtime, RuntimeEvent, Timestamp};
use frame_support::pallet_prelude::Get;
use frame_system::EnsureRoot;
use ismp::{
	error::Error,
	host::StateMachine,
	module::IsmpModule,
	router::{IsmpRouter, Post, Request, Response, Timeout},
};
use ismp_parachain::ParachainConsensusClient;
use pallet_ismp::{dispatcher::FeeMetadata, primitives::ModuleId};
use sp_std::prelude::*;

pub struct HostStateMachine;
impl Get<StateMachine> for HostStateMachine {
	fn get() -> StateMachine {
		StateMachine::Kusama(ParachainInfo::get().into())
	}
}

pub struct Coprocessor;

impl Get<Option<StateMachine>> for Coprocessor {
	fn get() -> Option<StateMachine> {
		Some(HostStateMachine::get())
	}
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	const INDEXING_PREFIX: &'static [u8] = b"ISMP";
	type AdminOrigin = EnsureRoot<AccountId>;
	type HostStateMachine = HostStateMachine;
	type Coprocessor = Coprocessor;
	type TimeProvider = Timestamp;
	type Router = Router;
	type ConsensusClients = (ParachainConsensusClient<Runtime, IsmpParachain>,);
	type WeightInfo = ();
	type WeightProvider = ();
}

#[derive(Default)]
pub struct ProxyModule;

impl IsmpModule for ProxyModule {
	fn on_accept(&self, _request: Post) -> Result<(), Error> {
		// we don't support any incoming post requests.
		Err(Error::CannotHandleMessage)
	}

	fn on_response(&self, response: Response) -> Result<(), Error> {
		if response.dest_chain() != HostStateMachine::get() {
			let meta = FeeMetadata { origin: [0u8; 32].into(), fee: Default::default() };
			return Ismp::dispatch_response(response, meta);
		}

		let request = &response.request();
		let from = match &request {
			Request::Post(post) => &post.from,
			Request::Get(get) => &get.from,
		};

		let pallet_id = ModuleId::from_bytes(from)
			.map_err(|err| Error::ImplementationSpecific(err.to_string()))?;

		#[allow(clippy::match_single_binding)]
		match pallet_id {
			// TODO: route to regions pallet
			_ => Err(Error::ImplementationSpecific("Destination module not found".to_string())),
		}
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), Error> {
		let from = match &timeout {
			Timeout::Request(Request::Post(post)) => &post.from,
			Timeout::Request(Request::Get(get)) => &get.from,
			Timeout::Response(res) => &res.post.to,
		};

		let pallet_id = ModuleId::from_bytes(from)
			.map_err(|err| Error::ImplementationSpecific(err.to_string()))?;

		#[allow(clippy::match_single_binding)]
		match pallet_id {
			// TODO: route to regions pallet
			// instead of returning an error, do nothing. The timeout is for a connected chain.
			_ => Ok(()),
		}
	}
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, _bytes: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		Ok(Box::new(ProxyModule))
	}
}
