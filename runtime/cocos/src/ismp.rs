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
use crate::{
	AccountId, Balance, Balances, Ismp, IsmpParachain, ParachainInfo, Runtime, RuntimeEvent,
	Timestamp,
};
use frame_support::{pallet_prelude::Get, parameter_types};
use frame_system::EnsureRoot;
use ismp::{error::Error, host::StateMachine, module::IsmpModule, router::IsmpRouter};
use ismp_parachain::ParachainConsensusClient;
use pallet_ismp::{weights::IsmpModuleWeight, ModuleId, NoOpMmrTree};
use sp_std::prelude::*;

pub struct HostStateMachine;

impl Get<StateMachine> for HostStateMachine {
	fn get() -> StateMachine {
		StateMachine::Kusama(ParachainInfo::get().into())
	}
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = None;
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IsmpHost = Ismp;
}

pub struct WeightProvider;
impl pallet_ismp::weights::WeightProvider for WeightProvider {
	fn module_callback(id: ModuleId) -> Option<Box<dyn IsmpModuleWeight>> {
		match id {
			pallet_regions::PALLET_ID =>
				Some(Box::<pallet_regions::IsmpRegionsModuleWeight<Runtime>>::default()),
			_ => None,
		}
	}
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AdminOrigin = EnsureRoot<AccountId>;
	type HostStateMachine = HostStateMachine;
	type TimestampProvider = Timestamp;
	type Router = Router;
	type Balance = Balance;
	type Currency = Balances;
	type Coprocessor = Coprocessor;
	type ConsensusClients = (ParachainConsensusClient<Runtime, IsmpParachain>,);

	type Mmr = NoOpMmrTree<Self>;
	type WeightProvider = WeightProvider;
}

#[derive(Default)]
pub struct Router;
impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		let module = match ModuleId::from_bytes(&id) {
			Ok(pallet_regions::PALLET_ID) =>
				Box::<pallet_regions::IsmpModuleCallback<Runtime>>::default(),
			_ => Err(Error::ModuleNotFound(id))?,
		};
		Ok(module)
	}
}
