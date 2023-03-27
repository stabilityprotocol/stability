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

pub fn u256_to_h256(value: U256) -> H256 {
	let mut tmp = [0u8; 32];
	value.to_big_endian(&mut tmp);
	H256::from(tmp)
}
