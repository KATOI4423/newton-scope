use std::fmt::LowerExp;
use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
    Rem, RemAssign,
    Neg,
};
use std::str::FromStr;

use formulac::core::Real;
use num_traits::{
    Num,
    ToPrimitive,
    One,
    Zero,
};
use serde::{Deserialize, Serialize};
use twofloat::{
    TwoFloat,
    TwoFloatError,
};

use crate::multi_precision::MD;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct F106 {
    inner: TwoFloat,
}

impl F106 {
    pub fn to_md128(&self) -> MD<128> {
        MD::<128>::from_f64(self.inner.hi()) + MD::<128>::from_f64(self.inner.lo())
    }
    pub fn to_f64(&self) -> f64 {
        self.inner.hi() + self.inner.lo()
    }
}

impl LowerExp for F106 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let md = self.to_md128();
        LowerExp::fmt(&md, f)
    }
}

impl From<TwoFloat> for F106 {
    fn from(value: TwoFloat) -> Self {
        Self { inner: value }
    }
}

impl Default for F106 {
    fn default() -> Self {
        Self { inner: TwoFloat::default() }
    }
}

impl Neg for F106 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from(self.inner.neg())
    }
}

impl Add for F106 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.inner.add(rhs.inner))
    }
}

impl Sub for F106 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.inner.sub(rhs.inner))
    }
}

impl Mul for F106 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from(self.inner.mul(rhs.inner))
    }
}

impl Div for F106 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from(self.inner.div(rhs.inner))
    }
}

impl Rem for F106 {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self::from(self.inner.rem(rhs.inner))
    }
}

impl AddAssign for F106 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl SubAssign for F106 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

impl MulAssign for F106 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul(rhs)
    }
}

impl DivAssign for F106 {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.div(rhs)
    }
}

impl RemAssign for F106 {
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.rem(rhs)
    }
}

impl Zero for F106 {
    fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }
    fn set_zero(&mut self) {
        self.inner.set_zero();
    }
    fn zero() -> Self {
        Self::from(TwoFloat::zero())
    }
}

impl One for F106 {
    fn is_one(&self) -> bool {
        self.inner.is_one()
    }
    fn one() -> Self {
        Self::from(TwoFloat::one())
    }
    fn set_one(&mut self) {
        self.inner.set_one();
    }
}

impl Num for F106 {
    type FromStrRadixErr = TwoFloatError;

    /// twofloat v0.8.4 のfrom_str_radix は必ずエラーとなっている
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(TwoFloat::from_str_radix(str, radix)?))
    }
}

impl FromStr for F106 {
    type Err = TwoFloatError;

    /// twofloat v0.8.4 は from_str 未実装
    /// from_strMD<128>を経由してパースする
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let md = MD::<128>::from_str(s)
            .map_err(|_| TwoFloatError::ParseError)?;

        // 上位 f64 はそのまま変換し、 下位 f64 は元の値から引き算して求める
        let hi = md.to_f64();
        let md_hi = MD::<128>::from_f64(hi);
        let lo = (md - md_hi).to_f64();

        Ok(Self::from(TwoFloat::new_add(hi, lo)))
    }
}

mod consts {
    use super::*;
    pub const E: F106               = F106 { inner: twofloat::consts::E };
    pub const FRAC_1_PI: F106       = F106 { inner: twofloat::consts::FRAC_1_PI };
    pub const FRAC_1_SQRT_2: F106   = F106 { inner: twofloat::consts::FRAC_1_SQRT_2 };
    pub const FRAC_2_PI: F106       = F106 { inner: twofloat::consts::FRAC_2_PI };
    pub const FRAC_2_SQRT_PI: F106  = F106 { inner: twofloat::consts::FRAC_2_SQRT_PI };
    pub const FRAC_PI_2: F106       = F106 { inner: twofloat::consts::FRAC_PI_2 };
    pub const FRAC_PI_3: F106       = F106 { inner: twofloat::consts::FRAC_PI_3 };
    pub const FRAC_PI_4: F106       = F106 { inner: twofloat::consts::FRAC_PI_4 };
    pub const FRAC_PI_6: F106       = F106 { inner: twofloat::consts::FRAC_PI_6 };
    pub const FRAC_PI_8: F106       = F106 { inner: twofloat::consts::FRAC_PI_8 };
    pub const LN_2: F106            = F106 { inner: twofloat::consts::LN_2 };
    pub const LN_10: F106           = F106 { inner: twofloat::consts::LN_10 };
    pub const LOG10_2: F106         = F106 { inner: twofloat::consts::LOG10_2 };
    pub const LOG10_E: F106         = F106 { inner: twofloat::consts::LOG10_E };
    pub const LOG2_10: F106         = F106 { inner: twofloat::consts::LOG2_10 };
    pub const LOG2_E: F106          = F106 { inner: twofloat::consts::LOG2_E };
    pub const PI: F106              = F106 { inner: twofloat::consts::PI };
    pub const SQRT_2: F106          = F106 { inner: twofloat::consts::SQRT_2 };
    pub const TAU: F106             = F106 { inner: twofloat::consts::TAU };
}

impl Real for F106 {
    // Basic
    fn from_f64(v: f64) -> Self {
        Self { inner: TwoFloat::from_f64(v) }
    }

    fn to_i32(&self) -> i32 {
        self.inner.to_i32()
            .unwrap_or({
                let max: TwoFloat = TwoFloat::from(i32::MAX);
                let min: TwoFloat = TwoFloat::from(i32::MIN);
                let trunced = self.inner.trunc();
                if trunced > max {
                    i32::MAX
                } else if trunced < min {
                    i32::MIN
                } else {
                    trunced.to_i32().unwrap()
                }
            })
    }

    fn is_i32_compatible(&self) -> bool {
        self.inner.to_i32().is_some()
    }

    fn fract(self) -> Self {
        Self::from(self.inner.fract())
    }

    fn trunc(self) -> Self {
        Self::from(self.inner.trunc())
    }

    // Constatns
    fn e()              -> Self { consts::E }
    fn frac_1_pi()      -> Self { consts::FRAC_1_PI }
    fn frac_1_sqrt_2()  -> Self { consts::FRAC_1_SQRT_2 }
    fn frac_2_pi()      -> Self { consts::FRAC_2_PI }
    fn frac_2_sqrt_pi() -> Self { consts::FRAC_2_SQRT_PI }
    fn frac_pi_2()      -> Self { consts::FRAC_PI_2 }
    fn frac_pi_3()      -> Self { consts::FRAC_PI_3 }
    fn frac_pi_4()      -> Self { consts::FRAC_PI_4 }
    fn frac_pi_6()      -> Self { consts::FRAC_PI_6 }
    fn frac_pi_8()      -> Self { consts::FRAC_PI_8 }
    fn ln_2()           -> Self { consts::LN_2 }
    fn ln_10()          -> Self { consts::LN_10 }
    fn log2_10()        -> Self { consts::LOG2_10 }
    fn log2_e()         -> Self { consts::LOG2_E }
    fn log10_2()        -> Self { consts::LOG10_2 }
    fn log10_e()        -> Self { consts::LOG10_E }
    fn pi()             -> Self { consts::PI }
    fn sqrt_2()         -> Self { consts::SQRT_2 }
    fn tau()            -> Self { consts::TAU }

    // Trigonometric functions
    fn sin(self) -> Self {
        Self::from(self.inner.sin())
    }
    fn cos(self) -> Self {
        Self::from(self.inner.cos())
    }
    fn tan(self) -> Self {
        Self::from(self.inner.tan())
    }
    fn asin(self) -> Self {
        Self::from(self.inner.asin())
    }
    fn acos(self) -> Self {
        Self::from(self.inner.acos())
    }
    fn atan(self) -> Self {
        Self::from(self.inner.atan())
    }
    fn atan2(self, other: Self) -> Self {
        Self::from(self.inner.atan2(other.inner))
    }
    fn sin_cos(self) -> (Self, Self) {
        let (sin, cos) = self.inner.sin_cos();
        (Self::from(sin), Self::from(cos))
    }

    // Hyperbolic functions
    fn sinh(self) -> Self {
        Self::from(self.inner.sinh())
    }
    fn cosh(self) -> Self {
        Self::from(self.inner.cosh())
    }
    fn tanh(self) -> Self {
        Self::from(self.inner.tanh())
    }
    fn asinh(self) -> Self {
        Self::from(self.inner.asinh())
    }
    fn acosh(self) -> Self {
        Self::from(self.inner.acosh())
    }
    fn atanh(self) -> Self {
        Self::from(self.inner.atanh())
    }

    // Exponential and Logarithmic
    fn exp(self) -> Self {
        Self::from(self.inner.exp())
    }
    fn ln(self) -> Self {
        Self::from(self.inner.ln())
    }
    fn log10(self) -> Self {
        Self::from(self.inner.log10())
    }

    // Others
    fn sqrt(self) -> Self {
        Self::from(self.inner.sqrt())
    }
    fn abs(self) -> Self {
        Self::from(self.inner.abs())
    }
    fn hypot(self, other: Self) -> Self {
        Self::from(self.inner.hypot(other.inner))
    }

    // Power
    fn pow(self, rhs: Self) -> Self {
        Self::from(self.inner.powf(rhs.inner))
    }
    fn powi(self, n: i32) -> Self {
        Self::from(self.inner.powi(n))
    }
}
