mod win32;

use crate::window::win32::{LiquidWindowWin32,WinProcData};
use crate::liquid_engine::{LiquidEngine, Event};
use crate::io::LiquidError;
use uuid::Uuid;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::core::{PCWSTR, HSTRING};
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
	Win32(LiquidWindowWin32),
}

impl Drop for LiquidWindow {
	fn drop(self: &mut LiquidWindow) {
		match self {
			LiquidWindow::Win32(window) => drop(window),
		}
	}
}

pub fn message_pump() {
	if cfg!(windows) {
		LiquidWindowWin32::message_pump();
	} else if cfg!(unix) {
	}
}

impl LiquidEngine
{
	pub fn create_window(&mut self, create_window_info : &CreateWindowInfo) -> Result<uuid::Uuid, LiquidError> {
		let uuid = uuid::Uuid::new_v4();
		if cfg!(windows) {
			self.windows.insert(*uuid.as_bytes(), LiquidWindow::Win32(LiquidWindowWin32::new(uuid, create_window_info, &self.event_queue as *const Arc<Mutex<Vec<Event>>>)?));
		} else if cfg!(unix) {
			panic!("Unix not supported yet");
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