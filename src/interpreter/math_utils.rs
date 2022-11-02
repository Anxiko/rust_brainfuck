pub fn safe_delta_u8(left: u8, right: i8) -> Option<u8> {
	if right > 0 {
		left.checked_add(right as u8)
	} else {
		let right: u8 = right.unsigned_abs();
		left.checked_sub(right)
	}
}
