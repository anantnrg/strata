use crate::{
	libs::{
		decorations::CustomRenderElements,
		structs::{
			backends::WinitData,
			state::{
				Backend,
				BorderShader,
				CalloopData,
				StrataState,
			},
		},
	},
	CONFIG,
};
use smithay::{
	backend::{
		renderer::{
			damage::OutputDamageTracker,
			element::AsRenderElements,
			glow::GlowRenderer,
		},
		winit::{
			self,
			WinitError,
			WinitEvent,
			WinitEventLoop,
			WinitGraphicsBackend,
		},
	},
	desktop::{
		layer_map_for_output,
		space::SpaceElement,
		LayerSurface,
	},
	output::{
		Mode,
		Output,
		PhysicalProperties,
		Subpixel,
	},
	reexports::{
		calloop::{
			timer::{
				TimeoutAction,
				Timer,
			},
			EventLoop,
		},
		wayland_server::Display,
	},
	utils::{
		Rectangle,
		Scale,
		Transform,
	},
	wayland::shell::wlr_layer::Layer,
};
use std::{
	cell::RefMut,
	process::Command,
	time::Duration,
};

impl Backend for WinitData {
	fn seat_name(&self) -> String {
		"winit".to_string()
	}
	fn get_backend(&mut self) -> RefMut<WinitGraphicsBackend<GlowRenderer>> {
		self.backend.borrow_mut()
	}
	fn get_damage_tracker(&mut self) -> RefMut<OutputDamageTracker> {
		self.damage_tracker.borrow_mut()
	}
}

pub fn init_winit() {
	let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new().unwrap();
	let mut display: Display<StrataState> = Display::new().unwrap();
	let (backend, mut winit) = winit::init().unwrap();
	let mode = Mode { size: backend.window_size().physical_size, refresh: 60_000 };
	let output = Output::new(
		"winit".to_string(),
		PhysicalProperties {
			size: (0, 0).into(),
			subpixel: Subpixel::Unknown,
			make: "Strata".into(),
			model: "Winit".into(),
		},
	);
	let _global = output.create_global::<StrataState>(&display.handle());
	output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
	output.set_preferred(mode);
	let damage_tracked_renderer = OutputDamageTracker::from_output(&output);
	let winitdata = WinitData::new(backend, damage_tracked_renderer);
	let state =
		StrataState::new(event_loop.handle(), event_loop.get_signal(), &mut display, winitdata);

	let mut data = CalloopData { display, state };
	let state = &mut data.state;
	BorderShader::init(state.backend_data.get_backend().renderer());
	for workspace in state.workspaces.iter() {
		workspace.add_output(output.clone());
	}

	std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);
	let mut full_redraw = 0u8;
	let timer = Timer::immediate();

	event_loop
		.handle()
		.insert_source(timer, move |_, _, data| {
			winit_dispatch(&mut winit, data, &output, &mut full_redraw);
			TimeoutAction::ToDuration(Duration::from_millis(16))
		})
		.unwrap();

	for cmd in &CONFIG.read().options.autostart {
		Command::new("/bin/sh").arg("-c").args(cmd).spawn().ok();
	}

	event_loop.run(None, &mut data, move |_| {}).unwrap();
}

pub fn winit_dispatch(
	winit: &mut WinitEventLoop,
	data: &mut CalloopData,
	output: &Output,
	full_redraw: &mut u8,
) {
	let display = &mut data.display;
	let state = &mut data.state;

	let res = winit.dispatch_new_events(|event| {
		match event {
			WinitEvent::Resized { size, .. } => {
				output.change_current_state(Some(Mode { size, refresh: 60_000 }), None, None, None);
			}
			WinitEvent::Input(event) => state.process_input_event(event),
			_ => (),
		}
	});

	let mut winitdata = &mut state.backend_data;
	let mut backend = winitdata.get_backend();
	let mut damage_tracker = winitdata.get_damage_tracker();

	if let Err(WinitError::WindowClosed) = res {
		state.loop_signal.stop();
	} else {
		res.unwrap();
	}

	*full_redraw = full_redraw.saturating_sub(1);

	let size = backend.window_size().physical_size;
	let damage = Rectangle::from_loc_and_size((0, 0), size);

	backend.bind().unwrap();

	let mut renderelements: Vec<CustomRenderElements<_>> = vec![];
	let workspace = state.workspaces.current_mut();
	let output = workspace.outputs().next().unwrap();
	let layer_map = layer_map_for_output(output);
	let (lower, upper): (Vec<&LayerSurface>, Vec<&LayerSurface>) = layer_map
		.layers()
		.rev()
		.partition(|s| matches!(s.layer(), Layer::Background | Layer::Bottom));

	renderelements.extend(
		upper
			.into_iter()
			.filter_map(|surface| layer_map.layer_geometry(surface).map(|geo| (geo.loc, surface)))
			.flat_map(|(loc, surface)| {
				AsRenderElements::<GlowRenderer>::render_elements::<CustomRenderElements<_>>(
					surface,
					backend.renderer(),
					loc.to_physical_precise_round(1),
					Scale::from(1.0),
					1.0,
				)
			}),
	);

	renderelements.extend(workspace.render_elements(backend.renderer()));

	renderelements.extend(
		lower
			.into_iter()
			.filter_map(|surface| layer_map.layer_geometry(surface).map(|geo| (geo.loc, surface)))
			.flat_map(|(loc, surface)| {
				AsRenderElements::<GlowRenderer>::render_elements::<CustomRenderElements<_>>(
					surface,
					backend.renderer(),
					loc.to_physical_precise_round(1),
					Scale::from(1.0),
					1.0,
				)
			}),
	);

	damage_tracker
		.render_output(backend.renderer(), 0, &renderelements, [0.1, 0.1, 0.1, 1.0])
		.unwrap();

	backend.submit(Some(&[damage])).unwrap();

	workspace.windows().for_each(|window| {
		window.send_frame(output, state.start_time.elapsed(), Some(Duration::ZERO), |_, _| {
			Some(output.clone())
		})
	});

	workspace.windows().for_each(|e| e.refresh());
	display.flush_clients().unwrap();
	state.popup_manager.cleanup();
	BorderShader::cleanup(backend.renderer());
}
