#[derive(Debug)]
pub struct LiquidError {
	pub msg : String,
	pub file : &'static str,
	pub line : u32,
	pub column : u32
}

impl Default for LiquidError{
	fn default() -> Self { 
		Self{ msg : "".to_string(), file : "", line : 0, column : 0 }
	}
}

#[macro_export]
macro_rules! newError {
	($base:literal) => {
		LiquidError{ 
			msg : $base.to_string(), 
			file : file!(),
			line : line!(),
			column : column!()
		}
	};
	($base:literal, $($args:tt),*) => {
		LiquidError{ 
			msg : format!($base, $($args),*), 
			file : file!(),
			line : line!(),
			column : column!()
		}
	};
}

impl std::fmt::Display for LiquidError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Liquid Error caught, {{msg : {}, file : {}, ({}:{}}}", self.msg, self.file, self.line, self.column)
	}
}

