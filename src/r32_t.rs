#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt;
use std::cmp::Ordering;
use std::ops::*;
use std::str::FromStr;

use gcd::Gcd;

use super::{ParseRatioErr, RatioErrKind};

/// The 32-bit floating bar type.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Eq, Default, Hash)]
pub struct r32(u32);

const SIGN_BIT: u32 = 0x8000_0000;
const SIZE_FIELD: u32 = SIGN_BIT - 1 << FRACTION_SIZE + 1 >> 1;
const FRACTION_FIELD: u32 = (1 << FRACTION_SIZE) - 1;

const FRACTION_SIZE: u32 = 26;

pub const NAN: r32 = r32(SIZE_FIELD);
pub const MAX: r32 = r32(FRACTION_FIELD);
pub const MIN: r32 = r32(SIGN_BIT | FRACTION_FIELD);
pub const MIN_POSITIVE: r32 = r32(FRACTION_SIZE << FRACTION_SIZE | FRACTION_FIELD);

impl r32 {
    // TODO unfinished; check input values that overflow
    #[inline]
    fn new(num: i32, den: u32) -> r32 {
        let size = 32 - den.leading_zeros() - 1;
        let denom_field = (1 << size) - 1;
        
        r32(0).set_sign(num.is_negative())
        .set_denom_size(size)
        .set_fraction(
            ((denom_field ^ FRACTION_FIELD) >> size & num.abs() as u32) << size |
            den & denom_field
        )
    }
    
    #[inline]
    fn denom_size(self) -> u32 {
        (self.0 & SIZE_FIELD) >> FRACTION_SIZE
    }
    
    /// Returns the numerator value for this rational number.
    #[inline]
    pub fn numer(self) -> u32 {
        if self.denom_size() == FRACTION_SIZE { 1 }
        else { (self.0 & FRACTION_FIELD) >> self.denom_size() }
    }
    
    /// Returns the denominator value for this rational number.
    #[inline]
    pub fn denom(self) -> u32 {
        let denom_region = (1 << self.denom_size()) - 1;
        self.0 & denom_region | 1 << self.denom_size()
    }
    
    /// Sets sign bit to the value given.
    /// 
    /// If `true`, sign bit is set. Otherwise, it's unset.
    #[inline]
    fn set_sign(self, sign: bool) -> r32 {
        r32(self.0 & !SIGN_BIT | (sign as u32) << 31)
    }
    
    #[inline]
    fn set_denom_size(self, size: u32) -> r32 {
        r32(self.0 & !SIZE_FIELD | (size & 0x1f) << FRACTION_SIZE)
    }
    
    #[inline]
    fn set_fraction(self, frac: u32) -> r32 {
        r32(self.0 & !FRACTION_FIELD | frac & FRACTION_FIELD)
    }
    
    #[inline]
    fn from_parts(sign: bool, numer: u32, denom: u32) -> r32 {
        let size = 32 - denom.leading_zeros() - 1;
        let denom_field = (1 << size) - 1;
        r32(
            if sign { SIGN_BIT } else { 0 } |
            size << FRACTION_SIZE |
            ((denom_field ^ FRACTION_FIELD) >> size & numer) << size |
            denom & denom_field
        )
    }
    
    #[inline]
    fn is_sign_positive(self) -> bool {
        self.0 & SIGN_BIT == 0
    }
    
    #[inline]
    fn is_sign_negative(self) -> bool {
        self.0 & SIGN_BIT != 0
    }
    
    // BEGIN related float stuff
    
    /// Returns the largest integer less than or equal to a number.
    #[inline]
    pub fn floor(self) -> r32 {
        unimplemented!()
    }
    
    /// Returns the smallest integer greater than or equal to a number.
    #[inline]
    pub fn ceil(self) -> r32 {
        unimplemented!()
    }
    
    // TODO is rounding away from 0 necessary here, or is it a floating-point
    // accuracy detail?
    /// Returns the nearest integer to a number. Round half-way cases away from `0`.
    #[inline]
    pub fn round(self) -> r32 {
        unimplemented!()
    }
    
    /// Returns the integer part of a number.
    #[inline]
    pub fn trunc(self) -> r32 {
        r32::from_parts(self.is_negative(), self.numer() / self.denom(), 1)
    }
    
    /// Returns the fractional part of a number.
    #[inline]
    pub fn fract(self) -> r32 {
        let d = self.denom();
        r32::from_parts(self.is_negative(), self.numer() % d, d)
    }
    
    /// Computes the absolute value of `self`. Returns NaN if the number is NaN.
    #[inline]
    pub fn abs(self) -> r32 {
        self.set_sign(false)
    }
    
    /// Returns a number that represents the sign of `self`.
    /// 
    /// * `1` if the number is positive
    /// * `-1` if the number is negative
    /// * `0` if the number is `+0`, `-0`, or `NaN`
    #[inline]
    pub fn signum(self) -> r32 {
        if self.numer() == 0 || self.is_nan() {
            r32(0)
        }
        else {
            r32(self.0 & SIGN_BIT | 1)
        }
    }
    
    /// Raises a number to an integer power.
    #[inline]
    pub fn pow(self, n: i32) -> r32 {
        unimplemented!()
    }
    /*
    TODO consider whether to actually add these.
    /// Takes the square root of a number.
    /// 
    /// If `self` is positive and numerator and denominator are perfect squares
    /// and are positive, returns their square root. Otherwise, returns `None`.
    #[inline]
    pub fn checked_sqrt(self) -> Option<r32> {
        unimplemented!()
    }
    
    /// Takes the cube root of a number.
    /// 
    /// If `self` is positive and its numerator and denominator are perfect
    /// cubes, returns their cube root. Otherwise, returns `None`.
    #[inline]
    pub fn checked_cbrt(self) -> Option<r32> {
        unimplemented!()
    }
    
    // TODO maybe?
    /// Calculates the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    #[inline]
    pub fn hypot(self) -> r32 {
        unimplemented!()
    }
    */
    /// Returns `true` if this value is `NaN` and `false` otherwise.
    #[inline]
    pub fn is_nan(self) -> bool {
        self.denom_size() > FRACTION_SIZE
    }
    
    /// Returns `true` if the number is neither zero, subnormal, or `NaN`.
    #[inline]
    pub fn is_normal(self) -> bool {
        unimplemented!()
    }
    
    /// Returns `true` if and only if `self` has a positive sign, including
    /// `+0.0` (but not NaNs with positive sign bit).
    #[inline]
    pub fn is_positive(self) -> bool {
        self.numer() != 0 && self.is_sign_positive()
    }
    
    /// Returns `true` if and only if self has a negative sign, including
    /// `-0.0` (but not NaNs with negative sign bit).
    #[inline]
    pub fn is_negative(self) -> bool {
        self.numer() != 0 && self.is_sign_negative()
    }
    
    /// Takes the reciprocal (inverse) of a number, `1/x`.
    /// 
    /// # Panics
    /// 
    /// Panics when trying to set a numerator of zero as denominator.
    #[inline]
    pub fn recip(self) -> r32 {
        assert!(self.numer() != 0, "attempt to divide by zero");
        assert!(self.denom_size() < 26, "subnormal overflow");
        r32::from_parts(self.is_negative(), self.denom(), self.numer())
    }
    
    /// Returns the maximum of the two numbers.
    /// 
    /// If one of the arguments is `NaN`, then the other argument is returned.
    #[inline]
    pub fn max(self, other: r32) -> r32 {
        match (self.is_nan(), other.is_nan()) {
            // this clobbers any "payload" bits being used.
            (true, true) => NAN,
            (true, false) => self,
            (false, true) => other,
            (false, false) => match self.partial_cmp(&other).unwrap() {
                Ordering::Less    => other,
                Ordering::Greater => self,
                // return self by default
                Ordering::Equal   => self,
            }
        }
    }
    
    /// Returns the minimum of the two numbers.
    /// 
    /// If one of the arguments is `NaN`, then the other argument is returned.
    #[inline]
    pub fn min(self, other: r32) -> r32 {
        match (self.is_nan(), other.is_nan()) {
            // this clobbers any "payload" bits being used.
            (true, true) => NAN,
            (true, false) => self,
            (false, true) => other,
            (false, false) => match self.partial_cmp(&other).unwrap() {
                Ordering::Greater => other,
                Ordering::Less    => self,
                // return self by default
                Ordering::Equal   => self,
            }
        }
    }
    
    /// Raw transmutation to `u32`.
    #[inline]
    pub fn to_bits(self) -> u32 { self.0 }
    
    /// Raw transmutation from `u32`.
    #[inline]
    pub fn from_bits(bits: u32) -> r32 { r32(bits) }
    
    /// Cancels out common factors between the numerator and the denominator.
    pub fn simplify(self) -> r32 {
        if self.is_nan() {
            return self;
        }
        
        if self.numer() == 0 {
            return r32(0);
        }
        
        let n = self.numer();
        let d = self.denom();
        
        // cancel out common factors
        let gcd = n.gcd(d);
        r32::from_parts(self.is_negative(), n / gcd, d / gcd)
    }
    
    // BEGIN related integer stuff
    
    /// Checked integer addition. Computes `self + rhs`, returning `None` if
    /// overflow occurred.
    pub fn checked_add(self, rhs: r32) -> Option<r32> {
        unimplemented!()
    }
    
    /// Checked integer subtraction. Computes `self - rhs`, returning `None` if
    /// overflow occurred.
    pub fn checked_sub(self, rhs: r32) -> Option<r32> {
        unimplemented!()
    }
    
    /// Checked integer multiplication. Computes `self * rhs`, returning `None`
    /// if overflow occurred.
    pub fn checked_mul(self, rhs: r32) -> Option<r32> {
        unimplemented!()
    }
    
    /// Checked integer division. Computes `self / rhs`, returning `None` if
    /// `rhs == 0` or the division results in overflow.
    pub fn checked_div(self, rhs: r32) -> Option<r32> {
        unimplemented!()
    }
    
    /// Checked integer remainder. Computes `self % rhs`, returning `None` if
    /// `rhs == 0` or the division results in overflow.
    pub fn checked_rem(self, rhs: r32) -> Option<r32> {
        unimplemented!()
    }
    
    /// Calculates `self` + `rhs`
    /// 
    /// Returns a tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    pub fn overflowing_add(self, rhs: r32) -> (r32, bool) {
        unimplemented!()
    }
    
    /// Calculates `self` - `rhs`
    /// 
    /// Returns a tuple of the subtraction along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_sub(self, rhs: r32) -> (r32, bool) {
        unimplemented!()
    }
    
    /// Calculates the multiplication of `self` and `rhs`.
    /// 
    /// Returns a tuple of the multiplication along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_mul(self, rhs: r32) -> (r32, bool) {
        unimplemented!()
    }
    
    /// Calculates the divisor when `self` is divided by `rhs`.
    /// 
    /// Returns a tuple of the divisor along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would occur then self
    /// is returned.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `rhs` is 0.
    pub fn overflowing_div(self, rhs: r32) -> (r32, bool) {
        unimplemented!()
    }
    
    /// Calculates the remainder when `self` is divided by `rhs`.
    /// 
    /// Returns a tuple of the remainder after dividing along with a boolean
    /// indicating whether an arithmetic overflow would occur. If an overflow
    /// would occur then 0 is returned.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `rhs` is 0.
    pub fn overflowing_rem(self, rhs: r32) -> (r32, bool) {
        unimplemented!()
    }
}

impl fmt::Display for r32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_nan() {
            return f.write_str("NaN");
        }
        
        let norm = self.simplify();
        
        if norm.is_negative() {
            f.write_str("-")?;
        }
        
        write!(f, "{}", norm.numer())?;
        
        if norm.denom_size() > 0 {
            write!(f, "/{}", norm.denom())?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for r32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_nan() {
            return f.write_str("NaN");
        }
        
        if self.is_sign_negative() {
            f.write_str("-")?;
        }
        
        write!(f, "{}/{}", self.numer(), self.denom())
    }
}

impl FromStr for r32 {
    type Err = ParseRatioErr;
    
    /// Converts a string in base 10 to a rational.
    /// 
    /// This function accepts strings such as
    /// 
    /// * '157/50'
    /// * '-157/50'
    /// * '25', or equivalently, '25/1'
    /// * 'NaN'
    /// 
    /// Leading and trailing whitespace represent an error.
    /// 
    /// # Return value
    /// 
    /// `Err(ParseRatioError)` if the string did not represent a valid number.
    /// Otherwise, `Ok(n)` where `n` is the floating-bar number represented by
    /// `src`.
    fn from_str(mut src: &str) -> Result<Self, Self::Err> {
        if src.is_empty() {
            return Err(ParseRatioErr { kind: RatioErrKind::Empty });
        }
        
        if src == "NaN" {
            return Ok(NAN);
        }
        
        // set sign
        let s = src.starts_with('-');
        
        // skip sign if set
        if src.starts_with('+') || src.starts_with('-') {
            src = &src[1..];
        }
        
        // if bar exists, parse as fraction, otherwise as integer.
        // TODO deal with unwraps below
        if let Some(pos) = src.find('/') {
            // bar is at the end. invalid.
            if pos == src.len() - 1 {
                return Err(ParseRatioErr { kind: RatioErrKind::Invalid });
            }
            
            let numerator: u32 = src[0..pos].parse().unwrap();
            let denominator: u32 = src[pos+1..].parse().unwrap();
            
            if denominator == 0 {
                return Err(ParseRatioErr { kind: RatioErrKind::Invalid });
            }
            
            let denom_size = 32 - denominator.leading_zeros() - 1;
            
            // subnormals
            if numerator == 1 && denom_size == 26 {
                let denominator = denominator & FRACTION_FIELD;
                Ok(r32::from_parts(s, 1, denominator))
            }
            else {
                let frac_size = denom_size + (32 - numerator.leading_zeros());
                
                if frac_size > FRACTION_SIZE {
                    Err(ParseRatioErr { kind: RatioErrKind::Overflow })
                }
                else {
                    Ok(r32::from_parts(s, numerator, denominator))
                }
            }
        }
        else {
            let numerator: u32 = src.parse().unwrap();
            let frac_size = 32 - numerator.leading_zeros();
            
            if frac_size > FRACTION_SIZE {
                return Err(ParseRatioErr { kind: RatioErrKind::Overflow });
            }
            
            Ok(r32::from_parts(s, numerator, 1))
        }
    }
}

impl From<u8> for r32 {
    #[inline]
    fn from(v: u8) -> Self { r32(v as u32) }
}

impl From<i8> for r32 {
    fn from(v: i8) -> Self {
        let n = if v == i8::min_value() { 128 } else { v.abs() as u32 };
        r32::from_parts(v.is_negative(), n, 1)
    }
}

impl From<u16> for r32 {
    #[inline]
    fn from(v: u16) -> Self { r32(v as u32) }
}

impl From<i16> for r32 {
    fn from(v: i16) -> Self {
        let n = if v == i16::min_value() { 32768 } else { v.abs() as u32 };
        r32::from_parts(v.is_negative(), n, 1)
    }
}

impl Into<f32> for r32 {
    fn into(self) -> f32 {
        let s = if self.is_negative() { -1.0 } else { 1.0 };
        s * self.numer() as f32 / self.denom() as f32
    }
}

impl Into<f64> for r32 {
    fn into(self) -> f64 {
        let s = if self.is_negative() { -1.0 } else { 1.0 };
        s * self.numer() as f64 / self.denom() as f64
    }
}

impl Neg for r32 {
    type Output = r32;
    
    fn neg(self) -> Self::Output {
        r32(self.0 ^ SIGN_BIT)
    }
}

impl PartialEq for r32 {
    fn eq(&self, other: &r32) -> bool {
        self.is_nan() && other.is_nan()
        || self.numer() == 0 && other.numer() == 0
        || self.simplify().0 == other.simplify().0
    }
}

impl PartialOrd for r32 {
    fn partial_cmp(&self, other: &r32) -> Option<Ordering> {
        // both are nan or both are zero
        if self.is_nan() && other.is_nan()
        || self.numer() == 0 && other.numer() == 0 {
            return Some(Ordering::Equal);
        }
        
        // only one of them is nan
        if self.is_nan() || other.is_nan() {
            return None;
        }
        
        // compare signs
        self.is_sign_positive()
        .partial_cmp(&other.is_sign_positive())
        .map(|c| c.then(
            // compare numbers
            // a/b = c/d <=> ad = bc
            // when a, b, c, and d are all > 0
            (self.numer() as u64 * other.denom() as u64)
            .cmp(&(self.denom() as u64 * other.numer() as u64))
        ))
    }
}

impl Mul for r32 {
    type Output = r32;
    
    fn mul(self, other: r32) -> r32 {
        let s = self.is_negative() != other.is_negative();
        let mut n = self.numer() as u64 * other.numer() as u64;
        let mut d = self.denom() as u64 * other.denom() as u64;
        
        let gcd = n.gcd(d);
        n /= gcd;
        d /= gcd;
        
        debug_assert!(
            (64 - d.leading_zeros() - 1) + (64 - n.leading_zeros()) <= FRACTION_SIZE,
            "attempt to multiply with overflow"
        );
        
        r32::from_parts(s, n as u32, d as u32)
    }
}

impl Div for r32 {
    type Output = r32;

    fn div(self, other: r32) -> r32 {
        self * other.recip()
    }
}

impl Add for r32 {
    type Output = r32;
    
    fn add(self, other: r32) -> r32 {
        // self = a/b, other = c/d
        
        let selfsign = (self.signum().0 as i32).signum() as i64;
        let othersign = (other.signum().0 as i32).signum() as i64;
        
        // TODO prove this won't panic/can't overflow.
        // num = ad + bc
        let num =
            (self.numer() as i64 * selfsign) * other.denom() as i64
            + self.denom() as i64 * (other.numer() as i64 * othersign);
        // den = bd
        let mut den = self.denom() as u64 * other.denom() as u64;
        let s = num.is_negative();
        let mut num = num.abs() as u64;
        
        let gcd = num.gcd(den);
        num /= gcd;
        den /= gcd;
        
        debug_assert!(
            (64 - den.leading_zeros() - 1) + (64 - num.leading_zeros()) <= FRACTION_SIZE,
            "attempt to add with overflow"
        );
        
        r32::from_parts(s, num as u32, den as u32)
    }
}

impl Sub for r32 {
    type Output = r32;

    fn sub(self, other: r32) -> r32 {
        self + -other
    }
}

impl Rem for r32 {
    type Output = r32;
    
    fn rem(self, other: r32) -> r32 {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn simplify() {
        assert_eq!(r32::from_parts(false, 4, 2).simplify(), r32::from_parts(false, 2, 1));
        assert_eq!(r32::from_parts(true, 4, 2).simplify(), r32::from_parts(true, 2, 1));
    }
    
    #[test]
    fn neg() {
        assert_eq!((-r32(0)).0, SIGN_BIT);
        assert_eq!((-r32(SIGN_BIT)).0, 0);
    }
    
    #[test]
    fn signum() {
        assert_eq!(r32(0).signum(), r32(0));
        assert_eq!(r32(SIGN_BIT).signum(), r32(0));
        assert_eq!(r32(1).signum(), r32(1));
        assert_eq!(r32(2).signum(), r32(1));
        assert_eq!(r32::from_parts(true, 1, 1).signum(), r32::from_parts(true, 1, 1));
        assert_eq!(r32::from_parts(true, 2, 1).signum(), r32::from_parts(true, 1, 1));
    }
    
    #[test]
    fn fract() {
        assert_eq!(r32(5).fract(), r32(0));
        assert_eq!(r32::from_parts(false, 3, 2).fract(), r32::from_parts(false, 1, 2));
        assert_eq!(r32::from_parts(true, 3, 2).fract(), r32::from_parts(true, 1, 2));
    }
    
    #[test]
    fn trunc() {
        assert_eq!(r32(5).trunc(), r32(5));
        assert_eq!(r32::from_parts(false, 3, 2).trunc(), r32(1));
        assert_eq!(r32::from_parts(true, 3, 2).trunc(), r32::from(-1 as i8));
    }
    
    #[test]
    fn recip() {
        assert_eq!(r32(5).recip(), r32::from_parts(false, 1, 5));
        assert_eq!(r32::from_parts(false, 5, 2).recip(), r32::from_parts(false, 2, 5));
        assert_eq!(r32(1).recip(), r32(1));
    }
    
    #[test]
    fn cmp() {
        assert!(r32(0) == r32(0));
        assert!(r32(0) == -r32(0));
        
        assert!(r32(0) < r32(1));
        assert!(r32(2) < r32(3));
        assert!(r32(0) > -r32(1));
        assert!(r32(2) > -r32(3));
    }
    
    #[test]
    fn mul() {
        assert_eq!(r32(0) * r32(0), r32(0));
        
        assert_eq!(r32(0) * r32(1), r32(0));
        assert_eq!(r32(1) * r32(0), r32(0));
        assert_eq!(r32(1) * r32(1), r32(1));
        
        assert_eq!(-r32(1) * r32(1), -r32(1));
        assert_eq!(r32(1) * -r32(1), -r32(1));
        assert_eq!(-r32(1) * -r32(1), r32(1));
        
        assert_eq!(r32(1) * r32(2), r32(2));
        assert_eq!(r32(2) * r32(2), r32(4));
        
        assert_eq!(
            r32::from_parts(false, 1, 2) * r32::from_parts(false, 1, 2),
            r32::from_parts(false, 1, 4)
        );
        assert_eq!(
            r32::from_parts(true, 1, 2) * r32::from_parts(false, 1, 2),
            r32::from_parts(true, 1, 4)
        );
        assert_eq!(
            r32::from_parts(false, 2, 3) * r32::from_parts(false, 2, 3),
            r32::from_parts(false, 4, 9)
        );
        assert_eq!(
            r32::from_parts(false, 3, 2) * r32::from_parts(false, 2, 3),
            r32(1)
        );
    }
    
    #[test] #[should_panic]
    fn mul_invalid() {
        let _ = r32(1 << FRACTION_SIZE - 1) * r32(1 << FRACTION_SIZE - 1);
    }
    
    #[test]
    fn div() {
        assert_eq!(r32(0) / r32(1), r32(0));
        assert_eq!(r32(0) / r32(2), r32(0));
        assert_eq!(r32(1) / r32(1), r32(1));
        
        assert_eq!(-r32(1) / r32(1), -r32(1));
        assert_eq!(r32(1) / -r32(1), -r32(1));
        assert_eq!(-r32(1) / -r32(1), r32(1));
        
        assert_eq!(r32(1) / r32(2), r32::from_parts(false, 1, 2));
        assert_eq!(r32(2) / r32(1), r32(2));
        assert_eq!(r32(2) / r32(2), r32(1));
    }
    
    #[test]
    fn add() {
        assert_eq!(r32(0) + r32(0), r32(0));
        assert_eq!(-r32(0) + r32(0), r32(0));
        
        assert_eq!(r32(1) + r32(1), r32(2));
        assert_eq!(r32(1) + -r32(1), r32(0));
        assert_eq!(-r32(1) + r32(1), r32(0));
        assert_eq!(-r32(1) + -r32(1), -r32(2));
        
        assert_eq!(r32(2) + r32(2), r32(4));
        assert_eq!(
            r32::from_parts(false, 1, 2) + r32::from_parts(false, 3, 4),
            r32::from_parts(false, 5, 4)
        );
        assert_eq!(
            r32::from_parts(false, 1, 2) + r32::from_parts(true, 3, 4),
            r32::from_parts(true, 1, 4)
        );
        assert_eq!(
            r32::from_parts(true, 1, 2) + r32::from_parts(false, 3, 4),
            r32::from_parts(false, 1, 4)
        );
    }
    
    #[test] #[should_panic]
    fn add_invalid() {
        let _ = r32(1 << FRACTION_SIZE - 1) + r32(1 << FRACTION_SIZE - 1);
    }
    
    #[test]
    fn from_str() {
        assert_eq!("0".parse::<r32>().unwrap(), r32(0));
        assert_eq!("1".parse::<r32>().unwrap(), r32(1));
        assert_eq!("+1".parse::<r32>().unwrap(), r32(1));
        assert_eq!("-1".parse::<r32>().unwrap(), r32::from(-1 as i8));
        assert_eq!("1/1".parse::<r32>().unwrap(), r32(1));
    }
    
    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", r32::from_parts(true, 0, 1)), "-0/1");
        assert_eq!(format!("{:?}", NAN), "NaN");
    }
    
    #[test]
    fn display() {
        assert_eq!(format!("{}", r32::from_parts(false, 0, 1)), "0");
        assert_eq!(format!("{}", NAN), "NaN");
        assert_eq!(format!("{}", r32::from_parts(true, 3, 2)), "-3/2");
    }
}
