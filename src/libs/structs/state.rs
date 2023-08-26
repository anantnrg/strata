use crate::libs::structs::workspaces::Workspaces;
use once_cell::sync::OnceCell;
use smithay::{
	backend::renderer::gles::GlesPixelProgram,
	desktop::PopupManager,
	input::{
		Seat,
		SeatState,
	},
	reexports::{
		calloop::{
			LoopHandle,
			LoopSignal,
		},
		wayland_server::{
			Display,
			DisplayHandle,
		},
	},
	utils::{
		Logical,
		Point,
	},
	wayland::{
		compositor::CompositorState,
		data_device::DataDeviceState,
		output::OutputManagerState,
		primary_selection::PrimarySelectionState,
		shell::{
			wlr_layer::WlrLayerShellState,
			xdg::{
				decoration::XdgDecorationState,
				XdgShellState,
			},
		},
		shm::ShmState,
	},
};
use std::{
	ffi::OsString,
	sync::Mutex,
	time::Instant,
};

pub struct CalloopData<BackendData: Backend + 'static> {
	pub state: StrataState<BackendData>,
	pub display: Display<StrataState<BackendData>>,
}

pub trait Backend {
	fn seat_name(&self) -> String;
}

pub struct StrataState<BackendData: Backend + 'static> {
	pub dh: DisplayHandle,
	pub backend_data: BackendData,
	pub start_time: Instant,
	pub loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
	pub loop_signal: LoopSignal,
	pub compositor_state: CompositorState,
	pub xdg_shell_state: XdgShellState,
	pub xdg_decoration_state: XdgDecorationState,
	pub shm_state: ShmState,
	pub output_manager_state: OutputManagerState,
	pub data_device_state: DataDeviceState,
	pub primary_selection_state: PrimarySelectionState,
	pub seat_state: SeatState<StrataState<BackendData>>,
	pub layer_shell_state: WlrLayerShellState,
	pub popup_manager: PopupManager,
	pub seat: Seat<Self>,
	pub seat_name: String,
	pub socket_name: OsString,
	pub workspaces: Workspaces,
	pub pointer_location: Point<f64, Logical>,
}

pub struct GlobalState<BackendData: Backend + 'static> {
	inner: OnceCell<Mutex<StrataState<BackendData>>>,
}

impl<BackendData: Backend + 'static> GlobalState<BackendData> {
	pub fn new() -> Self {
		Self { inner: OnceCell::new() }
	}

	pub fn set(&self, state: StrataState<BackendData>) -> Result<(), String> {
		match self.inner.set(Mutex::new(state)) {
			Ok(_) => Ok(()),
			Err(_) => Err("Failed to set StrataState in GlobalState".to_string()),
		}
	}

	pub fn get(&self) -> std::sync::MutexGuard<'_, StrataState<BackendData>> {
		self.inner.get().expect("State not initialized").lock().expect("Failed to lock state")
	}

	pub fn get_mut(&self) -> std::sync::MutexGuard<'_, StrataState<BackendData>> {
		self.inner.get().expect("State not initialized").lock().expect("Failed to lock state")
	}
}

pub struct BorderShader {
	pub rounded: GlesPixelProgram,
	pub default: GlesPixelProgram,
}
