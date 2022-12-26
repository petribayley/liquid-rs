use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::{sync::{Arc,Mutex}, rc::Rc};

use crate::newError;
use crate::{
	liquid_engine::
	{LiquidEngine, Event},
	window::{CreateWindowInfo, LiquidWindow},
	io::LiquidError
};

use log::{warn, error, info, trace, LevelFilter};
use windows::{
	Win32::{
		Foundation::{
		LRESULT,
		HWND,
		HINSTANCE,
		LPARAM,
		WPARAM,
		},
		UI::WindowsAndMessaging::{
				SW_SHOW,
				MSG,
				WNDCLASSEXW,
				CS_HREDRAW,
				CS_VREDRAW,
				CS_DBLCLKS,
				HICON,
				IDC_ARROW,
				HCURSOR,
				WS_EX_OVERLAPPEDWINDOW,
				WS_OVERLAPPEDWINDOW,
				CW_USEDEFAULT,
				HMENU,
				GWL_USERDATA,
				DefWindowProcW,
				DestroyWindow,
				LoadCursorW,
				RegisterClassExW,
				SetWindowLongPtrW,
				ShowWindow,
				GetMessageW,
				TranslateMessage,
				DispatchMessageW,
				GetWindowLongPtrW,
				PostQuitMessage,
				CreateWindowExW
			},
		Graphics::Gdi::{
			HBRUSH,
			GetStockObject,
			WHITE_BRUSH
		},
	},
	core::{
		HSTRING,
		PCWSTR
	},
};

#[derive(Debug)]
pub struct LiquidWindowWin32 {
	pub window_handle : HWND,
	pub instance : HINSTANCE,
	pub proc_data : Box<WinProcData>,
}

#[derive(Debug)]
pub struct WinProcData {
	pub window_id : uuid::Uuid,
	pub close_exits_program : bool,

	// Data returned from loop
	ptr_event_queue : *const Arc<Mutex<Vec<Event>>>,
}

impl WinProcData {
	fn new(window_id : uuid::Uuid, close_exits_program : bool, ptr_event_queue : *const Arc<Mutex<Vec<Event>>>) -> Self {
		WinProcData { 
			window_id : window_id, 
			close_exits_program : close_exits_program,
			ptr_event_queue,
		}
	}
}

unsafe impl Sync for WinProcData {}

unsafe impl Send for WinProcData {}

impl LiquidWindowWin32
{
	pub fn new(uuid : uuid::Uuid, create_window_info : &CreateWindowInfo, ptr_event_queue : *const Arc<Mutex<Vec<Event>>> ) -> Result<Self, LiquidError> {

		let instance : HINSTANCE = unsafe { 
			windows::Win32::System::LibraryLoader::GetModuleHandleW(None).expect("Error occoured while calling `GetModuleHandleW`")
		};

		let class_name = uuid::Uuid::new_v4().to_string();
		let mut class_name_encoded = class_name.encode_utf16().chain([0u16]).collect::<Vec<u16>>();

		let wc = WNDCLASSEXW {
			cbSize : std::mem::size_of::< WNDCLASSEXW >() as u32,
			style : CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
			lpfnWndProc : Some(win_proc),
			cbClsExtra : 0,
			cbWndExtra : 0,
			hInstance : instance,
			hIcon : HICON(0),
			hCursor : unsafe {LoadCursorW( HINSTANCE(0), IDC_ARROW )}.expect("Error occoured while calling `LoadCursorW`") as HCURSOR,
			hbrBackground : HBRUSH(unsafe {GetStockObject( WHITE_BRUSH )}.0),
			lpszMenuName : PCWSTR::null(),
			lpszClassName : PCWSTR(class_name_encoded.as_mut_ptr()),
			hIconSm : HICON(0)
		};

		if unsafe { RegisterClassExW( &wc ) } == 0 {
			return Err(crate::newError!("Unable to register class: {}", uuid));
		}

		let window = unsafe {
			CreateWindowExW(
				WS_EX_OVERLAPPEDWINDOW,
				&HSTRING::from(class_name),
				&HSTRING::from(create_window_info.title.to_owned()), 
				WS_OVERLAPPEDWINDOW,
				CW_USEDEFAULT, 
				CW_USEDEFAULT, 
				create_window_info.width as i32, 
				create_window_info.height  as i32, 
				HWND(0), 
				HMENU(0), 
				instance,
				None
				)
		};

		let should_window_close_program = match create_window_info.parent_window {
			Some(_) => {
				warn!("[ LiqLog--Window : CreateWindow ] Window specified with exiting the application has a parent window identified. Omitting the close on exit feature for child window.");
				false
			},
			None => create_window_info.close_exits_program,
		};

		let mut liquid_window_obj = LiquidWindowWin32{ 
			window_handle : window, 
			instance,
			proc_data : Box::new(WinProcData::new(uuid, should_window_close_program, ptr_event_queue))
		};

		unsafe { SetWindowLongPtrW (window, GWL_USERDATA, liquid_window_obj.proc_data.borrow_mut() as *mut WinProcData as isize ); }
		unsafe { ShowWindow( window, SW_SHOW ); }

		Ok(liquid_window_obj)
	}

	pub fn message_pump() {
		let mut msg = MSG::default();
		loop {
			if unsafe { GetMessageW(&mut msg, None, 0, 0) } == false {
				break;
			}

			let _translate_ret = unsafe { TranslateMessage(&msg) };
			let _dispatch_ret = unsafe { DispatchMessageW(&msg) };
		}
	}
}

unsafe extern "system" fn win_proc(
	h_wnd: HWND, 
	u_msg: u32,
	w_param: WPARAM, 
	l_param: LPARAM
	) -> LRESULT {
	// Creation items
	match u_msg {
		// WM_GETMINMAXINFO
		0x0024 => return unsafe{ DefWindowProcW(h_wnd, u_msg, w_param, l_param) },
		// WM_NCCREATE
		0x0081 => return unsafe{ DefWindowProcW(h_wnd, u_msg, w_param, l_param) },
		// WM_NCCALCSIZE
		0x0083 => return unsafe{ DefWindowProcW(h_wnd, u_msg, w_param, l_param) },
		// WM_CREATE
		0x0001 => return unsafe{ DefWindowProcW(h_wnd, u_msg, w_param, l_param) },
		_ => {}
	}
	
	let win_proc_data_ptr = GetWindowLongPtrW (h_wnd, GWL_USERDATA) as *mut WinProcData;
	if win_proc_data_ptr.is_null() {
		error!("User ptr is null! : {}", u_msg);
		return LRESULT(0);
	}
	let win_proc_data = &mut *win_proc_data_ptr;
	let event_queue = &*win_proc_data.ptr_event_queue;
	match u_msg {
		// WM_CLOSE
	    0x0010 => {
			match event_queue.lock() {
				Ok(mut vec) => vec.push(Event::Close(win_proc_data.window_id)),
				Err(err) => error!("Error while accessing events vector, mutex has been poisoned! {}", err),
			};
			if win_proc_data.close_exits_program {
				match event_queue.lock() {
					Ok(mut vec) => vec.push(Event::Exit),
					Err(err) => error!("Error while accessing events vector, mutex has been poisoned! {}", err),
				};
				PostQuitMessage(0);
			}
			DestroyWindow(h_wnd);
	    }
		// WM_QUIT
		0x0012 => {
			match event_queue.lock() {
				Ok(mut vec) => vec.push(Event::Exit),
				Err(err) => error!("Error while accessing events vector, mutex has been poisoned! {}", err),
			}
		}
	    // Default return
	    _ => return unsafe{ DefWindowProcW(h_wnd, u_msg, w_param, l_param) }
	}
	return LRESULT(0);
}