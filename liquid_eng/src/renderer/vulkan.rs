use crate::{io, 
	io::LiquidError,
	window::LiquidWindow,
	renderer::{LiquidRendererDevices, LiquidCreateRendererInfo, LiquidAttachRendererToWindowInfo}, 
	newError
};
use erupt::vk1_0::{DeviceCreateInfo, DeviceCreateFlags, DeviceQueueCreateInfo};
use erupt::{
    vk::{self, Win32SurfaceCreateInfoKHR}, EntryLoader, InstanceLoader, DeviceLoader
};
use core::ffi::c_void;
use std::{
	ffi::{CString, CStr},
	sync::{
		Arc,
		Mutex
	}
};

use log::{warn, error, info, trace, LevelFilter};

#[derive(Debug)]
pub struct LiquidVulkanRenderer {
	// Custom Attributes
	pub screen_width : u32,
	pub screen_height : u32,
	pub renderer_devices : Vec<LiquidRendererDevices>,

	// Vulkan Objects, should stack from the bottom in lifetime
	pub vk_device_loader : std::sync::Arc<std::sync::Mutex<DeviceLoader>>,
	#[cfg(debug_assertions)]
	pub vk_debug_messenger : vk::DebugUtilsMessengerEXT,
	pub vk_instance : Arc<Mutex<InstanceLoader>>,
	pub vk_entry : Arc<Mutex<EntryLoader>>,
}

impl Drop for LiquidVulkanRenderer {
	fn drop(&mut self) {
		match self.vk_device_loader.lock() {
			Ok(device) => unsafe { device.destroy_device(None) },
			Err(err) => error!("Unable to get mutex lock on vk_device {}", err),
		}
		// Instance and Debugger
		#[cfg(debug_assertions)]
		match self.vk_instance.lock() {
			Ok(instance) => unsafe { 
				#[cfg(debug_assertions)]
				instance.destroy_debug_utils_messenger_ext(self.vk_debug_messenger, None);
				instance.destroy_instance(None) 
			},
			Err(err) => error!("Unable to get mutex lock on vk_instance {}", err),
		}
	}
}

impl LiquidVulkanRenderer {
	pub fn new(create_renderer_info : &LiquidCreateRendererInfo) -> Result<Self, LiquidError> {
		let vk_entry = match EntryLoader::new() {
			Ok(entry) => std::sync::Arc::new( Mutex::new( entry ) ),
			Err(err) => return Err(crate::newError!("Unable to load vulkan library {}", err))
		};

		let application_create_info = vk::ApplicationInfo { 
			s_type: vk::StructureType::APPLICATION_INFO, 
			p_next: std::ptr::null(), 
			p_application_name: create_renderer_info.application_name.as_ptr() as *const i8, 
			application_version: vk::make_api_version(create_renderer_info.version_variant, create_renderer_info.version_major, create_renderer_info.version_minor, create_renderer_info.version_patch), 
			p_engine_name: env!("CARGO_PKG_NAME").as_ptr() as *const i8, 
			engine_version: vk::make_api_version(
				0,
				env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap(), 
				env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap(), 
				env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap()), 
			api_version: vk::API_VERSION_1_3 
		};

		let mut enabled_instance_layers: Vec<std::ffi::CString> = Vec::new();
		let mut enabled_instance_extensions: Vec<std::ffi::CString> = Vec::new();
		let mut enabled_device_extensions: Vec<std::ffi::CString> = Vec::new();
		
		enabled_instance_layers.push(std::ffi::CString::new("VK_LAYER_KHRONOS_synchronization2").unwrap());
		enabled_instance_extensions.push(std::ffi::CString::new("VK_KHR_surface").unwrap());
		
		enabled_device_extensions.push(std::ffi::CString::new("VK_KHR_swapchain").unwrap());

		#[cfg(debug_assertions)]
		{
			enabled_instance_layers.push(std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap());
			enabled_instance_layers.push(std::ffi::CString::new("VK_LAYER_LUNARG_monitor").unwrap());
			enabled_instance_extensions.push(std::ffi::CString::new("VK_EXT_debug_utils").unwrap());
		}

		#[cfg(target_os = "windows")]
		{
			enabled_instance_extensions.push(std::ffi::CString::new("VK_KHR_win32_surface").unwrap());
		}

		#[cfg(target_os = "macos")]
		{
			enabled_instance_extensions.push(std::ffi::CString::new("VK_EXT_metal_surface").unwrap());
		}

		#[cfg(target_os = "linux")]
		{
			enabled_instance_extensions.push(std::ffi::CString::new("VK_KHR_xcb_surface").unwrap());
		}

		let p_enabled_instance_layers: Vec<*const i8> = enabled_instance_layers.iter().map(|layer_name| layer_name.as_ptr()).collect();
		let p_enabled_instance_extensions: Vec<*const i8> = enabled_instance_extensions.iter().map(|extensions_name| extensions_name.as_ptr()).collect();

		let instance_create_info = vk::InstanceCreateInfo{
		    s_type: vk::StructureType::INSTANCE_CREATE_INFO,
		    p_next: std::ptr::null(),
		    flags: vk::InstanceCreateFlags::empty(),
		    p_application_info: &application_create_info,
		    enabled_layer_count: enabled_instance_layers.len() as u32,
		    pp_enabled_layer_names: p_enabled_instance_layers.as_ptr(),
		    enabled_extension_count: enabled_instance_extensions.len() as u32,
		    pp_enabled_extension_names: p_enabled_instance_extensions.as_ptr(),
		};

		let vk_instance = match unsafe{ InstanceLoader::new(&vk_entry.lock().unwrap(), &instance_create_info) } {
			Ok(instance) => std::sync::Arc::new( Mutex::new( instance ) ),
			Err(e) => {
				return Err(crate::newError!("Error while calling create_instance: {}", e))
			}
		};

		let debug_info = vk::DebugUtilsMessengerCreateInfoEXT{
				message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO_EXT,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT,
				//pfn_user_callback: Some(debug_full_callback),
				pfn_user_callback: Some(debug_callback),
				..Default::default()
			};

		#[cfg(debug_assertions)]
		let vk_debug_messenger  = unsafe { vk_instance.lock().unwrap().create_debug_utils_messenger_ext(&debug_info, None).unwrap() };

		let physical_devices = match unsafe { vk_instance.lock().unwrap().enumerate_physical_devices(None) }.value {
		    Some(physical_devices) => physical_devices,
		    None => return Err(crate::newError!("Unable to find any graphical devices")),
		};

		let mut devices : Vec<LiquidRendererDevices> = Vec::new();
		let chosen_device = physical_devices.get(0).unwrap();
		for device in &physical_devices {
			let mut features : erupt::vk1_1::PhysicalDeviceFeatures2 = erupt::vk1_1::PhysicalDeviceFeatures2::default();
			let mut properties : erupt::vk1_1::PhysicalDeviceProperties2 = erupt::vk1_1::PhysicalDeviceProperties2::default();
			unsafe { vk_instance.lock().unwrap().get_physical_device_features2(*device, &mut features) };
			unsafe { vk_instance.lock().unwrap().get_physical_device_properties2(*device, &mut properties) };

        	let tmp = unsafe { CStr::from_ptr(&properties.properties.device_name as *const i8) };
        	let dev_name = tmp.to_str().unwrap();

			info!("{:?} Device found {} {}:{}", 
				properties.properties.device_type, 
				dev_name, 
				properties.properties.vendor_id, 
				properties.properties.device_id
			);
			devices.push(LiquidRendererDevices {
				name: dev_name.to_string()
			});
			break;
		};
		
		unsafe {vk_instance.lock().unwrap().get_physical_device_queue_family_properties(*chosen_device, None)};
		
		let priority_list: [f32; 1] = [1.0];
		let queue_list: [DeviceQueueCreateInfo; 2] = 
		[
		DeviceQueueCreateInfo{
			queue_family_index: 0,
			queue_count: 1,
			p_queue_priorities: &priority_list as *const f32,
			..Default::default()
		},
		DeviceQueueCreateInfo{
			queue_family_index: 1,
			queue_count: 1,
			p_queue_priorities: &priority_list as *const f32,
			..Default::default()
		},
		];

		let p_enabled_device_extensions: Vec<*const i8> = enabled_device_extensions.iter().map(|extensions_name| extensions_name.as_ptr()).collect();
		let device_create_info = DeviceCreateInfo{
		    flags: DeviceCreateFlags::empty(),
		    queue_create_info_count: queue_list.len() as u32,
		    p_queue_create_infos: &queue_list as *const DeviceQueueCreateInfo,
		    enabled_layer_count: 0,
		    pp_enabled_layer_names: std::ptr::null(),
		    enabled_extension_count: p_enabled_device_extensions.len() as u32,
		    pp_enabled_extension_names: p_enabled_device_extensions.as_ptr(),
		    // p_enabled_features:,
			..Default::default()
		};
		let vk_device_loader = match unsafe { DeviceLoader::new(&vk_instance.lock().unwrap(), *chosen_device, &device_create_info ) }{
			Ok(device_loader) => Arc::new(Mutex::new(device_loader)),
			Err(err) => return Err(newError!("Unable to create device loader {}", err)),
		};

		Ok(LiquidVulkanRenderer{ 
			screen_width: 1080,
			screen_height: 720,
			vk_entry,
			vk_instance,
			vk_device_loader,
			#[cfg(debug_assertions)]
			vk_debug_messenger,
			renderer_devices : devices,
		})
	}

	pub fn get_devices(&self) -> &Vec<LiquidRendererDevices> {
    	&self.renderer_devices
	}

	/// Returns the screen width : u32 and height : u32
	pub fn get_render_screen_size(&self) -> (u32, u32) {
		(self.screen_width, self.screen_height)
	}

	pub fn attach_renderer_to_window(&self, _liquid_window : &LiquidWindow) {
		
	}
}

#[allow(dead_code)] // Allowing for optional use for extended callback information.
unsafe extern "system" fn debug_full_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
		let p_message_id_name = (*p_callback_data).p_message_id_name;
		let message_id_number = (*p_callback_data).message_id_number;
		let p_message = (*p_callback_data).p_message;
		let queue_label_count = (*p_callback_data).queue_label_count as isize;
		let p_queue_labels = (*p_callback_data).p_queue_labels;
		let cmd_buf_label_count = (*p_callback_data).cmd_buf_label_count as isize;
		let p_cmd_buf_labels  = (*p_callback_data).p_cmd_buf_labels;
        let p_objects = (*p_callback_data).p_objects;
        let object_count = (*p_callback_data).object_count as isize;

		info!("-----------------------------");
        info!(" Start Of Vulkan callback");
        info!("-----------------------------");

    	let mut id_name : &str = "";
    	if p_message_id_name != std::ptr::null(){
        	let tmp = CStr::from_ptr(p_message_id_name);
        	id_name = tmp.to_str().unwrap();
    	}
    	info!("[id:{}]\n[id name:0x{:X}]", id_name, message_id_number);
    	
    	if p_message != std::ptr::null(){
        	let tmp = CStr::from_ptr(p_message);
        	let mut message = tmp.to_str().unwrap().split("|").last().unwrap().chars();
        	message.next();
    		info!("{}", message.as_str());
    	}

        info!("-----------------------------");
        info!(" Active Queue(s)");
        info!("-----------------------------");
        for i in 0.. {
        	if i >= queue_label_count {
        		break;
        	}
        	let queue_label = *( p_queue_labels.offset(i) );
        	let mut label_name : &str = "";
        	if queue_label.p_label_name != std::ptr::null(){
	        	let label_name_tmp = CStr::from_ptr(queue_label.p_label_name);
	        	label_name = label_name_tmp.to_str().unwrap();
        	}
        	info!("Queue label: {}", label_name);
        }

        info!("-----------------------------");
        info!(" Active Command Buffer(s)");
        info!("-----------------------------");
        for i in 0isize.. {
        	if i >= cmd_buf_label_count {
        		break;
        	}
        	let cmd_buf_label = *( p_cmd_buf_labels.offset(i) );
        	let mut cmd_buffer_name : &str = "";
        	if cmd_buf_label.p_label_name != std::ptr::null(){
	        	let label_name_tmp = CStr::from_ptr(cmd_buf_label.p_label_name);
	        	cmd_buffer_name = label_name_tmp.to_str().unwrap();
        	}
        	info!("Queue label: {}", cmd_buffer_name);
        }

        info!("-----------------------------");
        info!(" Active Object(s)");
        info!("-----------------------------");
        for i in 0.. {
        	if i >= object_count {
        		break;
        	}
        	let object = *( p_objects.offset(i) );
        	let object_type_tmp = format!("{:?}", object.object_type);
        	let object_type : &str = object_type_tmp.as_str();
        	let object_handle = object.object_handle;
        	let mut object_name : &str = "";
        	if object.p_object_name != std::ptr::null(){
	        	let object_name_tmp = CStr::from_ptr(object.p_object_name);
	        	object_name = object_name_tmp.to_str().unwrap();
        	}
        	info!("Object type: {} Object name: {} Object handle: 0x{:X}", object_type, object_name, object_handle);
        }
        info!("-----------------------------");
        info!(" End Of Vulkan Callback");
        info!("-----------------------------\n\n");
    vk::FALSE
}

unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
	let p_message_id_name = (*p_callback_data).p_message_id_name;
	let message_id_number = (*p_callback_data).message_id_number;
	let p_message = (*p_callback_data).p_message;

	if message_id_number == 0 {
		return vk::FALSE;
	}

	let mut id_name : &str = "";
	if p_message_id_name != std::ptr::null(){
		let tmp = CStr::from_ptr(p_message_id_name);
		id_name = tmp.to_str().unwrap();
	}
	if p_message != std::ptr::null(){
		let tmp = CStr::from_ptr(p_message);
		let message = tmp.to_str().unwrap().split("|").last().unwrap();
		info!("[id:{}] [id name:0x{:X}] {}", id_name, message_id_number, message);
	}
    vk::FALSE
}