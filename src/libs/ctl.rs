use crate::libs::backends::winit::init_winit;
use anyhow::Context;
use log::{
	error,
	info,
};
use std::{
	fs::{
		metadata,
		remove_file,
	},
	io::Read,
	os::unix::net::{
		UnixListener,
		UnixStream,
	},
};

pub fn ctl() -> anyhow::Result<()> {
	let socket_path: &str = "/tmp/strata_socket";

	if metadata(socket_path).is_ok() {
		info!("A socket is already present. Deleting it ...");
		remove_file(socket_path)
			.with_context(|| format!("Could not delete previous socket at {:?}", socket_path))?;
	}

	let unix_listener =
		UnixListener::bind(socket_path).context("Could not create the unix socket")?;

	loop {
		let (unix_stream, _socket_address) = unix_listener
			.accept()
			.context("Failed at accepting a connection on the unix listener")?;
		handle_stream(unix_stream)?;
	}
}

fn handle_stream(mut unix_stream: UnixStream) -> anyhow::Result<()> {
	info!("Connection established to Strata CTL!");
	let mut command = String::new();
	unix_stream.read_to_string(&mut command).context("Failed at reading the unix stream")?;

	match &command.as_str() {
		&"launch winit" => {
			info!("Received to signal to launch using Winit backend. Launching...");
			let _ = init_winit();
		}
		&"launch udev" => {
			println!("TTY-Udev is not implement yet");
			info!("TTY-Udev is not implement yet");
		}
		&_ => {}
	}

	Ok(())
}
