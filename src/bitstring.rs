/*!
User-facing types that wrap a decimal number encoded in a buffer.
*/

macro_rules! classify {
    ($d:ident) => {
        impl $d {
            /**
            Whether or not the sign bit is set.
            */
            pub fn is_sign_negative(&self) -> bool {
                $crate::binary::is_sign_negative(&self.0)
            }

            /**
            Whether or not the decimal is a finite number.
            */
            pub fn is_finite(&self) -> bool {
                $crate::binary::is_finite(&self.0)
            }

            /**
            Whether or not the decimal is an infinity.
            */
            pub fn is_infinite(&self) -> bool {
                $crate::binary::is_infinite(&self.0)
            }

            /**
            Whether the decimal is not a NaN.
            */
            pub fn is_nan(&self) -> bool {
                $crate::binary::is_nan(&self.0)
            }

            /**
            Whether the decimal is a qNaN.
            */
            pub fn is_quiet_nan(&self) -> bool {
                $crate::binary::is_quiet_nan(&self.0)
            }

            /**
            Whether the decimal is a sNaN.
            */
            pub fn is_signaling_nan(&self) -> bool {
                $crate::binary::is_signaling_nan(&self.0)
            }
        }
    };
}

macro_rules! d2s {
    ($d:ident) => {
        impl core::fmt::Debug for $d {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                $crate::convert::decimal_to_fmt(&self.0, f)
            }
        }

        impl core::fmt::Display for $d {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                $crate::convert::decimal_to_fmt(&self.0, f)
            }
        }
    };
}

macro_rules! try_s2d {
    ($b:ty => $d:ident) => {
        impl $d {
            /**
            Try parse a decimal from a string.

            This method is more efficient that `try_parse` if you already have a string to parse.
            */
            pub fn try_parse_str(s: &str) -> Result<$d, $crate::Error> {
                Ok($d($crate::convert::decimal_from_str(s)?))
            }

            /**
            Try parse a decimal from some formattable value.

            This method can avoid needing to buffer an entire number upfront.
            */
            pub fn try_parse(n: impl core::fmt::Display) -> Result<$d, $crate::Error> {
                Ok($d($crate::convert::decimal_from_fmt(n, <$b>::default())?))
            }
        }

        impl<'a> TryFrom<&'a str> for $d {
            type Error = $crate::Error;

            fn try_from(s: &'a str) -> Result<$d, Self::Error> {
                $d::try_parse_str(s)
            }
        }

        impl core::str::FromStr for $d {
            type Err = $crate::Error;

            fn from_str(s: &str) -> Result<$d, Self::Err> {
                $d::try_parse_str(s)
            }
        }
    };
}

macro_rules! i2d {
    ($i:ident => $convert:ident => $d:ident) => {
        impl $d {
            /**
            Convert an integer into a decimal.
            */
            pub fn $convert(i: $i) -> $d {
                $d($crate::convert::decimal_from_int(i).expect("infallible conversion"))
            }
        }

        impl From<$i> for $d {
            fn from(i: $i) -> $d {
                $d::$convert(i)
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            let _ = $d::$convert(0);
            let _ = $d::$convert($i::MIN);
            let _ = $d::$convert($i::MAX);
        }
    };
}

macro_rules! try_i2d {
    ($i:ident => $convert:ident => $d:ident) => {
        impl $d {
            /**
            Try convert an integer into a decimal.
            */
            pub fn $convert(i: $i) -> Option<$d> {
                Some($d($crate::convert::decimal_from_int(i).ok()?))
            }
        }

        impl TryFrom<$i> for $d {
            type Error = $crate::Error;

            fn try_from(i: $i) -> Result<$d, Self::Error> {
                Ok($d($crate::convert::decimal_from_int(i)?))
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            assert!(
                $d::$convert(0).is_some(),
                "{} should have been converted",
                0
            );

            if $i::MIN != 0 {
                assert!(
                    $d::$convert($i::MIN).is_none(),
                    "{} should not have been converted",
                    $i::MIN
                );
            }

            assert!(
                $d::$convert($i::MAX).is_none(),
                "{} should not have been converted",
                $i::MAX
            );
        }
    };
}

macro_rules! try_d2i {
    ($d:ident => $convert:ident => $i:ident) => {
        impl $d {
            /**
            Try convert a decimal into an integer.
            */
            pub fn $convert(&self) -> Option<$i> {
                $crate::convert::decimal_to_int(&self.0).ok()
            }
        }

        impl TryFrom<$d> for $i {
            type Error = $crate::Error;

            fn try_from(d: $d) -> Result<$i, Self::Error> {
                Ok($crate::convert::decimal_to_int(&d.0)?)
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            assert!(
                $d::zero().$convert().is_some(),
                "{} should have been converted",
                $d::zero()
            );

            if $i::MIN != 0 {
                if let Some::<$d>(min) = $d::min().into() {
                    assert!(
                        min.$convert().is_none(),
                        "{} should not have been converted",
                        min
                    );
                }
            }

            if let Some::<$d>(max) = $d::max().into() {
                assert!(
                    max.$convert().is_none(),
                    "{} should not have been converted",
                    max
                );
            }
        }
    };
}

macro_rules! f2d {
    ($f:ident => $convert:ident => $d:ident) => {
        impl $d {
            /**
            Convert a binary floating point into a decimal.
            */
            pub fn $convert(f: $f) -> $d {
                $d($crate::convert::decimal_from_binary_float(f).expect("infallible conversion"))
            }
        }

        impl From<$f> for $d {
            fn from(f: $f) -> $d {
                $d::$convert(f)
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            let _ = $d::$convert(0.0);
            let _ = $d::$convert($f::MIN);
            let _ = $d::$convert($f::MAX);
            let _ = $d::$convert($f::MIN_POSITIVE);
        }
    };
}

macro_rules! d2f {
    ($d:ident => $convert:ident => $f:ident) => {
        impl $d {
            /**
            Convert a decimal into a binary floating point.
            */
            pub fn $convert(&self) -> $f {
                $crate::convert::decimal_to_binary_float(&self.0).expect("infallible conversion")
            }
        }

        impl From<$d> for $f {
            fn from(d: $d) -> $f {
                d.$convert()
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            let _ = $d::zero().$convert();

            if let Some::<$d>(min) = $d::min().into() {
                let _ = min.$convert();
            }

            if let Some::<$d>(max) = $d::max().into() {
                let _ = max.$convert();
            }

            if let Some::<$d>(min_positive) = $d::min_positive().into() {
                let _ = min_positive.$convert();
            }
        }
    };
}

macro_rules! try_f2d {
    ($f:ident => $convert:ident => $d:ident) => {
        impl $d {
            /**
            Try convert a binary floating point into a decimal.
            */
            pub fn $convert(f: $f) -> Option<$d> {
                Some($d($crate::convert::decimal_from_binary_float(f).ok()?))
            }
        }

        impl TryFrom<$f> for $d {
            type Error = $crate::Error;

            fn try_from(f: $f) -> Result<$d, Self::Error> {
                Ok($d($crate::convert::decimal_from_binary_float(f)?))
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            assert!(
                $d::$convert(0.0).is_some(),
                "{} should have been converted",
                0.0
            );
            assert!(
                $d::$convert($f::MIN).is_none(),
                "{} should not have been converted",
                $f::MIN
            );
            assert!(
                $d::$convert($f::MAX).is_none(),
                "{} should not have been converted",
                $f::MAX
            );
            assert!(
                $d::$convert($f::MIN_POSITIVE).is_none(),
                "{} should not have been converted",
                $f::MIN_POSITIVE
            );
        }
    };
}

macro_rules! try_d2f {
    ($d:ident => $convert:ident => $f:ident) => {
        impl $d {
            /**
            Try convert a decimal into a binary floating point.
            */
            pub fn $convert(&self) -> Option<$f> {
                Some($crate::convert::decimal_to_binary_float(&self.0).ok()?)
            }
        }

        impl TryFrom<$d> for $f {
            type Error = $crate::Error;

            fn try_from(d: $d) -> Result<$f, Self::Error> {
                Ok($crate::convert::decimal_to_binary_float(&d.0)?)
            }
        }

        #[cfg(test)]
        #[test]
        fn $convert() {
            assert!(
                $d::zero().$convert().is_some(),
                "{} should have been converted",
                $d::zero()
            );

            if let Some::<$d>(min) = $d::min().into() {
                assert!(
                    min.$convert().is_none(),
                    "{} should not have been converted",
                    min
                );
            }

            if let Some::<$d>(max) = $d::max().into() {
                assert!(
                    max.$convert().is_none(),
                    "{} should not have been converted",
                    max
                );
            }
        }
    };
}

mod dynamic;
mod fixed128;
mod fixed32;
mod fixed64;

#[cfg(feature = "arbitrary-precision")]
mod arbitrary;

#[cfg(feature = "arbitrary-precision")]
pub use self::arbitrary::*;

pub use self::{
    dynamic::*,
    fixed128::*,
    fixed32::*,
    fixed64::*,
};
