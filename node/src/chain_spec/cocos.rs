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

use crate::chain_spec::{
	get_account_id_from_seed, get_collator_keys_from_seed, ChainSpec, Extensions,
};
use cocos_runtime::{COCOS, COCOS_EXISTENTIAL_DEPOSIT, ROC_EXISTENTIAL_DEPOSIT};
use cumulus_primitives_core::ParaId;
use orml_asset_registry::AssetMetadata;
use regionx_runtime_common::{
	assets::{AssetsStringLimit, RELAY_CHAIN_ASSET_ID},
	primitives::{AccountId, AuraId, Balance},
};
use sc_service::ChainType;
use sp_core::{sr25519, Encode};
use xcm::opaque::lts::MultiLocation;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn session_keys(keys: AuraId) -> cocos_runtime::SessionKeys {
	cocos_runtime::SessionKeys { aura: keys }
}

pub fn cocos_config(id: u32) -> ChainSpec<cocos_runtime::RuntimeGenesisConfig> {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "COCOS".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		cocos_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions { relay_chain: "rococo".into(), para_id: id },
	)
	.with_name("RegionX Cocos")
	.with_id("cocos")
	.with_chain_type(ChainType::Live)
	.with_genesis_config_patch(cocos_genesis(id.into()))
	.with_protocol_id("cocos")
	.with_properties(properties)
	.build()
}

pub fn development_config(id: u32) -> ChainSpec<cocos_runtime::RuntimeGenesisConfig> {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "COCOS".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		cocos_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: id,
		},
	)
	.with_name("RegionX Cocos Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		id.into(),
	))
	.with_protocol_id("regionx-dev")
	.with_properties(properties)
	.build()
}

pub fn local_testnet_config(id: u32) -> ChainSpec<cocos_runtime::RuntimeGenesisConfig> {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "COCOS".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		cocos_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: id,
		},
	)
	.with_name("RegionX Cocos Local")
	.with_id("cocos_local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		id.into(),
	))
	.with_protocol_id("cocos-local")
	.with_properties(properties)
	.build()
}

fn cocos_genesis(id: ParaId) -> serde_json::Value {
	serde_json::json!({
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"candidacyBond": 500 * COCOS,
			"invulnerables": [
				"5F76zmFCUHBaSFoCecC54VCiZMpPWnRy4oPBrZpeLiuYRVCn"
			]
		},
		"assetRegistry": {
			"lastAssetId": RELAY_CHAIN_ASSET_ID,
			"assets": vec![(RELAY_CHAIN_ASSET_ID,
				AssetMetadata::<Balance, (), AssetsStringLimit>::encode(&AssetMetadata{
					decimals: 12,
					name: b"ROC".to_vec().try_into().expect("Invalid asset name"),
					symbol: b"ROC".to_vec().try_into().expect("Invalid asset symbol"),
					existential_deposit: ROC_EXISTENTIAL_DEPOSIT,
					location: Some(MultiLocation::parent().into()),
					additional: Default::default(),
				})
			)]
		},
		"sudo": {
			// The RegionX account:
			//
			// It should be needless to say that sudo is only available on the testnet.
			"key": "5DnKDEGGGo67szBc6tA42HMv7Q2vHe9y9xxNbaXknUcDPJcL"
		},
		"balances": {
		  "balances": [
			[
			  "5GL8eBMvz9LwbXciBufpEFPpecPbWM62gSH3hd9aAb1jrpo1",
			  1_000_000 * COCOS
			],
			[
			  "5DnKDEGGGo67szBc6tA42HMv7Q2vHe9y9xxNbaXknUcDPJcL",
			  1_000_000 * COCOS
			],
			[
			  "5DADsnBXr5DXiEAjdJvruf6c7ZSUR8iXUTATQqJfheGLiEVm",
			  500_000 * COCOS
			],
			[
			  "5F76zmFCUHBaSFoCecC54VCiZMpPWnRy4oPBrZpeLiuYRVCn",
			  500_000 * COCOS
			],
			[
			  "5DqwiiztGYq9P4jxG7RJJQWz4dMYREtbMrMSv62TwRAqVa1v",
			  500_000 * COCOS
			],
			[
			  "5FbSPQrNexY6dZ2JEVRguyqtrZbpD7qzRttXbiWKneagfEqr",
			  500_000 * COCOS
			],
			[
			  "5HGe3UghLKPuYPAkzFWwfARkGmE5mJ4sjs5SF8xfvtiB7o4v",
			  10_000 * COCOS
			],
		  ]
		},
		"technicalCommittee": {
			"members": [
				"5DnKDEGGGo67szBc6tA42HMv7Q2vHe9y9xxNbaXknUcDPJcL",
				"5DADsnBXr5DXiEAjdJvruf6c7ZSUR8iXUTATQqJfheGLiEVm",
				"5DqwiiztGYq9P4jxG7RJJQWz4dMYREtbMrMSv62TwRAqVa1v",
				"5FbSPQrNexY6dZ2JEVRguyqtrZbpD7qzRttXbiWKneagfEqr",
			]
		},
		"session": {
		  "keys": [
			[
			  "5F76zmFCUHBaSFoCecC54VCiZMpPWnRy4oPBrZpeLiuYRVCn",
			  "5F76zmFCUHBaSFoCecC54VCiZMpPWnRy4oPBrZpeLiuYRVCn",
			  {
				"aura": "5F76zmFCUHBaSFoCecC54VCiZMpPWnRy4oPBrZpeLiuYRVCn"
			  }
			]
		  ]
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
	})
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": COCOS_EXISTENTIAL_DEPOSIT * 16,
		},
		"assetRegistry": {
			"lastAssetId": RELAY_CHAIN_ASSET_ID,
			"assets": vec![(RELAY_CHAIN_ASSET_ID,
				AssetMetadata::<Balance, (), AssetsStringLimit>::encode(&AssetMetadata{
					decimals: 12,
					name: b"ROC".to_vec().try_into().expect("Invalid asset name"),
					symbol: b"ROC".to_vec().try_into().expect("Invalid asset symbol"),
					existential_deposit: ROC_EXISTENTIAL_DEPOSIT,
					location: Some(MultiLocation::parent().into()),
					additional: Default::default(),
				})
			)]
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                // account id
						acc,                        // validator id
						session_keys(aura), 		// session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) }
	})
}
