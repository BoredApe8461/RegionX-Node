use crate::Balance;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Asset identifier.
pub type AssetId = u32;

#[derive(
	Clone,
	Copy,
	Default,
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct CustomMetadata {
	pub existential_deposit: Balance,
}

pub const REGX_ASSET_ID: AssetId = 0;
pub const RELAY_CHAIN_ASSET_ID: AssetId = 1;
