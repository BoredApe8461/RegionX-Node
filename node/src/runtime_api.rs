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

use cumulus_primitives_core::CollectCollationInfo;
use pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi;
use regionx_runtime_common::primitives::{AccountId, AuraId, Balance, Block, Nonce};
use sc_offchain::OffchainWorkerApi;
use sp_api::{ApiExt, Metadata};
use sp_block_builder::BlockBuilder;
use sp_consensus_aura::AuraApi;
use sp_core::H256;
use sp_session::SessionKeys;
use sp_transaction_pool::runtime_api::TaggedTransactionQueue;
use substrate_frame_rpc_system::AccountNonceApi;

pub trait BaseHostRuntimeApis:
	TaggedTransactionQueue<Block>
	+ ApiExt<Block>
	+ BlockBuilder<Block>
	+ AccountNonceApi<Block, AccountId, Nonce>
	+ Metadata<Block>
	+ AuraApi<Block, AuraId>
	+ OffchainWorkerApi<Block>
	+ SessionKeys<Block>
	+ CollectCollationInfo<Block>
	+ TransactionPaymentRuntimeApi<Block, Balance>
	+ ismp_parachain_runtime_api::IsmpParachainApi<Block>
	+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
	+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>
{
}

impl<Api> BaseHostRuntimeApis for Api where
	Api: TaggedTransactionQueue<Block>
		+ ApiExt<Block>
		+ BlockBuilder<Block>
		+ AccountNonceApi<Block, AccountId, Nonce>
		+ Metadata<Block>
		+ AuraApi<Block, AuraId>
		+ OffchainWorkerApi<Block>
		+ SessionKeys<Block>
		+ CollectCollationInfo<Block>
		+ TransactionPaymentRuntimeApi<Block, Balance>
		+ ismp_parachain_runtime_api::IsmpParachainApi<Block>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>
{
}
