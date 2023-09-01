use smithay::backend::{
	renderer::{
		damage::OutputDamageTracker,
		glow::GlowRenderer,
	},
	winit::WinitGraphicsBackend,
};
use std::cell::RefCell;

pub struct WinitData {
	pub backend: RefCell<WinitGraphicsBackend<GlowRenderer>>,
	pub damage_tracker: RefCell<OutputDamageTracker>,
}

impl WinitData {
	pub fn new(
		backend: WinitGraphicsBackend<GlowRenderer>,
		damage_tracker: OutputDamageTracker,
	) -> Self {
		Self { backend: RefCell::new(backend), damage_tracker: RefCell::new(damage_tracker) }
	}
}
