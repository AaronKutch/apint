use crate::{
    storage::Storage,
    BitPos,
    Digit,
    Error,
    Result,
    ShiftAmount,
};

use core::{
    convert::{
        TryFrom,
        TryInto,
    },
    num::NonZeroUsize,
};

/// The `BitWidth` represents the length of an `ApInt`.
///
/// Its invariant restricts it to always be a positive, non-zero value.
/// Code that built's on top of `BitWidth` may and should use this invariant.
///
/// This is currently just a wrapper around `NonZeroUsize` (in case
/// future compiler optimizations can make use of it), but this is not
/// exposed because of the option of custom functions and
/// allowing forks of `ApInt` to use other internal types.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitWidth(NonZeroUsize);

impl From<NonZeroUsize> for BitWidth {
    /// Creates a `BitWidth` from the given `NonZeroUsize`.
    fn from(width: NonZeroUsize) -> Self {
        BitWidth(width)
    }
}

impl TryFrom<usize> for BitWidth {
    type Error = Error;

    /// Creates a `BitWidth` from the given `usize`.
    ///
    /// # Errors
    ///
    /// - If the given `width` is equal to zero.
    fn try_from(width: usize) -> Result<Self> {
        match NonZeroUsize::new(width) {
            Some(bitwidth) => Ok(BitWidth(bitwidth)),
            None => Err(Error::invalid_zero_bitwidth()),
        }
    }
}

pub(crate) fn bw<S>(width: S) -> BitWidth
where
    S: TryInto<BitWidth>,
{
    // For this case, we erase the regular error unwrapping message by converting
    // the `Result` to an `Option`, and displaying a different message.
    width.try_into().ok().expect(
        "Tried to construct an invalid BitWidth of 0 using the `apint::bw` function",
    )
}

impl BitWidth {
    /// Converts this `BitWidth` into a `usize`.
    #[inline]
    pub fn to_usize(self) -> usize {
        self.0.get()
    }

    /// Returns a storage specifier that tells the caller if `ApInt`'s
    /// associated with this bitwidth require an external memory (`Ext`) to
    /// store their digits or may use inplace memory (`Inl`).
    ///
    /// *Note:* Maybe this method should be removed. A constructor for
    ///         `Storage` fits better for this purpose.
    #[inline]
    pub(crate) fn storage(self) -> Storage {
        Storage::from(self)
    }

    /// Returns the number of exceeding bits that is implied for `ApInt`
    /// instances with this `BitWidth`.
    ///
    /// For example for an `ApInt` with a `BitWidth` of `140` bits requires
    /// exactly `3` digits (each with its `64` bits). The third however,
    /// only requires `140 - 128 = 12` bits of its `64` bits in total to
    /// represent the `ApInt` instance. So `excess_bits` returns `12` for
    /// a `BitWidth` that is equal to `140`.
    ///
    /// *Note:* A better name for this method has yet to be found!
    pub(crate) fn excess_bits(self) -> Option<usize> {
        match self.to_usize() % Digit::BITS {
            0 => None,
            n => Some(n),
        }
    }

    /// Returns the exceeding `BitWidth` of this `BitWidth`.
    ///
    /// *Note:* This is just a simple wrapper around the `excess_bits` method.
    ///         Read the documentation of `excess_bits` for more information
    ///         about what is actually returned by this.
    pub(crate) fn excess_width(self) -> Option<BitWidth> {
        match self.to_usize() % Digit::BITS {
            0 => None,
            n => BitWidth::try_from(n).ok(),
        }
    }

    /// Returns the number of digits that are required to represent an
    /// `ApInt` with this `BitWidth`.
    ///
    /// *Note:* Maybe we should move this method somewhere else?
    #[inline]
    pub(crate) fn required_digits(self) -> usize {
        ((self.to_usize() - 1) / Digit::BITS) + 1
    }

    /// Returns `true` if the given `BitPos` is valid for this `BitWidth`.
    #[inline]
    pub(crate) fn is_valid_pos<P>(self, pos: P) -> bool
    where
        P: Into<BitPos>,
    {
        pos.into().to_usize() < self.to_usize()
    }

    /// Returns `true` if the given `ShiftAmount` is valid for this `BitWidth`.
    #[inline]
    pub(crate) fn is_valid_shift_amount<S>(self, shift_amount: S) -> bool
    where
        S: Into<ShiftAmount>,
    {
        shift_amount.into().to_usize() < self.to_usize()
    }

    /// Returns the `BitPos` for the sign bit of an `ApInt` with this
    /// `BitWidth`.
    #[inline]
    pub(crate) fn sign_bit_pos(self) -> BitPos {
        BitPos::from(self.to_usize() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod excess_bits {
        use super::*;

        #[test]
        fn multiples_of_50() {
            assert_eq!(bw(50).unwrap().excess_bits(), Some(50));
            assert_eq!(bw(100).unwrap().excess_bits(), Some(36));
            assert_eq!(bw(150).unwrap().excess_bits(), Some(22));
            assert_eq!(bw(200).unwrap().excess_bits(), Some(8));
            assert_eq!(bw(250).unwrap().excess_bits(), Some(58));
            assert_eq!(bw(300).unwrap().excess_bits(), Some(44));
        }
    }
}
