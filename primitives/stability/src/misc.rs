#[macro_export]
macro_rules! map_err {
	($result:expr, $err:expr) => {
		match $result {
			Err(_) => return Err($err),
			Ok(item) => item,
		}
	};
}

#[macro_export]
macro_rules! some_or_err {
	($result:expr, $err:expr) => {
		match $result {
			None => return Err($err),
			Some(item) => item,
		}
	};
}
