
#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
use crate::window::win32::{LiquidWindowWin32,WinProcData};
use crate::liquid_engine::{LiquidEngine, Event};
use crate::io::LiquidError;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use log::{warn, error, info, trace, LevelFilter};

pub struct CreateWindowInfo {
	pub title : String,
	pub width : u32,
	pub height : u32,
	pub close_exits_program : bool,
	pub parent_window : Option<uuid::Uuid>
}
#[derive(Debug)]
pub enum LiquidWindow {
	#[cfg(target_os = "windows")]
	Win32(LiquidWindowWin32),
	#[cfg(target_os = "macos")]
	Macos(LiquidWindowMacos),
}

pub fn message_pump() {
	#[cfg(target_os = "windows")]
	{
		LiquidWindowWin32::message_pump();
	} 
}

impl LiquidEngine
{
	pub fn create_window(&mut self, create_window_info : &CreateWindowInfo) -> Result<uuid::Uuid, LiquidError> {
		let uuid = uuid::Uuid::new_v4();
		#[cfg(target_os = "windows")]
		{
			self.windows.insert(*uuid.as_bytes(), LiquidWindow::Win32(LiquidWindowWin32::new(uuid, create_window_info, &self.event_queue as *const Arc<Mutex<Vec<Event>>>)?));
		}  
		#[cfg(target_os = "macos")]
		{
			panic!("macos not supported yet");
		}
		info!("Created window of ID: {}", uuid);
		Ok(uuid)
	}

	pub fn destroy_window(&mut self, uuid : &uuid::Uuid) -> Result<bool, LiquidError> {
		LiquidEngine::destroy_window_raw(self, uuid.as_bytes())
	}

	pub fn destroy_window_raw(&mut self, uuid : &[u8; 16]) -> Result<bool, LiquidError> {
		match self.windows.remove(uuid) {
		    Some(e) => { 
				drop(e); 
				info!("Destroyed window of ID: {}", uuid::Uuid::from_bytes_ref(uuid));
				Ok(true)
			},
		    None => return Err(crate::newError!("Unable to find and destroy window of uuid {:?}", uuid)),
		}
	}

	pub fn get_liquid_window(&mut self, uuid : &uuid::Uuid) -> Result<&mut LiquidWindow, LiquidError> {
		LiquidEngine::get_liquid_window_raw(self, uuid.as_bytes())
	}

	pub fn get_liquid_window_raw(&mut self, uuid : &[u8; 16]) -> Result<&mut LiquidWindow, LiquidError> {
		return match self.windows.get_mut(uuid) {
			Some(res) => Ok(res),
			None => Err(crate::newError!("Unable to find and return window of uuid {:?}", uuid)),
		}
	}
}