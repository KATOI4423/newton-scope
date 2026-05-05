//! f128.rs
//!
//! f128 による多倍長浮動小数点のラッパー定義

use f128::f128;
use formulac::core::Real;
use num_traits::{
    Float,
    FloatConst,
    Num,
    One,
    ToPrimitive,
    Zero,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use serde::de::{
    self,
    Visitor,
};
use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
    Rem, RemAssign,
    Neg,
};
use std::str::FromStr;

/// f128 のラッパー構造体
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct F128 {
    value: f128,
}

impl F128 {
    pub(crate) fn to_f64(&self) -> f64 {
        self.value.to_f64().unwrap()
    }
}

impl Serialize for F128 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // f128 の Display は出力桁数が不明確なため、 LowerExp を使う
        // roundtrip 保証桁数は $ceil(113 * log10(2)) + 2 = 36$ となる (10進 <-> 2進の変換の丸め誤差を吸収するために + 2 する必要がある)
        let s = format!("{:.36e}", self);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for F128 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct F128Visitor;

        impl<'de> Visitor<'de> for F128Visitor {
            type Value = F128;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string representing an f128 value")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<F128, E> {
                f128::parse(v)
                .map(|value| F128 { value })
                    .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(F128Visitor)
    }
}

impl<T: Into<f128>> From<T> for F128 {
    fn from(value: T) -> Self {
        Self { value: value.into() }
    }
}

impl std::fmt::Display for F128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::fmt::LowerExp for F128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.value;
        let (abs_v, sign) = if v.is_sign_negative() {
            ( -v, "-" )
        } else if f.sign_plus() {
            ( v, "+" )
        } else {
            ( v, "" )
        };
        if v.is_infinite() {
            return write!(f, "{sign}inf");
        }
        if v.is_nan() {
            return write!(f, "NaN");
        }

        let prec = f.precision().unwrap_or(33); // f128は約34桁
        if v == f128::ZERO || v == -f128::ZERO {
            return write!(f, "{sign}0.{:0<prec$}e+00", "");
        }

        // 桁数を求めるために、f64で指数だけ求める
        let abs_f64: f64 = abs_v.into();
        let exp = abs_f64.abs().log10().floor() as isize;

        // f128のまま仮数部を計算
        let scale = pow10_f128(exp);
        let mantissa = abs_v / scale; // 1.0 <= mantissa < 10.0

        let digits = extract_digits_f128(mantissa, prec + 1);

        // 丸め後に繰り上がりで桁数が変わる場合の補正
        let (digits, exp) = if digits.len() > prec + 1 {
            // 繰り上がる場合 (ex: 9.999... -> 10.000...)
            (digits[1..prec + 2].to_string(), exp + 1)
        } else {
            (digits, exp)
        };

        let head = &digits[..1];
        let frac = &digits[1..];

        write!(f, "{sign}{head}.{frac}e{exp:+03}")
    }
}

/// 10^exp を f128 で計算
fn pow10_f128(exp: isize) -> f128 {
    let base = f128::from(10u32);
    if exp >= 0 {
        let mut result = f128::ONE;
        for _ in 0..exp {
            result *= base;
        }
        result
    } else {
        let mut result = f128::ONE;
        for _ in 0..(-exp) {
            result /= base;
        }
        result
    }
}

/// f128 の仮数部から prec+1 桁の数字列を抽出
fn extract_digits_f128(mantissa: f128, n: usize) -> String {
    let ten = f128::from(10u32);
    let mut result = String::with_capacity(n + 1);
    let mut x = mantissa;
    for _ in 0..n {
        let digit = x.trunc();
        let d = digit.to_u8().unwrap();
        result.push((b'0' + d.min(9)) as char);
        x = (x - digit) * ten;
    }
    // 四捨五入する
    let next = x.trunc().to_f64().unwrap() as u8;
    if next >= 5 {
        round_up_digits(&mut result);
    }
    result
}

fn round_up_digits(s: &mut String) {
    let bytes = unsafe { s.as_bytes_mut() };
    let mut i = bytes.len();
    loop {
        if i == 0 {
            // 全桁繰り上がり: ex) "999" --> "1000"
            s.insert(0, '1');
            // 末尾の '0' は呼び出し元でスライスするので不要
            break;
        }
        i -= 1;
        if bytes[i] < b'9' {
            bytes[i] += 1;
            break;
        } else {
            bytes[i] = b'0';
        }
    }
}

impl AddAssign for F128 {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}
impl SubAssign for F128 {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}
impl MulAssign for F128 {
    fn mul_assign(&mut self, rhs: Self) {
        self.value *= rhs.value;
    }
}
impl DivAssign for F128 {
    fn div_assign(&mut self, rhs: Self) {
        self.value /= rhs.value;
    }
}
impl RemAssign for F128 {
    fn rem_assign(&mut self, rhs: Self) {
        self.value %= rhs.value;
    }
}

impl Add for F128 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}
impl Sub for F128 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}
impl Mul for F128 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}
impl Div for F128 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}
impl Rem for F128 {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}
impl Neg for F128 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

impl Zero for F128 {
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
    fn set_zero(&mut self) {
        self.value.set_zero();
    }
    fn zero() -> Self {
        Self { value: f128::ZERO }
    }
}

impl One for F128 {
    fn is_one(&self) -> bool {
        self.value.is_one()
    }
    fn set_one(&mut self) {
        self.value.set_one();
    }
    fn one() -> Self {
        Self { value: f128::ONE }
    }
}

impl Num for F128 {
    type FromStrRadixErr = ();
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        // TODO: f128::from_str_radix が `unimplemented!` なので使えない
        Ok(Self { value: f128::from_str_radix(str, radix)? })
    }
}

impl FromStr for F128 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

impl Real for F128 {
    // Basic
    fn from_f64(v: f64) -> Self {
        v.to_f64().unwrap().into()
    }
    fn to_i32(&self) -> i32 {
        if let Some(v) = self.value.to_i32() {
            v
        } else {
            let max = f128::from(i32::MAX);
            let min = f128::from(i32::MIN);
            if self.value >= max {
                i32::MAX
            } else if self.value <= min {
                i32::MIN
            } else {
                self.value.trunc().to_i32().unwrap()
            }
        }
    }
    fn is_i32_compatible(&self) -> bool {
        self.value.to_i32().is_some()
    }
    fn fract(self) -> Self {
        self.value.fract().into()
    }
    fn trunc(self) -> Self {
        self.value.trunc().into()
    }

    // Constants
    fn e() -> Self {
        const E: F128 = F128 { value: f128::E };
        E
    }
    fn frac_1_pi() -> Self {
        f128::FRAC_1_PI().into()
    }
    fn frac_1_sqrt_2() -> Self {
        f128::FRAC_1_SQRT_2().into()
    }
    fn frac_2_pi() -> Self {
        f128::FRAC_2_PI().into()
    }
    fn frac_2_sqrt_pi() -> Self {
        f128::FRAC_2_SQRT_PI().into()
    }
    fn frac_pi_2() -> Self {
        f128::FRAC_PI_2().into()
    }
    fn frac_pi_3() -> Self {
        f128::FRAC_PI_3().into()
    }
    fn frac_pi_4() -> Self {
        f128::FRAC_PI_4().into()
    }
    fn frac_pi_6() -> Self {
        f128::FRAC_PI_6().into()
    }
    fn frac_pi_8() -> Self {
        f128::FRAC_PI_8().into()
    }
    fn ln_10() -> Self {
        f128::LN_10().into()
    }
    fn ln_2() -> Self {
        f128::LN_2().into()
    }
    fn log10_e() -> Self {
        f128::LOG10_E().into()
    }
    fn log10_2() -> Self {
        f128::LOG10_2().into()
    }
    fn log2_10() -> Self {
        f128::LOG2_10().into()
    }
    fn log2_e() -> Self {
        f128::LOG2_E().into()
    }
    fn pi() -> Self {
        const PI: F128 = F128 { value: f128::PI };
        PI
    }
    fn sqrt_2() -> Self {
        f128::SQRT_2().into()
    }
    fn tau() -> Self {
        f128::TAU().into()
    }

    // Trigonometric functions
    fn sin(self) -> Self {
        self.value.sin().into()
    }
    fn cos(self) -> Self {
        self.value.cos().into()
    }
    fn tan(self) -> Self {
        self.value.tan().into()
    }
    fn asin(self) -> Self {
        self.value.asin().into()
    }
    fn acos(self) -> Self {
        self.value.acos().into()
    }
    fn atan(self) -> Self {
        self.value.atan().into()
    }
    fn atan2(self, other: Self) -> Self {
        self.value.atan2(other.value).into()
    }
    fn sin_cos(self) -> (Self, Self) {
        let (sin, cos) = self.value.sin_cos();
        (sin.into(), cos.into())
    }

    // Hypoerbolic function
    fn sinh(self) -> Self {
        self.value.sinh().into()
    }
    fn cosh(self) -> Self {
        self.value.cosh().into()
    }
    fn tanh(self) -> Self {
        self.value.tanh().into()
    }
    fn asinh(self) -> Self {
        self.value.asinh().into()
    }
    fn acosh(self) -> Self {
        self.value.acosh().into()
    }
    fn atanh(self) -> Self {
        self.value.atanh().into()
    }

    // Exponential and Logarithmic
    fn exp(self) -> Self {
        self.value.exp().into()
    }
    fn ln(self) -> Self {
        self.value.ln().into()
    }
    fn log10(self) -> Self {
        self.value.log10().into()
    }

    // Others
    fn sqrt(self) -> Self {
        self.value.sqrt().into()
    }
    fn abs(self) -> Self {
        self.value.abs().into()
    }
    fn hypot(self, other: Self) -> Self {
        self.value.hypot(other.value).into()
    }

    // Power
    fn pow(self, rhs: Self) -> Self {
        self.value.powf(rhs.value).into()
    }
    fn powi(self, n: i32) -> Self {
        self.value.powi(n).into()
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn roundtrip_string() {
        let original = F128::pi();
        let json = serde_json::to_string(&original).unwrap();
        let restored: F128 = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}

#[cfg(test)]
mod tests_lower_exp {
    use super::*;

    fn fmt(v: F128) -> String {
        format!("{:e}", v)
    }
    fn fmt_prec(v: F128, prec: usize) -> String {
        format!("{:.prec$e}", v)
    }
    fn fmt_plus(v: F128) -> String {
        format!("{:+e}", v)
    }

    // ── 特殊値 ────────────────────────────────────────────────

    #[test]
    fn inf_positive() {
        let v = F128::from_f64(f64::INFINITY);
        assert_eq!(fmt(v), "inf");
    }

    #[test]
    fn inf_negative() {
        let v = F128::from_f64(f64::NEG_INFINITY);
        assert_eq!(fmt(v), "-inf");
    }

    #[test]
    fn inf_positive_sign_plus() {
        let v = F128::from_f64(f64::INFINITY);
        assert_eq!(fmt_plus(v), "+inf");
    }

    #[test]
    fn inf_negative_sign_plus() {
        // 負の無限大は sign_plus に関係なく "-"
        let v = F128::from_f64(f64::NEG_INFINITY);
        assert_eq!(fmt_plus(v), "-inf");
    }

    // ── ゼロ ──────────────────────────────────────────────────

    #[test]
    fn zero() {
        let v = F128::from_f64(0.0);
        let s = fmt(v);
        assert!(s.starts_with("0."), "zero: {}", s);
        assert!(s.ends_with("e+00"), "zero exponent: {}", s);
    }

    #[test]
    fn zero_sign_plus() {
        let v = F128::from_f64(0.0);
        let s = fmt_plus(v);
        assert!(s.starts_with("+0."), "zero sign_plus: {}", s);
    }

    #[test]
    fn zero_precision() {
        let v = F128::from_f64(0.0);
        let s = fmt_prec(v, 5);
        assert_eq!(s, "0.00000e+00", "zero prec=5: {}", s);
    }

    // ── 符号 ──────────────────────────────────────────────────

    #[test]
    fn positive_no_sign() {
        let v = F128::from_f64(1.0);
        let s = fmt(v);
        assert!(!s.starts_with('+') && !s.starts_with('-'), "positive: {}", s);
    }

    #[test]
    fn positive_sign_plus() {
        let v = F128::from_f64(1.0);
        assert!(fmt_plus(v).starts_with('+'));
    }

    #[test]
    fn negative_sign() {
        let v = F128::from_f64(-1.0);
        assert!(fmt(v).starts_with('-'));
    }

    #[test]
    fn negative_sign_plus() {
        // 負数は sign_plus に関係なく "-"
        let v = F128::from_f64(-1.0);
        assert!(fmt_plus(v).starts_with('-'));
    }

    // ── フォーマット構造 ───────────────────────────────────────

    #[test]
    fn format_structure() {
        // "d.dddde±NN" の形であることを確認
        let v = F128::from_f64(1.0);
        let s = fmt(v);
        let e_pos = s.find('e').expect("must contain 'e'");
        // 先頭が数字
        assert!(s.chars().next().unwrap().is_ascii_digit(), "first char: {}", s);
        // 2文字目が '.'
        assert_eq!(s.chars().nth(1).unwrap(), '.', "dot: {}", s);
        // 'e' の後が '+' か '-'
        let after_e = &s[e_pos + 1..];
        assert!(
            after_e.starts_with('+') || after_e.starts_with('-'),
            "exp sign: {}",
            s
        );
    }

    #[test]
    fn exponent_width_two_digits_minimum() {
        // 指数部は最低2桁 (e+00, e+01, e-01 など)
        let v = F128::from_f64(1.0);
        let s = fmt(v);
        let e_pos = s.find('e').unwrap();
        let exp_digits = &s[e_pos + 2..]; // '+' or '-' の後
        assert!(exp_digits.len() >= 2, "exp digits width: {}", s);
    }

    // ── precision 指定 ─────────────────────────────────────────

    #[test]
    fn precision_0() {
        let v = F128::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 0);
        // "3.e+00" または "3e+00" — 小数部が空
        let e_pos = s.find('e').unwrap();
        let dot_pos = s.find('.');
        if let Some(d) = dot_pos {
            assert_eq!(e_pos - d - 1, 0, "prec=0 frac empty: {}", s);
        }
    }

    #[test]
    fn precision_5_digit_count() {
        let v = F128::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 5);
        let dot = s.find('.').unwrap();
        let e   = s.find('e').unwrap();
        assert_eq!(e - dot - 1, 5, "prec=5 digit count: {}", s);
    }

    #[test]
    fn precision_5_value() {
        let v = F128::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 5);
        // "3.14159e+00" であること
        assert!(s.starts_with("3.14159"), "prec=5 value: {}", s);
    }

    #[test]
    fn precision_15_digit_count() {
        let v = F128::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 15);
        let dot = s.find('.').unwrap();
        let e   = s.find('e').unwrap();
        assert_eq!(e - dot - 1, 15, "prec=15 digit count: {}", s);
    }

    // ── 既知定数の精度検証 ─────────────────────────────────────

    /// 数字列だけ抽出（ピリオドと e より後を除去）
    fn mantissa_digits(s: &str) -> String {
        s.chars()
            .take_while(|&c| c != 'e' && c != 'E')
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    fn check_digits(got: &str, expected_digits: &str, check_len: usize) {
        let g = mantissa_digits(got);
        let e = mantissa_digits(expected_digits);
        let n = check_len.min(g.len()).min(e.len());
        assert_eq!(
            &g[..n], &e[..n],
            "\nActual:   {}\nExpected: {}", got, expected_digits
        );
    }

    #[test]
    fn pi_digits() {
        let pi = F128::pi();
        let s  = fmt_prec(pi, 33);
        // π の正しい桁: 3.14159265358979323846264338327950288...
        check_digits(&s, "3.14159265358979323846264338327950288e+00", 33);
    }

    #[test]
    fn e_digits() {
        let e = F128::e();
        let s = fmt_prec(e, 33);
        // e = 2.71828182845904523536028747135266249...
        check_digits(&s, "2.71828182845904523536028747135266249e+00", 33);
    }

    #[test]
    fn sqrt2_digits() {
        let s2 = F128::sqrt_2();
        let s  = fmt_prec(s2, 33);
        // √2 = 1.41421356237309504880168872420969807...
        check_digits(&s, "1.41421356237309504880168872420969807e+00", 33);
    }

    #[test]
    fn ln2_digits() {
        let ln2 = F128::ln_2();
        let s   = fmt_prec(ln2, 33);
        // ln2 = 6.93147180559945309417232121458176568e-01
        check_digits(&s, "6.93147180559945309417232121458176568e-01", 33);
    }

    // ── 指数スケール ───────────────────────────────────────────

    #[test]
    fn one() {
        let s = fmt(F128::from_f64(1.0));
        let e_pos = s.find('e').unwrap();
        assert_eq!(&s[e_pos..e_pos + 4], "e+00", "1.0 exponent: {}", s);
    }

    #[test]
    fn large_exponent() {
        let v = F128::from_f64(1.23e50);
        let s = fmt(v);
        assert!(s.contains("e+50"), "1.23e50: {}", s);
    }

    #[test]
    fn small_exponent() {
        let v = F128::from_f64(1.23e-50);
        let s = fmt(v);
        assert!(s.contains("e-50"), "1.23e-50: {}", s);
    }

    #[test]
    fn negative_pi() {
        let v = -F128::pi();
        let s = fmt_prec(v, 10);
        // 3.14159265358.. -> 3.1415926536
        assert!(s.starts_with("-3.1415926536"), "negative pi: {}", s);
    }

    // ── ラウンドトリップ ───────────────────────────────────────

    #[test]
    fn roundtrip_f64() {
        let cases = [1.0_f64, -1.0, 0.5, 100.0, 1e-10, 1e10, 1.23456789];
        for &v in &cases {
            let md  = F128::from_f64(v);
            let s   = fmt_prec(md, 15);
            let got = s.parse::<f64>().unwrap();
            let tol = v.abs() * 1e-14 + 1e-300;
            assert!(
                (got - v).abs() < tol,
                "roundtrip failed for {}: s={}, got={}", v, s, got
            );
        }
    }
}
