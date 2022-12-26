use crate::{io, 
	io::LiquidError,
	renderer::{LiquidRendererDevices, LiquidCreateRendererInfo}
};

use glow::*;

/// Macos only supports OpenGL 4.1
#[derive(Debug)]
pub struct LiquidOpenGLRenderer {
	// Custom Attributes
	pub screen_width : u32,
	pub screen_height : u32,
	pub renderer_devices : Vec<LiquidRendererDevices>,
}

impl Drop for LiquidOpenGLRenderer {
	fn drop(&mut self) {
	}
}

impl LiquidOpenGLRenderer {
	pub fn new(_create_renderer_info : &LiquidCreateRendererInfo) -> Result<Self, LiquidError> {
		/*
		let window = glutin::context::ContextAttributesBuilder::new().build();

		unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s)) };
		glow::
		*/
		Ok(LiquidOpenGLRenderer{
			screen_width: 1920,
			screen_height: 1080,
			renderer_devices: Vec::new()
		})
	}
}