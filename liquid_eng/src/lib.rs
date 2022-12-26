#![cfg_attr(debug_assertions, allow(unused_imports))]
pub mod window;
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

	#[derive(Debug, Clone, Copy)]
	pub enum Event {
		Close(Uuid),
		Exit
	}

	pub struct LiquidCreateEngineInfo{
		pub create_renderer_info : LiquidCreateRendererInfo,
	}

	#[derive(Debug)]
	pub struct LiquidEngine {
		pub is_running : bool,
		pub windows : std::collections::HashMap<[u8; 16], LiquidWindow>,
		pub renderer : LiquidRenderer,
		pub event_queue : Arc<Mutex<Vec<Event>>>
	}

	unsafe impl Sync for LiquidEngine {}

	pub static LIQ_FLAGS:		OnceCell<RwLock<std::collections::HashMap<[u8; 16], LiquidWindow>>> = OnceCell::new();
	pub static LIQ_WINDOWS:		OnceCell<RwLock<std::collections::HashMap<[u8; 16], LiquidWindow>>> = OnceCell::new();

	impl LiquidEngine
	{
		/// Constructor for engine returning default values
		pub fn new(create_liquid_engine_info : &LiquidCreateEngineInfo) -> Result<bool, LiquidError>
		{
			let config_str = include_str!("log_config.yml");
			let config = serde_yaml::from_str(config_str).unwrap();
			log4rs::init_raw_config(config).unwrap();

			LIQ_WINDOWS.set(RwLock::new(
				std::collections::HashMap::new()
			));


			INSTANCE.set(RefCell::new(LiquidEngine { 
				is_running: false,
				windows: std::collections::HashMap::new(),
				renderer: LiquidRenderer::new(&create_liquid_engine_info.create_renderer_info)?,
				event_queue : Arc::new(Mutex::new(Vec::new())),
			})).unwrap();
			Ok(true)
		}

		/// Loop function for running 
		pub fn run()
		{
			let engine = INSTANCE.get().unwrap();
			engine.lock().unwrap().is_running = true;

			// Start thread for event handling, physics and game logic. The branching off from the main thread for game logic is a safety mechanism to ensure that window exiting will 
			// still function if the game logic hangs.
			let thread_handle = thread::spawn(move | | {
				'outer : loop{
					let mut events : Vec<Event> = Vec::new();
					match self.event_queue.lock() {
						Ok(mut ret) => {
							events.clone_from(&ret);
							ret.clear();
						},
						Err(e) => warn!("Warning, mutex poisoned! {}", e),
					}
					for event in events {
						match event {
							Event::Close(uuid) => {
								self.destroy_window(&uuid).unwrap();
							},
							Event::Exit => {
								break 'outer;
							}
						}
					}
				}
			});

			// Run the OS message loop
			message_pump();
			// Join back to the thread after the message loop as ended
			let _ = thread_handle.join();
		}
	}
}