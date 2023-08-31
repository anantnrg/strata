use smithay::backend::{
	renderer::{
		damage::OutputDamageTracker,
		glow::GlowRenderer,
	},
	winit::WinitGraphicsBackend,
};

pub struct WinitData {
	pub backend: WinitGraphicsBackend<GlowRenderer>,
	pub damage_tracker: OutputDamageTracker,
}

impl WinitData {
	pub fn new(
		backend: WinitGraphicsBackend<GlowRenderer>,
		damage_tracker: OutputDamageTracker,
	) -> Self {
		Self { backend, damage_tracker }
	}
}
