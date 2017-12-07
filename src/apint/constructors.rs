
use apint::{ApInt, ApIntData};
use bitwidth::{BitWidth};
use errors::{Error, Result};
use storage::{Storage};
use digit::{Digit};
use digit;

use std::ptr::Unique;

impl Drop for ApInt {
	fn drop(&mut self) {
		if self.len.storage() == Storage::Ext {
			let len = self.len_digits();
			unsafe{
				drop(Vec::from_raw_parts(self.data.ext.as_ptr(), len, len))
			}
		}
	}
}

//  =======================================================================
///  Constructors
/// =======================================================================
impl ApInt {

	/// Creates a new `ApInt` from a given `i8` value with a bit-width of 8.
	#[inline]
	pub fn from_i8(val: i8) -> ApInt {
		ApInt::from_u8(val as u8)
	}

	/// Creates a new `ApInt` from a given `i8` value with a bit-width of 8.
	#[inline]
	pub fn from_u8(val: u8) -> ApInt {
		ApInt{len: BitWidth::w8(), data: ApIntData{inl: Digit(u64::from(val))}}
	}

	/// Creates a new `ApInt` from a given `i16` value with a bit-width of 16.
	#[inline]
	pub fn from_i16(val: i16) -> ApInt {
		ApInt::from_u16(val as u16)
	}

	/// Creates a new `ApInt` from a given `i16` value with a bit-width of 16.
	#[inline]
	pub fn from_u16(val: u16) -> ApInt {
		ApInt{len: BitWidth::w16(), data: ApIntData{inl: Digit(u64::from(val))}}
	}

	/// Creates a new `ApInt` from a given `i32` value with a bit-width of 32.
	#[inline]
	pub fn from_i32(val: i32) -> ApInt {
		ApInt::from_u32(val as u32)
	}

	/// Creates a new `ApInt` from a given `i32` value with a bit-width of 32.
	#[inline]
	pub fn from_u32(val: u32) -> ApInt {
		ApInt{len: BitWidth::w32(), data: ApIntData{inl: Digit(u64::from(val))}}
	}

	/// Creates a new `ApInt` from a given `i64` value with a bit-width of 64.
	#[inline]
	pub fn from_i64(val: i64) -> ApInt {
		ApInt::from_u64(val as u64)
	}

	/// Creates a new `ApInt` from a given `i64` value with a bit-width of 64.
	#[inline]
	pub fn from_u64(val: u64) -> ApInt {
		ApInt{len: BitWidth::w64(), data: ApIntData{inl: Digit(val)}}
	}

	/// Creates a new `ApInt` from a given `i64` value with a bit-width of 64.
	#[inline]
	pub fn from_i128(val: i128) -> ApInt {
		ApInt::from_u128(val as u128)
	}

	/// Creates a new `ApInt` from a given `i64` value with a bit-width of 64.
	#[inline]
	pub fn from_u128(val: u128) -> ApInt {
		let buffer = vec![
			Digit((val & 0xFFFF_FFFF_FFFF_FFFF) as u64),
			Digit((val >> digit::BITS) as u64)
		];
		assert_eq!(buffer.len(), 2);
		ApInt::from_iter(buffer)
			.expect("We just asserted that `buffer.len()` is exactly 2 \
				     so we can expect `ApInt::from_iter` to be successful.")
	}

	/// Creates a new `ApInt` from the given iterator over `Digit`s.
	/// 
	/// This results in `ApInt` instances with bitwidths that are a multiple
	/// of a `Digit`s bitwidth (e.g. 64 bit).
	/// 
	/// Users of this API may truncate, extend or simply resize the resulting
	/// `ApInt` afterwards to obtain the desired bitwidth. This may be very cheap
	/// depending on the difference between the actual and target bitwidths.
	/// For example, `move_truncate`ing a `128` bitwidth `ApInt` to `100` is
	/// relatively cheap and won't allocate memory since both `ApInt` instances can use
	/// the same amount of `Digit`s.
	/// 
	/// # Errors
	/// 
	/// - If the iterator yields no elements.
	pub(crate) fn from_iter<I>(digits: I) -> Result<ApInt>
		where I: IntoIterator<Item=Digit>,
	{
		let buffer = digits.into_iter().collect::<Vec<_>>();
		match buffer.len() {
			0 => {
				Err(Error::expected_non_empty_digits())
			}
			1 => {
				let first_and_only = *buffer
					.first()
					.expect("We have already asserted that `digits.len()` must be at exactly `1`.");
				Ok(ApInt{
					len: BitWidth::w64(),
					data: ApIntData{inl: first_and_only}
				})
			}
			n => {
				use std::mem;
				let bitwidth = BitWidth::new(n * digit::BITS)
					.expect("We have already asserted that the number of items the given Iterator \
						     iterates over is greater than `1` and thus non-zero and thus a valid `BitWidth`.");
				let ptr_buffer = buffer.as_ptr() as *mut Digit;
				mem::forget(buffer);
				Ok(ApInt{len: bitwidth, data: ApIntData{ext: unsafe{ Unique::new_unchecked(ptr_buffer) }}})
			}
		}
	}

	/// Creates a new `ApInt` that represents the repetition of the given digit
	/// up to the given bitwidth.
	/// 
	/// Note: The last digit in the generated sequence is truncated to make the `ApInt`'s
	///       value representation fit the given bit-width.
	fn repeat_digit<D>(bitwidth: BitWidth, digit: D) -> ApInt
		where D: Into<Digit>
	{
		let digit = digit.into();
		match bitwidth.storage() {
			Storage::Inl => {
				ApInt{len: bitwidth, data: ApIntData{inl: digit.truncated(bitwidth).unwrap()}}
			}
			Storage::Ext => {
				use std::mem;
				let req_blocks = bitwidth.required_blocks();
				let mut buffer = vec![digit; req_blocks];
				let last_width = bitwidth.to_usize() % digit::BITS;
				buffer.last_mut().unwrap().truncate(last_width).unwrap();
				assert_eq!(buffer.capacity(), req_blocks);
				let ptr_buffer = buffer.as_ptr() as *mut Digit;
				mem::forget(buffer);
				ApInt{len: bitwidth, data: ApIntData{ext: unsafe{ Unique::new_unchecked(ptr_buffer) }}}
			}
		}
	}

	/// Creates a new `ApInt` with the given bit-width that represents zero.
	pub fn zero(width: BitWidth) -> ApInt {
		ApInt::repeat_digit(width, digit::ZERO)
	}

	/// Creates a new `ApInt` with the given bit-width that represents one.
	pub fn one(width: BitWidth) -> ApInt {
		ApInt::from_u64(1).zero_extend(width).unwrap()
	}

	/// Creates a new `ApInt` with the given bit-width that has all bits set.
	/// 
	/// This is equal to calling `ApInt::zero(..)` with the given `width`.
	pub fn zeros(width: BitWidth) -> ApInt {
		ApInt::zero(width)
	}

	/// Creates a new `ApInt` with the given bit-width that has all bits set.
	pub fn ones(width: BitWidth) -> ApInt
	{
		ApInt::repeat_digit(width, Digit::ones())
	}
}

impl From<u8> for ApInt {
	#[inline]
	fn from(val: u8) -> ApInt {
		ApInt::from_u8(val)
	}
}

impl From<i8> for ApInt {
	#[inline]
	fn from(val: i8) -> ApInt {
		ApInt::from_i8(val)
	}
}

impl From<u16> for ApInt {
	#[inline]
	fn from(val: u16) -> ApInt {
		ApInt::from_u16(val)
	}
}

impl From<i16> for ApInt {
	#[inline]
	fn from(val: i16) -> ApInt {
		ApInt::from_i16(val)
	}
}

impl From<u32> for ApInt {
	#[inline]
	fn from(val: u32) -> ApInt {
		ApInt::from_u32(val)
	}
}

impl From<i32> for ApInt {
	#[inline]
	fn from(val: i32) -> ApInt {
		ApInt::from_i32(val)
	}
}

impl From<u64> for ApInt {
	#[inline]
	fn from(val: u64) -> ApInt {
		ApInt::from_u64(val)
	}
}

impl From<i64> for ApInt {
	#[inline]
	fn from(val: i64) -> ApInt {
		ApInt::from_i64(val)
	}
}

impl From<u128> for ApInt {
	#[inline]
	fn from(val: u128) -> ApInt {
		ApInt::from_u128(val)
	}
}

impl From<i128> for ApInt {
	#[inline]
	fn from(val: i128) -> ApInt {
		ApInt::from_i128(val)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::ops::Range;

	fn powers() -> impl Iterator<Item=u128> {
		(0..128).map(|p| 1 << p)
	}

	fn powers_from_to(range: Range<usize>) -> impl Iterator<Item=u128> {
		powers().skip(range.start).take(range.end - range.start)
	}

	mod tests {
		use super::{powers, powers_from_to};

		#[test]
		fn test_powers() {
			let mut pows = powers();
			assert_eq!(pows.next(), Some(1 << 0));
			assert_eq!(pows.next(), Some(1 << 1));
			assert_eq!(pows.next(), Some(1 << 2));
			assert_eq!(pows.next(), Some(1 << 3));
			assert_eq!(pows.next(), Some(1 << 4));
			assert_eq!(pows.next(), Some(1 << 5));
			assert_eq!(pows.last(), Some(1 << 127));
		}

		#[test]
		fn test_powers_from_to() {
			{
				let mut powsft = powers_from_to(0..4);
				assert_eq!(powsft.next(), Some(1 << 0));
				assert_eq!(powsft.next(), Some(1 << 1));
				assert_eq!(powsft.next(), Some(1 << 2));
				assert_eq!(powsft.next(), Some(1 << 3));
				assert_eq!(powsft.next(), None);
			}
			{
				let mut powsft = powers_from_to(4..7);
				assert_eq!(powsft.next(), Some(1 << 4));
				assert_eq!(powsft.next(), Some(1 << 5));
				assert_eq!(powsft.next(), Some(1 << 6));
				assert_eq!(powsft.next(), None);
			}
		}
	}

	fn test_values_u8() -> impl Iterator<Item=u8> {
		powers_from_to(0..8)
			.map(|v| v as u8)
			.chain([
				u8::max_value(),
				10,
				42,
				99,
				123
			].into_iter()
			 .map(|v| *v))
	}

	#[test]
	fn from_w8() {
		for val in test_values_u8() {
			let explicit_u8 = ApInt::from_u8(val);
			let explicit_i8 = ApInt::from_i8(val as i8);
			let implicit_u8 = ApInt::from(val);
			let implicit_i8 = ApInt::from(val as i8);
			let expected = ApInt{
				len : BitWidth::w8(),
				data: ApIntData{inl: Digit(u64::from(val))}
			};
			assert_eq!(explicit_u8, explicit_i8);
			assert_eq!(explicit_u8, implicit_i8);
			assert_eq!(explicit_u8, implicit_u8);
			assert_eq!(explicit_u8, expected);
		}
	}

	fn test_values_u16() -> impl Iterator<Item=u16> {
		test_values_u8()
			.map(u16::from)
			.chain(powers_from_to(8..16)
				.map(|v| v as u16))
			.chain([
				u16::max_value(),
				500,
				1000,
				1337,
				7777,
				42_000
			].into_iter().map(|v| *v))
	}

	#[test]
	fn from_w16() {
		for val in test_values_u16() {
			let explicit_u16 = ApInt::from_u16(val);
			let explicit_i16 = ApInt::from_i16(val as i16);
			let implicit_u16 = ApInt::from(val);
			let implicit_i16 = ApInt::from(val as i16);
			let expected = ApInt{
				len : BitWidth::w16(),
				data: ApIntData{inl: Digit(u64::from(val))}
			};
			assert_eq!(explicit_u16, explicit_i16);
			assert_eq!(explicit_u16, implicit_i16);
			assert_eq!(explicit_u16, implicit_u16);
			assert_eq!(explicit_u16, expected);
		}
	}

}
