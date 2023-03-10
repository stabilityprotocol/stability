#[macro_export]
macro_rules! map_err {
	($result:expr, $func:expr) => {
		match $result {
			Err(e) => return Err($func(e)),
			Ok(item) => item,
		}
	};
}

#[macro_export]
macro_rules! some_or_err {
	($result:expr, $func:expr) => {
		match $result {
			None => return Err($func()),
			Some(item) => item,
		}
	};
}

#[macro_export]
macro_rules! none_or_err {
	($result:expr, $func:expr) => {
		match $result {
			None => (),
			Some(e) => return Err($func(e)),
		}
	};
}