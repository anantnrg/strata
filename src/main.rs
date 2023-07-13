mod libs;
use clap::Parser;
pub use libs::{
	backends::winit::init_winit,
	ctl::ctl,
	parse_config::parse_config,
	structs::{
		CalloopData,
		Strata,
	},
};

use anyhow::Context;
use std::{
	error::Error,
	process::Command,
};

fn main() -> Result<(), Box<dyn Error>> {
	if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
		tracing_subscriber::fmt().with_env_filter(env_filter).init();
	} else {
		tracing_subscriber::fmt().init();
	}

	let _ = parse_config();
	let _ = ctl();

	Ok(())
}
