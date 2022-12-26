use liquid_eng::{
	liquid_engine::{LiquidCreateEngineInfo, LiquidEngine, INSTANCE},
	window::CreateWindowInfo,
	renderer::{LiquidRendererTypes, LiquidCreateRendererInfo}
	};

fn main() {
	let create_engine_info = LiquidCreateEngineInfo{
    	create_renderer_info: { LiquidCreateRendererInfo {
    		application_name: "Test Window",
		    version_variant: 0,
		    version_major: 1,
		    version_minor: 0,
		    version_patch: 0,
		    renderer_type: LiquidRendererTypes::Vulkan,
    	} },
	};

	LiquidEngine::new(&create_engine_info).unwrap();
	let engine = INSTANCE.get_mut().unwrap();

	let window_info = CreateWindowInfo{ 
		title: "Test Window".to_string(),
	    width: 1920,
	    height: 1080,
	    close_exits_program: true,
	    parent_window: None, 
	};

	engine.lock().unwrap().create_window(&window_info).unwrap();

	LiquidEngine::run(engine);
}