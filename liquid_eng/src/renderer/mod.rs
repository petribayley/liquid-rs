mod vulkan;
mod opengl;

use log::{warn, error, info, trace, LevelFilter};
use crate::{
	liquid_engine::LiquidEngine,
	io, 
	io::LiquidError,
	renderer::vulkan::LiquidVulkanRenderer,
	renderer::opengl::LiquidOpenGLRenderer,
};

#[derive(Clone, Copy, Debug)]
pub enum LiquidRendererTypes {
	Vulkan,
	OpenGL,
}

pub struct LiquidCreateRendererInfo {
	pub application_name : &'static str,
	pub version_variant : u32,
	pub version_major : u32,
	pub version_minor : u32,
	pub version_patch : u32,
	pub renderer_type : LiquidRendererTypes
}

pub struct LiquidAttachRendererToWindowInfo {
	pub window_uuid : uuid::Uuid,
}

#[derive(Debug)]
pub struct LiquidRendererDevices{
	pub name : String,
}

#[derive(Debug)]
pub enum LiquidRenderer {
	Vulkan(LiquidVulkanRenderer),
	OpenGL(LiquidOpenGLRenderer),
}

impl Drop for LiquidRenderer {
	fn drop(self: &mut LiquidRenderer) {
		match self {
			LiquidRenderer::Vulkan(renderer) => drop(renderer),
    		LiquidRenderer::OpenGL(renderer) => drop(renderer),
		}
		info!("Destroying renderer");
	}
}

impl LiquidRenderer
{
	pub fn new(create_renderer_info : &LiquidCreateRendererInfo) -> Result<LiquidRenderer, LiquidError> {
		info!("Creating renderer of type {:?}", create_renderer_info.renderer_type);
		match create_renderer_info.renderer_type {
	    	LiquidRendererTypes::Vulkan => Ok(LiquidRenderer::Vulkan(LiquidVulkanRenderer::new(create_renderer_info)?)),
	    	LiquidRendererTypes::OpenGL => Ok(LiquidRenderer::OpenGL(LiquidOpenGLRenderer::new(create_renderer_info)?)),
		}
	}
}

impl LiquidEngine
{
	pub fn attach_renderer_to_window(&mut self, attach_renderer_to_window_info : &LiquidAttachRendererToWindowInfo) -> Result<bool, LiquidError> {
		info!("Attaching renderer to window");
		match &self.renderer {
			LiquidRenderer::Vulkan(renderer) => {
				//let uuid = attach_renderer_to_window_info.window_uuid.clone();
				//let window = self.get_liquid_window(&uuid)?;
				//renderer.attach_renderer_to_window(window);
			},
			LiquidRenderer::OpenGL(_renderer) => {
				todo!("Opengl not yet supported");
			},
		}

		Ok(true)
	}
}