pub(super) struct DeltaError {
	pub right: i8,
}

pub(super) fn safe_delta_u8(left: u8, right: i8) -> Result<u8, DeltaError> {
	if right > 0 {
		left.checked_add(right as u8).ok_or(DeltaError {
			right,
		})
	} else {
		let unsigned_right: u8 = right.unsigned_abs();
		left.checked_sub(unsigned_right).ok_or(DeltaError {
			right,
		})
	}
}
