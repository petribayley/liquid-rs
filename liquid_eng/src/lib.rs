#![cfg_attr(debug_assertions, allow(unused_imports))]
pub mod renderer;
pub mod io;

pub mod liquid_engine{
	use std::{
		thread,
		thread::Thread,
		cell::RefCell,
		sync::{
			Arc,
			Mutex,
			RwLock
		},
	};
	use log::{error, info, trace, warn};
	use log4rs;
	use serde_yaml;
	use uuid::Uuid;
	use crate::{
		io, 
		io::LiquidError,
		renderer,
		renderer::{LiquidRenderer, LiquidRendererDevices, LiquidCreateRendererInfo, LiquidRendererTypes},
		window::{message_pump, LiquidWindow}, newError
	};
	use once_cell::sync::OnceCell;

	use winit::{
		event::{Event, WindowEvent},
		event_loop::EventLoop,
		window::WindowBuilder,
	};

	pub struct LiquidCreateEngineInfo{
		pub create_renderer_info : LiquidCreateRendererInfo,
	}

	pub static LIQ_FLAGS:		OnceCell<RwLock<[u8; 1]>> = OnceCell::new();
	pub static LIQ_WINDOWS:		OnceCell<RwLock<std::collections::HashMap<[u8; 16], LiquidWindow>>> = OnceCell::new();
	pub static LIQ_RENDERER:	OnceCell<RwLock<LiquidRenderer>> = OnceCell::new();

	/// Constructor for engine returning default values
	pub fn new(create_liquid_engine_info : &LiquidCreateEngineInfo) -> Result<bool, LiquidError>
	{
		let config_str = include_str!("log_config.yml");
		let config = serde_yaml::from_str(config_str).unwrap();
		log4rs::init_raw_config(config).unwrap();

		LIQ_WINDOWS.set(RwLock::new(
			std::collections::HashMap::new()
		));

		LIQ_RENDERER.set(RwLock::new(
			LiquidRenderer::new(&create_liquid_engine_info.create_renderer_info)?
		));
		Ok(true)
	}

	/// Loop function for running 
	pub fn run()
	{
		// Set runing flag to true
		let event_loop = EventLoop::new();
		LIQ_FLAGS.get().unwrap().write().unwrap()[0] = 1;

		// Start thread for event handling, physics and game logic. The branching off from the main thread for game logic is a safety mechanism to ensure that window exiting will 
		// still function if the game logic hangs.
		let thread_handle = thread::spawn(move | | {
		});
		
		// Run the OS message loop
		event_loop.run(move |event, _event_loop, control_flow| {
			match event {
			Event::NewEvents(newEvent) => {

			},
			Event::WindowEvent { window_id, event } => {
				match event {
					WindowEvent::Resized(_) => {

					},
					WindowEvent::Moved(_) => {
						
					},
					WindowEvent::CloseRequested => {
						
					},
					WindowEvent::Destroyed => {
						
					},
					WindowEvent::DroppedFile(_) => {
						
					},
					WindowEvent::HoveredFile(_) => {
						
					},
					WindowEvent::HoveredFileCancelled => {
						
					},
					WindowEvent::ReceivedCharacter(_) => {
						
					},
					WindowEvent::Focused(_) => {
						
					},
					WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
						
					},
					WindowEvent::ModifiersChanged(_) => {
						
					},
					WindowEvent::Ime(_) => {
						
					},
					WindowEvent::CursorMoved { device_id, position, modifiers } => {
						
					},
					WindowEvent::CursorEntered { device_id } => {
						
					},
					WindowEvent::CursorLeft { device_id } => {
						
					},
					WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {
						
					},
					WindowEvent::MouseInput { device_id, state, button, modifiers } => {
						
					},
					WindowEvent::TouchpadPressure { device_id, pressure, stage } => {
						
					},
					WindowEvent::AxisMotion { device_id, axis, value } => {
						
					},
					WindowEvent::Touch(_) => todo!(),
					WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
						
					},
					WindowEvent::ThemeChanged(_) => {
						
					},
					WindowEvent::Occluded(_) => {
						
					},
				}
			},
			Event::DeviceEvent { device_id, event } => {

			},
			Event::UserEvent(userEvent) => {

			},
			Event::Suspended => {

			},
			Event::Resumed => {

			},
			Event::MainEventsCleared => {
				
			},
			Event::RedrawRequested(_) => {
				
			},
			Event::RedrawEventsCleared => {
				
			},
			Event::LoopDestroyed => {
				
			},
		}
		});
		// Join back to the thread after the message loop as ended
		let _ = thread_handle.join();
	}
}