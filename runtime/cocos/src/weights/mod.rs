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

// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Expose the auto generated weight files.

pub mod block_weights;
pub mod cumulus_pallet_parachain_system;
pub mod cumulus_pallet_xcmp_queue;
pub mod extrinsic_weights;
pub mod frame_system;
pub mod pallet_balances;
pub mod pallet_collator_selection;
pub mod pallet_conviction_voting_delegated_conviction_voting;
pub mod pallet_conviction_voting_native_conviction_voting;
pub mod pallet_market;
pub mod pallet_multisig;
pub mod pallet_orders;
pub mod pallet_proxy;
pub mod pallet_regions;
pub mod pallet_session;
pub mod pallet_sudo;
pub mod pallet_timestamp;
pub mod pallet_utility;
pub mod pallet_whitelist;
pub mod paritydb_weights;
pub mod rocksdb_weights;

pub use block_weights::constants::BlockExecutionWeight;
pub use extrinsic_weights::constants::ExtrinsicBaseWeight;
pub use rocksdb_weights::constants::RocksDbWeight;
