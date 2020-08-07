// Copyright 2020 Parity Technologies (UK) Ltd.

//! Cumulus test parachain collator

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
	command::run()
}
