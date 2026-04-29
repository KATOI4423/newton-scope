//! dashu.rs
//!
//! dashu_float による多倍長浮動小数点演算のラッパー定義

use std::collections::HashMap;
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
use std::sync::{
    OnceLock,
    RwLock,
};
use dashu::integer::IBig;
use dashu_float::{
    Context,
    FBig,
};
use dashu_float::round::mode::HalfAway;
use formulac::core::Real;
use num_traits::{
    FromPrimitive,
    Num,
    One,
    Signed,
    ToPrimitive,
    Zero,
};
use serde::{
    Deserialize,
    Serialize,
};

/// dashu_float のラッパー構造体
///
/// # Generics
///  - N: Context の presicion bit数
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct MD<const N: usize> {
    value: FBig<HalfAway, 2>,
}

impl <const N: usize> MD<N> {
    fn ctx() -> Context<HalfAway> {
        Context::<HalfAway>::new(N)
    }
}

// e のキャッシュをグローバルに保持 (N: 精度 -> Fbig: 計算結果)
static E_CACHE: OnceLock<RwLock<HashMap<usize, FBig<HalfAway, 2>>>> = OnceLock::new();
// π のキャッシュをグローバルに保持 (N: 精度 -> Fbig: 計算結果)
static PI_CACHE: OnceLock<RwLock<HashMap<usize, FBig<HalfAway, 2>>>> = OnceLock::new();
// Ln(2) のキャッシュをグローバルに保持 (N: 精度 -> Fbig: 計算結果)
static LN2_CACHE: OnceLock<RwLock<HashMap<usize, FBig<HalfAway, 2>>>> = OnceLock::new();
// Ln(10) のキャッシュをグローバルに保持 (N: 精度 -> Fbig: 計算結果)
static LN10_CACHE: OnceLock<RwLock<HashMap<usize, FBig<HalfAway, 2>>>> = OnceLock::new();
// sqrt(2) のキャッシュをグローバルに保持 (N: 精度 -> Fbig: 計算結果)
static SQRT2_CACHE: OnceLock<RwLock<HashMap<usize, FBig<HalfAway, 2>>>> = OnceLock::new();

/// chudnovsky のアルゴリズムにより pi を計算する
impl <const N: usize> MD<N> {
    /// Chudnovsky の アルゴリズムにより pi を計算する
    ///
    /// # Args
    /// - terms: 計算に使用する項数. 1項あたり約14.18桁収束.
    ///
    /// # Formula
    /// ```text, ignore
    ///   1       12       ∞ (-1)^k * (6k)! * (A + B*k)
    /// ---- = -------- *  Σ -----------------------------,     A = 13591409, B = 545140134, C = 640320
    ///   pi     C^(3/2)   k=0  (3k)! * (k!)^3 * C^(3k)
    /// ```
    fn chudnovsky(terms: usize) -> Self {
        // a_0 = A / 1
        let a0_t = Self::from(13591409);
        let a0_q = Self::one();

        let (t, q) = Self::chudnovsky_split(0, terms, a0_t, a0_q);

        Self::from(426880) * Self::from(10005).sqrt() * q / t
    }

    /// chudnovsky 級数の Binary Spitting を行う.
    ///
    /// # 概要
    ///
    /// chudnovsky 級数の各項 a_k は以下の式で定義される:
    /// ``` text, ignore
    ///        (-1)^k * (6k)! * (A + B*k)
    /// a_k = ----------------------------, A = A = 13591409, B = 545140134, C = 640320
    ///          (3k)! * (k!)^3 * C^(3k)
    /// ```
    ///
    /// この関数は区間[a, b) の総和 Σ a_k を、分子 T, 分母 Q の形で返す:
    /// ```text, ignore
    ///  T     b-1
    /// --- =   Σ a_k
    ///  Q     k=a
    /// ```
    ///
    /// # Returns
    /// - (T, Q): T/Q = Σ_{k=a}^{b-1} a_k
    fn chudnovsky_split(a: usize, b: usize, a_t: Self, a_q: Self) -> (Self, Self) {
        if b - a == 1 {
            // 基底をそのまま返す
            return (a_t, a_q);
        }

        let m = (a + b) / 2;
        let (m_t, m_q) = Self::ak_advance(a_t.clone(), a_q.clone(), a, m);
        let (t1, q1) = Self::chudnovsky_split(a, m, a_t, a_q);
        let (t2, q2) = Self::chudnovsky_split(m, b, m_t, m_q);

        // T/Q = T1/Q1 + T2/Q2 = (T1*Q2 + T2*Q1) / (Q1*Q2)
        (&t1 * &q2 + &t2 * &q1, q1 * q2)
    }


    /// a_k / a_{k-1} の差分分子: (6k-5)(6k-4)...(6k) * (A + B*k)
    fn ak_diff_num(k: usize) -> Self {
        let mut result = Self::from(13591409) + Self::from(545140134) * Self::from(k);
        for i in (6*k-5)..=(6*k) {
            result *= Self::from(i);
        }
        result
    }

    /// a_k / a_{k-1} の差分分母: (3k)(3k-1)(3k-2) * (k^3) * (C^3) * (A + B*(k-1))
    fn ak_diff_den(k: usize) -> Self {
        let k_md = Self::from(k);
        let c3 = Self::from(262537412640768000u64);
        let a = Self::from(13591409);
        let b = Self::from(545140134);
        let mut result =  a + b * Self::from(k-1);
        result *= &k_md * &k_md * k_md * c3;
        for i in (3*k-2)..=(3*k) {
            result *= Self::from(i);
        }
        result
    }

    /// a_{from_k} から a_{to_k} を差分で前進させる
    ///
    /// # Returns
    /// - (T, Q): a_{to_k} の (符号込み分子、分母)
    fn ak_advance(mut t: Self, mut q: Self, from_k: usize, to_k: usize) -> (Self, Self) {
        for i in (from_k + 1)..=to_k {
            t *= -Self::ak_diff_num(i);
            q *= Self::ak_diff_den(i);
        }
        (t, q)
    }
}

/// 三角関数を求めるためのヘルパー関数群
impl <const N: usize> MD<N> {
    // 必要反復数: |x| <= π/8 ~ 0.393
    // x^(2n+1) / (2n+1)! < 10^(-N*log10(2)) となる n
    // => n ~ N * log10(2) / (2 * log10(8/π)) ~ N * 0.87
    const fn iterations() -> usize {
        // N * 0.87 の切り上げを整数演算でエミュレート
        (N * 87 + 99) / 100 + 4 // +4 程度の余裕を持たせる
    }

    /// sin と cos を Taylor 級数で同時に求める
    fn sincos_taylor(&self) -> (Self, Self) {
        let x2 = self * self;

        let mut sin_sum = self.clone();
        let mut cos_sum = Self::one();
        let mut sin_term = self.clone();
        let mut cos_term = Self::one();

        for k in 1..=Self::iterations() {
            let k2m = Self::from(2*k-1);
            let k2 = Self::from(2*k);
            let k2p = Self::from(2*k+1);

            // sin の次項: term *= -x^2 / (2k * (2k+1))
            sin_term *= -(&x2 / &(&k2 * &k2p));

            // cos の次項: term *= -x^2 / ((2k-1) * 2k)
            cos_term *= -(&x2 / &(&k2m * &k2));

            sin_sum += &sin_term;
            cos_sum += &cos_term;

            // 収束判定: 項が精度限界以下になったら打ち切る
            if sin_term.is_zero() && cos_term.is_zero() {
                break;
            }
        }

        (sin_sum, cos_sum)
    }

    /// |x| <= tan(π/8) での atan Taylor 級数.
    /// atan(x) = x - x^3 / 3 + x^5 / 5 - x^7 / 7 + ...
    fn atan_taylor(&self) -> Self {
        let x2 = self * &self;
        let mut sum = self.clone();
        let mut term = self.clone(); // k=0 の項: x

        for k in 1..=Self::iterations() {
            term *= -(&x2 * &Self::from(2*k-1) / Self::from(2*k+1));
            sum += &term;
            if term.is_zero() {
                break;
            }
        }
        sum
    }
}

impl <const N: usize, const B: u64> From<FBig<HalfAway, B>> for MD<N> {
    fn from(value: FBig<HalfAway, B>) -> Self {
        let fbig2 = value.with_rounding::<HalfAway>().with_base::<2>().value();
        let normalized = Self::ctx().add(fbig2.repr(), FBig::<HalfAway>::ZERO.repr()).value();
        Self { value: normalized }
    }
}
impl <const N: usize> From<usize> for MD<N> {
    fn from(value: usize) -> Self {
        FBig::<HalfAway>::from_usize(value).unwrap().into()
    }
}
impl <const N: usize> From<u64> for MD<N> {
    fn from(value: u64) -> Self {
        FBig::<HalfAway>::from_u64(value).unwrap().into()
    }
}
impl <const N: usize> From<u32> for MD<N> {
    fn from(value: u32) -> Self {
        FBig::<HalfAway>::from_u32(value).unwrap().into()
    }
}
impl <const N: usize> From<u16> for MD<N> {
    fn from(value: u16) -> Self {
        FBig::<HalfAway>::from_u16(value).unwrap().into()
    }
}
impl <const N: usize> From<u8> for MD<N> {
    fn from(value: u8) -> Self {
        FBig::<HalfAway>::from_u8(value).unwrap().into()
    }
}
impl <const N: usize> From<i64> for MD<N> {
    fn from(value: i64) -> Self {
        FBig::<HalfAway>::from_i64(value).unwrap().into()
    }
}
impl <const N: usize> From<i32> for MD<N> {
    fn from(value: i32) -> Self {
        FBig::<HalfAway>::from_i32(value).unwrap().into()
    }
}
impl <const N: usize> From<i16> for MD<N> {
    fn from(value: i16) -> Self {
        FBig::<HalfAway>::from_i16(value).unwrap().into()
    }
}
impl <const N: usize> From<i8> for MD<N> {
    fn from(value: i8) -> Self {
        FBig::<HalfAway>::from_i8(value).unwrap().into()
    }
}

fn sign_str<T>(v: &T, f: &std::fmt::Formatter<'_>) -> &'static str
where
    T: Signed,
{
    if v.is_negative() {
        "-"
    } else if f.sign_plus() {
        "+"
    } else {
        ""
    }
}

impl <const N: usize> std::fmt::Display for MD<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerExp::fmt(&self, f)
    }
}

impl <const N: usize> LowerExp for MD<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 特殊値
        let v = &self.value;
        let sign = sign_str(v, f);
        if v.repr().is_infinite() {
            return write!(f, "{sign}inf");
        }

        let prec = f.precision().unwrap_or((N as f64 * std::f64::consts::LOG10_2) as usize);
        if v.is_zero() {
            return write!(f, "{sign}0.{:0<prec$}e+00", "")
        }

        let dbig = v.to_decimal().value()
            .with_precision(prec + 1).value(); // with_precision は 全桁合わせた桁数. fmt の precision は小数点以下の桁数.
        let repr = dbig.repr();
        let exp10 = repr.exponent();

        let digits = repr.significand().abs().to_string();
        let len: isize = digits.len().try_into().map_err(|_| std::fmt::Error)?;

        let exp = exp10.checked_add(len - 1).ok_or(std::fmt::Error)?;
        // with_precision(prec) 後は digits.len() == prec が保証されるため、小数部は digits[1..] をそのまま使える
        // ただし、 prec == 0 の場合は frac が空になる
        let frac = &digits[1..];

        write!(f, "{sign}{head}.{frac}e{exp:+03}",
            head = &digits[..1],
        )
    }
}

impl <const N: usize> AddAssign for MD<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}
impl <const N: usize> AddAssign<&Self> for MD<N> {
    fn add_assign(&mut self, rhs: &Self) {
        self.value += &rhs.value;
    }
}

impl <const N: usize> SubAssign for MD<N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}
impl <const N: usize> SubAssign<&Self> for MD<N> {
    fn sub_assign(&mut self, rhs: &Self) {
        self.value -= &rhs.value;
    }
}

impl <const N: usize> MulAssign for MD<N> {
    fn mul_assign(&mut self, rhs: Self) {
        self.value *= rhs.value;
    }
}
impl <const N: usize> MulAssign<&Self> for MD<N> {
    fn mul_assign(&mut self, rhs: &Self) {
        self.value *= &rhs.value;
    }
}

impl <const N: usize> DivAssign for MD<N> {
    fn div_assign(&mut self, rhs: Self) {
        self.value /= rhs.value;
    }
}
impl <const N: usize> DivAssign<&Self> for MD<N> {
    fn div_assign(&mut self, rhs: &Self) {
        self.value /= &rhs.value;
    }
}

impl <const N: usize> RemAssign for MD<N> {
    fn rem_assign(&mut self, rhs: Self) {
        self.value %= rhs.value;
    }
}
impl <const N: usize> RemAssign<&Self> for MD<N> {
    fn rem_assign(&mut self, rhs: &Self) {
        self.value %= &rhs.value;
    }
}


impl <const N: usize> Add for MD<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::ctx().add(self.value.repr(), rhs.value.repr()).value().into()
    }
}
impl <const N: usize> Add for &MD<N> {
    type Output = MD<N>;
    fn add(self, rhs: Self) -> Self::Output {
        MD::<N>::ctx().add(self.value.repr(), rhs.value.repr()).value().into()
    }
}

impl <const N: usize> Sub for MD<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::ctx().sub(self.value.repr(), rhs.value.repr()).value().into()
    }
}
impl <const N: usize> Sub for &MD<N> {
    type Output = MD<N>;
    fn sub(self, rhs: Self) -> Self::Output {
        MD::<N>::ctx().sub(self.value.repr(), rhs.value.repr()).value().into()
    }
}

impl <const N: usize> Mul for MD<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::ctx().mul(self.value.repr(), rhs.value.repr()).value().into()
    }
}
impl <const N: usize> Mul for &MD<N> {
    type Output = MD<N>;
    fn mul(self, rhs: Self) -> Self::Output {
        MD::<N>::ctx().mul(self.value.repr(), rhs.value.repr()).value().into()
    }
}

impl <const N: usize> Div for MD<N> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::ctx().div(self.value.repr(), rhs.value.repr()).value().into()
    }
}
impl <const N: usize> Div for &MD<N> {
    type Output = MD<N>;
    fn div(self, rhs: Self) -> Self::Output {
        MD::<N>::ctx().div(self.value.repr(), rhs.value.repr()).value().into()
    }
}

impl <const N: usize> Rem for MD<N> {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        let rem = Self::ctx().rem(self.value.repr(), rhs.value.repr()).value();
        let mut res: Self = rem.into();
        // ユークリッド除法（常に正の余りを返す）に合わせる
        if res.value.is_negative() {
            let abs_rhs = rhs.value.abs();
            res.value += abs_rhs;
        }
        res
    }
}
impl <const N: usize> Rem for &MD<N> {
    type Output = MD<N>;
    fn rem(self, rhs: Self) -> Self::Output {
        let rem = MD::<N>::ctx().rem(self.value.repr(), rhs.value.repr()).value();
        let mut res = MD::<N>::from(rem);
        // ユークリッド除法（常に正の余りを返す）に合わせる
        if res.value.is_negative() {
            let abs_rhs = rhs.value.abs();
            res.value += abs_rhs;
        }
        res
    }
}

impl <const N: usize> Neg for MD<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.value.neg().into()
    }
}
impl <const N: usize> Neg for &MD<N> {
    type Output = MD::<N>;
    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl <const N: usize> One for MD<N> {
    fn is_one(&self) -> bool {
        self.value.is_one()
    }
    fn one() -> Self {
        FBig::<HalfAway>::one().into()
    }
    fn set_one(&mut self) {
        self.value.set_one();
    }
}

impl <const N: usize> Zero for MD<N> {
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
    fn zero() -> Self {
        FBig::<HalfAway>::zero().into()
    }
    fn set_zero(&mut self) {
        self.value.set_zero();
    }
}

impl <const N: usize> Num for MD<N> {
    type FromStrRadixErr = dashu_base::error::ParseError;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let dec = FBig::<HalfAway, 10>::from_str_radix(str, radix)?;
        Ok(Self::from(dec.with_rounding::<HalfAway>().with_base::<2>().value()))
    }
}
impl <const N: usize> FromStr for MD<N> {
    type Err = dashu_base::error::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

impl <const N: usize> Real for MD<N> {
    // Basic
    fn from_f64(v: f64) -> Self {
        let value = FBig::<HalfAway, 2>::from_f64(v).unwrap();
        value.into()
        // Self::ctx().with_precision(value.repr()).value().into()
    }
    fn to_i32(&self) -> i32 {
        if let Some(v) = self.value.to_i32() {
            v
        } else {
            let max = FBig::<HalfAway>::from_i32(i32::MAX).unwrap();
            let min = FBig::<HalfAway>::from_i32(i32::MIN).unwrap();
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
        let cache = E_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        if let Some(val) = cache.read().unwrap().get(&N) {
            return val.clone().into()
        }

        let new: Self = Self::one().exp().into();
        let mut map = cache.write().unwrap();
        map.insert(N, new.value.clone());

        new
    }
    fn frac_1_pi() -> Self {
        Self::one() / Self::pi()
    }
    fn frac_1_sqrt_2() -> Self {
        Self::one() / Self::sqrt_2()
    }
    fn frac_2_pi() -> Self {
        Self::from(2) / Self::pi()
    }
    fn frac_2_sqrt_pi() -> Self {
        Self::from(2) / Self::pi().sqrt()
    }
    fn frac_pi_2() -> Self {
        Self::pi() / Self::from(2)
    }
    fn frac_pi_3() -> Self {
        Self::pi() / Self::from(3)
    }
    fn frac_pi_4() -> Self {
        Self::pi() / Self::from(4)
    }
    fn frac_pi_6() -> Self {
        Self::pi() / Self::from(6)
    }
    fn frac_pi_8() -> Self {
        Self::pi() / Self::from(8)
    }
    fn ln_2() -> Self {
        let cache = LN2_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        if let Some(val) = cache.read().unwrap().get(&N) {
            return val.clone().into();
        }

        let val = Self::from(2).ln();

        let mut map = cache.write().unwrap();
        map.insert(N, val.value.clone());

        val
    }
    fn ln_10() -> Self {
        let cache = LN10_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        if let Some(val) = cache.read().unwrap().get(&N) {
            return val.clone().into();
        }

        let val = Self::from(10).ln();

        let mut map = cache.write().unwrap();
        map.insert(N, val.value.clone());

        val
    }
    fn log2_10() -> Self {
        Self::ln_10() / Self::ln_2()
    }
    fn log2_e() -> Self {
        Self::one() / Self::ln_2()
    }
    fn log10_2() -> Self {
        Self::ln_2() / Self::ln_10()
    }
    fn log10_e() -> Self {
        Self::one() / Self::ln_10()
    }
    fn pi() -> Self {
        let cache = PI_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        if let Some(val) = cache.read().unwrap().get(&N) {
            return val.clone().into();
        }

        // N bit -> 必要10進桁数
        let decimal_digits = (N as f64 * std::f64::consts::LOG10_2).ceil();
        // chudnovsky は 1項あたり約14.18桁収束
        const DIGITS_PER_TERM: f64 = 14.181647462;
        let terms = (decimal_digits / DIGITS_PER_TERM).ceil() as usize + 2; // +2項の余裕を持たせる
        let val = Self::chudnovsky(terms);

        let mut map = cache.write().unwrap();
        map.insert(N, val.value.clone());

        val
    }
    fn sqrt_2() -> Self {
        let cache = SQRT2_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        if let Some(val) = cache.read().unwrap().get(&N) {
            return val.clone().into();
        }

        let val = Self::from(2).sqrt();

        let mut map = cache.write().unwrap();
        map.insert(N, val.value.clone());

        val
    }
    fn tau() -> Self {
        Self::pi() * Self::from(2)
    }

    // Trigonometric functions
    fn sin(self) -> Self {
        let (s, _) = Self::sin_cos(self);
        s
    }

    fn cos(self) -> Self {
        let (_, c) = Self::sin_cos(self);
        c
    }

    fn tan(self) -> Self {
        let (s, c) = Self::sin_cos(self);
        s / c
    }

    fn asin(self) -> Self {
        let one = Self::one();
        // x±1 の特異点を先に処理する
        if self == one {
            return Self::frac_pi_2();
        } else if self == -&one {
            return -Self::frac_pi_2();
        }

        let x2 = &self * &self;
        // |x| が大きいときは、補角の公式により精度低下・ゼロ除算を避ける
        if self.value.abs() > Self::frac_1_sqrt_2().value {
            let next_x = (one - x2).sqrt();
            let res = Self::frac_pi_2() - next_x.asin();
            return if self.value.is_negative() { -res } else { res }
        }

        // asin(x) = atan(x / sqrt(1 - x^2))
        let denom = (one - x2).sqrt();
        (self / denom).atan()
    }

    fn acos(self) -> Self {
        // acos(x) = π/2 - asin(x)
        Self::frac_pi_2() - self.asin()
    }

    /// atan を Euler の変換公式 + Taylor 級数で計算する
    ///
    /// # Algorithm
    ///
    /// 1. 引数縮小:
    ///  - |x| > 1 なら atan(x) = π/2 - atan(1/x).
    ///  - |x| > tan(π/8) なら atan9x) = π/4 + atan((x-1)/(x+1)).
    /// これで |r| <= tan(π/8) ~ 0.414 に収める.
    ///
    /// 2. Taylor 級数: atan(r) = r - r^3 / 3 + r^5 / 5 - ...
    fn atan(self) -> Self {
        let one = Self::one();
        let two = Self::from(2);
        let pi_8 = Self::frac_pi_8();
        let pi_4 = &pi_8 * &two;
        let pi_2 = &pi_4 * &two;

        // 符号を分離
        let neg = self.value.is_negative();
        let x = if neg { -self } else { self };

        // |x| > 1: atan(x) = π/2 - atan(1/x)
        let (x, adjustment) = if x > one {
            (&one / &x, pi_2)
        } else {
            (x, Self::zero()) // 仮置き
        };

        // |x| > tan(π/8): atan(x) = π/4 + atan((x-1)/(x+1))
        let tan_pi_8 = &Self::sqrt_2() - &one; // tan(π/8) = √2 - 1
        let (r, offset) = if x > tan_pi_8 {
            let r = (&x - &one) / (&x + &one);
            (r, Some(pi_4))
        } else {
            (x, None)
        };

        // Taylor 展開
        let result = r.atan_taylor();

        // 補正を適応
        let result = if let Some(offset) = offset {
            offset + result
        } else {
            result
        };

        let result = if !adjustment.is_zero() {
            adjustment - result
        } else {
            result
        };

        if neg {
            -result
        } else {
            result
        }
    }

    fn atan2(self, other: Self) -> Self {
        let zero = Self::zero();
        let y = self;
        let x = other;
        let pi = Self::pi();
        let pi_2 = pi.clone() / Self::from(2);

        if x > zero {
            (y / x).atan()
        } else if x < zero {
            if y.value.is_negative() {
                (y / x).atan() - pi
            } else {
                (y / x).atan() + pi
            }
        } else {
            if y > zero {
                pi_2
            } else if y < zero {
                -pi_2
            } else {
                zero // 未定義
            }
        }
    }

    /// sin と cos を Tylor 級数で同時計算する
    ///
    /// # Algorithm
    ///
    /// 1. 引数縮小: x を [-π/4, π/4] に収める
    ///     x = 2^k * r となる k, r を求める.
    ///     sin(2x) = 2*sin(x)*cos(x), cos(2x) = 1 - 2*sin^2(x) で倍角復元.
    /// 2. Taylor 級数(|r| <= π/4 で高速収束)
    ///     sin(r) = r - r^3/3! + r^5/5! - ...,
    ///     cos(r) = 1 - r^2/2! + r^4/4! - ...,
    ///
    /// 3. 象限補正
    fn sin_cos(self) -> (Self, Self) {
        let two = Self::from(2);
        let pi = Self::pi();
        let tau = Self::tau();
        let pi_2 = Self::frac_pi_2();
        let pi_3_2 = &pi_2 * &Self::from(3);

        // x を [0, 2π) に正規化
        let x = {
            let ctx = Self::ctx();
            // x % 2π
            let v = &self.value;
            let q = ctx.div(v.repr(), tau.value.repr()).value();
            let q_floor = q.floor();
            let r = ctx.sub(
                v.repr(),
                ctx.mul(q_floor.repr(), tau.value.repr()).value().repr()
            ).value();
            Self::from(r)
        };

        // 象限判定
        let (reduced, quadrant) = if x < pi_2.clone() {
            (x, 0u8)
        } else if x < pi {
            (x - pi_2, 1)
        } else if x < pi_3_2 {
            (x - pi, 2)
        } else {
            (x - pi_3_2, 3)
        };

        // x を更に半分に畳み、Taylor 収束を加速させる
        let mut half_count = 0usize;
        let mut r = reduced;
        let threshold = Self::frac_pi_8();
        while r > threshold {
            r = Self::ctx().div(r.value.repr(), two.value.repr()).value().into();
            half_count += 1;
        }

        // Taylor 級数で計算
        let (mut sin, mut cos) = r.sincos_taylor();

        // 倍角公式で half_count 回復元
        for _ in 0..half_count {
            let new_sin = &(&two * &sin) * &cos;
            let new_cos = &cos * &cos - &sin * &sin;
            sin = new_sin;
            cos = new_cos;
        }

        // 象限補正
        match quadrant {
            0 => (sin, cos),
            1 => (cos, -sin),
            2 => (-sin, -cos),
            3 => (-cos, sin),
            _ => unreachable!(),
        }
    }

    // Hyperbolic function
    fn sinh(self) -> Self {
        (self.clone().exp() - (-self).exp()) / Self::from(2)
    }
    fn cosh(self) -> Self {
        (self.clone().exp() + (-self).exp()) / Self::from(2)
    }
    fn tanh(self) -> Self {
        let e_x = self.clone().exp();
        let e_mx = (-self).exp();
        (e_x.clone() - e_mx.clone()) / (e_x + e_mx)
    }
    fn asinh(self) -> Self {
        // ln(x + √(x^2 + 1))
        (&self + &(&self * &self + Self::one()).sqrt()).ln()
    }
    fn acosh(self) -> Self {
        // ln(x + √(x^2 - 1))
        (&self + &(&self * &self - Self::one()).sqrt()).ln()
    }

    fn atanh(self) -> Self {
        // 0.5 * ln((1 + x)/(1 - x))
        Self::from_f64(0.5) * ((&Self::one() + &self) / (&Self::one() - &self)).ln()
    }

    // Exponential and Logarithmic
    fn exp(self) -> Self {
        Self::ctx().exp(self.value.repr()).value().into()
    }
    fn ln(self) -> Self {
        Self::ctx().ln(self.value.repr()).value().into()
    }
    fn log10(self) -> Self {
        self.ln() / Self::ln_10()
    }

    // Others
    fn sqrt(self) -> Self {
        Self::ctx().sqrt(self.value.repr()).value().into()
    }
    fn abs(self) -> Self {
        self.value.abs().into()
    }
    fn hypot(self, other: Self) -> Self {
        (&self * &self + &other * &other).sqrt()
    }

    // Power
    fn pow(self, rhs: Self) -> Self {
        self.value.powf(&rhs.value).into()
    }
    fn powi(self, n: i32) -> Self {
        let n = IBig::from(n);
        self.value.powi(n).into()
    }
}

#[cfg(test)]
mod tests_chudnovsky {
    use super::*;
    use std::str::FromStr;

    // 既知のπ（100桁）
    const PI_STR: &str =
        "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";

    fn pi_ref<const N: usize>() -> MD<N> {
        MD::from_str(PI_STR).unwrap()
    }

    // 仮数部の数字のみ抽出して比較
    fn mantissa_digits(s: &str) -> String {
        s.chars()
            .take_while(|&c| c != 'e')
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    // 桁一致チェック（前方一致）
    fn assert_prefix_eq<const N: usize>(a: &MD<N>, b: &MD<N>, digits: usize) {
        let sa = format!("{:.1$e}", a, digits + 2);
        let sb = format!("{:.1$e}", b, digits + 2);
        let da = mantissa_digits(&sa);
        let db = mantissa_digits(&sb);
        let n = digits.min(da.len()).min(db.len());
        assert_eq!(&da[..n], &db[..n], "\nActual:   {}\nExpected: {}", sa, sb);
    }

    #[test]
    fn basic_accuracy() {
        let pi = MD::<128>::chudnovsky(5);
        let ref_pi = pi_ref::<128>();

        // 5項 ≈ 70桁精度だが、余裕見て30桁一致
        assert_prefix_eq(&pi, &ref_pi, 30);
    }

    #[test]
    fn convergence() {
        let pi_1 = MD::<128>::chudnovsky(1);
        let pi_3 = MD::<128>::chudnovsky(3);
        let ref_pi = pi_ref::<128>();

        let err_1 = (pi_1.clone() - ref_pi.clone()).abs();
        let err_3 = (pi_3.clone() - ref_pi.clone()).abs();

        assert!(err_3 < err_1);
    }

    #[test]
    fn high_precision() {
        let pi = MD::<256>::chudnovsky(8);
        let ref_pi = pi_ref::<256>();

        // 8項 → 約100桁以上
        assert_prefix_eq(&pi, &ref_pi, 60);
    }

    #[test]
    fn pi_inverse_consistency() {
        let pi = MD::<128>::chudnovsky(6);
        let one = MD::<128>::one();

        let inv_pi = one.clone() / pi.clone();
        let back = pi * inv_pi;

        let err = (back - one).abs();

        // 許容誤差（適当に小さく）
        let tol = MD::<128>::from_f64(1e-25);

        assert!(err < tol);
    }

    #[test]
    fn multiple_precision_levels() {
        let pi_64 = MD::<64>::chudnovsky(5);
        let pi_128 = MD::<128>::chudnovsky(5);

        let ref_pi_64 = pi_ref::<64>();
        let ref_pi_128 = pi_ref::<128>();

        // 精度低い方は少なめにチェック
        assert_prefix_eq(&pi_64, &ref_pi_64, 15);
        assert_prefix_eq(&pi_128, &ref_pi_128, 30);
    }
}

#[cfg(test)]
mod tests_lower_exp {
    use super::*;

    fn fmt(v: MD<113>) -> String {
        format!("{:e}", v)
    }
    fn fmt_prec(v: MD<113>, prec: usize) -> String {
        format!("{:.prec$e}", v)
    }
    fn fmt_plus(v: MD<113>) -> String {
        format!("{:+e}", v)
    }

    // ── 特殊値 ────────────────────────────────────────────────

    #[test]
    fn inf_positive() {
        let v = MD::<113>::from_f64(f64::INFINITY);
        assert_eq!(fmt(v), "inf");
    }

    #[test]
    fn inf_negative() {
        let v = MD::<113>::from_f64(f64::NEG_INFINITY);
        assert_eq!(fmt(v), "-inf");
    }

    #[test]
    fn inf_positive_sign_plus() {
        let v = MD::<113>::from_f64(f64::INFINITY);
        assert_eq!(fmt_plus(v), "+inf");
    }

    #[test]
    fn inf_negative_sign_plus() {
        // 負の無限大は sign_plus に関係なく "-"
        let v = MD::<113>::from_f64(f64::NEG_INFINITY);
        assert_eq!(fmt_plus(v), "-inf");
    }

    // ── ゼロ ──────────────────────────────────────────────────

    #[test]
    fn zero() {
        let v = MD::<113>::from_f64(0.0);
        let s = fmt(v);
        assert!(s.starts_with("0."), "zero: {}", s);
        assert!(s.ends_with("e+00"), "zero exponent: {}", s);
    }

    #[test]
    fn zero_sign_plus() {
        let v = MD::<113>::from_f64(0.0);
        let s = fmt_plus(v);
        assert!(s.starts_with("+0."), "zero sign_plus: {}", s);
    }

    #[test]
    fn zero_precision() {
        let v = MD::<113>::from_f64(0.0);
        let s = fmt_prec(v, 5);
        assert_eq!(s, "0.00000e+00", "zero prec=5: {}", s);
    }

    // ── 符号 ──────────────────────────────────────────────────

    #[test]
    fn positive_no_sign() {
        let v = MD::<113>::from_f64(1.0);
        let s = fmt(v);
        assert!(!s.starts_with('+') && !s.starts_with('-'), "positive: {}", s);
    }

    #[test]
    fn positive_sign_plus() {
        let v = MD::<113>::from_f64(1.0);
        assert!(fmt_plus(v).starts_with('+'));
    }

    #[test]
    fn negative_sign() {
        let v = MD::<113>::from_f64(-1.0);
        assert!(fmt(v).starts_with('-'));
    }

    #[test]
    fn negative_sign_plus() {
        // 負数は sign_plus に関係なく "-"
        let v = MD::<113>::from_f64(-1.0);
        assert!(fmt_plus(v).starts_with('-'));
    }

    // ── フォーマット構造 ───────────────────────────────────────

    #[test]
    fn format_structure() {
        // "d.dddde±NN" の形であることを確認
        let v = MD::<113>::from_f64(1.0);
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
        let v = MD::<113>::from_f64(1.0);
        let s = fmt(v);
        let e_pos = s.find('e').unwrap();
        let exp_digits = &s[e_pos + 2..]; // '+' or '-' の後
        assert!(exp_digits.len() >= 2, "exp digits width: {}", s);
    }

    // ── precision 指定 ─────────────────────────────────────────

    #[test]
    fn precision_0() {
        let v = MD::<113>::from_f64(std::f64::consts::PI);
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
        let v = MD::<113>::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 5);
        let dot = s.find('.').unwrap();
        let e   = s.find('e').unwrap();
        assert_eq!(e - dot - 1, 5, "prec=5 digit count: {}", s);
    }

    #[test]
    fn precision_5_value() {
        let v = MD::<113>::from_f64(std::f64::consts::PI);
        let s = fmt_prec(v, 5);
        // "3.14159e+00" であること
        assert!(s.starts_with("3.14159"), "prec=5 value: {}", s);
    }

    #[test]
    fn precision_15_digit_count() {
        let v = MD::<113>::from_f64(std::f64::consts::PI);
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
        let pi = MD::<113>::pi();
        let s  = fmt_prec(pi, 33);
        // π の正しい桁: 3.14159265358979323846264338327950288...
        check_digits(&s, "3.14159265358979323846264338327950288e+00", 33);
    }

    #[test]
    fn e_digits() {
        let e = MD::<113>::e();
        let s = fmt_prec(e, 33);
        // e = 2.71828182845904523536028747135266249...
        check_digits(&s, "2.71828182845904523536028747135266249e+00", 33);
    }

    #[test]
    fn sqrt2_digits() {
        let s2 = MD::<113>::sqrt_2();
        let s  = fmt_prec(s2, 33);
        // √2 = 1.41421356237309504880168872420969807...
        check_digits(&s, "1.41421356237309504880168872420969807e+00", 33);
    }

    #[test]
    fn ln2_digits() {
        let ln2 = MD::<113>::ln_2();
        let s   = fmt_prec(ln2, 33);
        // ln2 = 6.93147180559945309417232121458176568e-01
        check_digits(&s, "6.93147180559945309417232121458176568e-01", 33);
    }

    // ── 指数スケール ───────────────────────────────────────────

    #[test]
    fn one() {
        let s = fmt(MD::<113>::from_f64(1.0));
        let e_pos = s.find('e').unwrap();
        assert_eq!(&s[e_pos..e_pos + 4], "e+00", "1.0 exponent: {}", s);
    }

    #[test]
    fn large_exponent() {
        let v = MD::<113>::from_f64(1.23e50);
        let s = fmt(v);
        assert!(s.contains("e+50"), "1.23e50: {}", s);
    }

    #[test]
    fn small_exponent() {
        let v = MD::<113>::from_f64(1.23e-50);
        let s = fmt(v);
        assert!(s.contains("e-50"), "1.23e-50: {}", s);
    }

    #[test]
    fn negative_pi() {
        let v = -MD::<113>::pi();
        let s = fmt_prec(v, 10);
        assert!(s.starts_with("-3.1415926535"), "negative pi: {}", s);
    }

    // ── ラウンドトリップ ───────────────────────────────────────

    #[test]
    fn roundtrip_f64() {
        let cases = [1.0_f64, -1.0, 0.5, 100.0, 1e-10, 1e10, 1.23456789];
        for &v in &cases {
            let md  = MD::<113>::from_f64(v);
            let s   = fmt_prec(md, 15);
            let got = s.parse::<f64>().unwrap();
            let tol = v.abs() * 1e-14 + 1e-300;
            assert!(
                (got - v).abs() < tol,
                "roundtrip failed for {}: s={}, got={}", v, s, got
            );
        }
    }

    // ── f256 精度でも動作確認 ──────────────────────────────────

    #[test]
    fn f256_pi_digits() {
        let pi = MD::<237>::pi();
        let s  = format!("{:.70e}", pi);
        // 先頭 35 桁が正しいことを確認
        check_digits(
            &s,
            "3.14159265358979323846264338327950288419716939937510e+00",
            50,
        );
    }
}

#[cfg(test)]
mod tests_real {
    use std::f64;

    use super::*;

    // ── ヘルパー ──────────────────────────────────────────────

    /// MD<N> の値を f64 に変換する
    fn to_f64<const N: usize>(v: &MD<N>) -> f64 {
        format!("{:.20e}", v).parse::<f64>().unwrap()
    }

    /// f64 との近似一致チェック (相対誤差 tol)
    fn approx_eq<const N: usize>(a: &MD<N>, expected: f64, tol: f64) -> bool {
        let got = to_f64(a);
        if expected == 0.0 {
            got.abs() < tol
        } else {
            ((got - expected) / expected).abs() < tol
        }
    }

    /// 数字列だけ抽出して先頭 n 桁を比較する
    fn check_digits<const N: usize>(v: &MD<N>, expected: &str, digits: usize) {
        let s = format!("{:.prec$e}", v, prec = digits + 2);
        let got: String = s.chars()
            .take_while(|&c| c != 'e')
            .filter(|c| c.is_ascii_digit())
            .collect();
        let exp: String = expected.chars()
            .filter(|c| c.is_ascii_digit())
            .take(digits)
            .collect();
        let n = digits.min(got.len()).min(exp.len());
        assert_eq!(&got[..n], &exp[..n],
            "\nActual:   {}\nExpected: {}", s, expected);
    }

    const TOL_F64: f64 = 1e-14;   // f64 相当の誤差
    const TOL_HIGH: f64 = 1e-28;  // 高精度 (MD<128>) の誤差

    // ── from_f64 / to_i32 ────────────────────────────────────

    #[test]
    fn from_f64_roundtrip() {
        let cases = [0.0_f64, 1.0, -1.0, 0.5, 1e10, -1e10, 1e-10];
        for &v in &cases {
            let md = MD::<128>::from_f64(v);
            let got = to_f64(&md);
            assert!((got - v).abs() <= v.abs() * 1e-15 + 1e-300,
                "from_f64 roundtrip: v={v}, got={got}");
        }
    }

    #[test]
    fn to_i32_normal() {
        // dashu の to_i32() は 最近切丸め (3.7 -> 4)
        assert_eq!(MD::<128>::from_f64(3.7).to_i32(), 4);
        assert_eq!(MD::<128>::from_f64(-3.7).to_i32(), -4);
        assert_eq!(MD::<128>::from_f64(0.0).to_i32(), 0);
    }

    #[test]
    fn to_i32_clamp() {
        let large = MD::<128>::from_f64(1e18);
        assert_eq!(large.to_i32(), i32::MAX);
        let small = MD::<128>::from_f64(-1e18);
        assert_eq!(small.to_i32(), i32::MIN);
    }

    #[test]
    fn is_i32_compatible() {
        assert!(MD::<128>::from_f64(42.0).is_i32_compatible());
        assert!(!MD::<128>::from_f64(1e18).is_i32_compatible());
    }

    #[test]
    fn fract_and_trunc() {
        // 演算を通して precision を確定
        let v = MD::<128>::from_f64(3.0) + MD::<128>::from_f64(0.75);
        assert!(approx_eq(&v.clone().fract(), 0.75, TOL_F64));
        assert!(approx_eq(&v.trunc(), 3.0, TOL_F64));

        let v = MD::<128>::from_f64(-3.0) - MD::<128>::from_f64(0.75);
        assert!(approx_eq(&v.clone().fract(), -0.75, TOL_F64));
        assert!(approx_eq(&v.trunc(), -3.0, TOL_F64));
    }

    // ── 数学定数 ─────────────────────────────────────────────

    #[test]
    fn const_pi() {
        check_digits(&MD::<128>::pi(),
            "314159265358979323846264338327950", 30);
    }

    #[test]
    fn const_e() {
        check_digits(&MD::<128>::e(),
            "271828182845904523536028747135266", 30);
    }

    #[test]
    fn const_tau() {
        // tau = 2π
        let tau = MD::<128>::tau();
        let two_pi = MD::<128>::pi() * MD::<128>::from(2usize);
        assert!(approx_eq(&(tau - two_pi), 0.0, TOL_HIGH));
    }

    #[test]
    fn const_sqrt_2() {
        check_digits(&MD::<128>::sqrt_2(),
            "141421356237309504880168872420969", 30);
    }

    #[test]
    fn const_ln_2() {
        check_digits(&MD::<128>::ln_2(),
            "693147180559945309417232121458176", 30);
    }

    #[test]
    fn const_ln_10() {
        check_digits(&MD::<128>::ln_10(),
            "230258509299404568401799145468436", 30);
    }

    #[test]
    fn const_log_identities() {
        // log2(e) = 1/ln(2)
        let lhs = MD::<128>::log2_e();
        let rhs = MD::<128>::one() / MD::<128>::ln_2();
        assert!(approx_eq(&(lhs - rhs), 0.0, TOL_HIGH));

        // log2(10) * log10(2) = 1
        let product = MD::<128>::log2_10() * MD::<128>::log10_2();
        assert!(approx_eq(&(product - MD::<128>::one()), 0.0, TOL_HIGH));

        // log10(e) = 1/ln(10)
        let lhs = MD::<128>::log10_e();
        let rhs = MD::<128>::one() / MD::<128>::ln_10();
        assert!(approx_eq(&(lhs - rhs), 0.0, TOL_HIGH));
    }

    #[test]
    fn const_pi_fractions() {
        let pi = MD::<128>::pi();

        // frac_pi_2 = π/2
        let diff = MD::<128>::frac_pi_2() - pi.clone() / MD::<128>::from(2usize);
        assert!(approx_eq(&diff, 0.0, TOL_HIGH));

        // frac_pi_4 = π/4
        let diff = MD::<128>::frac_pi_4() - pi.clone() / MD::<128>::from(4usize);
        assert!(approx_eq(&diff, 0.0, TOL_HIGH));

        // frac_1_pi * π = 1
        let product = MD::<128>::frac_1_pi() * pi.clone();
        assert!(approx_eq(&(product - MD::<128>::one()), 0.0, TOL_HIGH));

        // frac_2_pi * π = 2
        let product = MD::<128>::frac_2_pi() * pi.clone();
        assert!(approx_eq(&(product - MD::<128>::from(2usize)), 0.0, TOL_HIGH));

        // frac_2_sqrt_pi^2 * π = 4
        let v = MD::<128>::frac_2_sqrt_pi();
        let product = v.clone() * v * pi.clone();
        assert!(approx_eq(&(product - MD::<128>::from(4usize)), 0.0, TOL_HIGH));
    }

    #[test]
    fn const_sqrt_2_fractions() {
        // frac_1_sqrt_2 = 1/√2
        let v = MD::<128>::frac_1_sqrt_2();
        let product = v * MD::<128>::sqrt_2();
        assert!(approx_eq(&(product - MD::<128>::one()), 0.0, TOL_HIGH));
    }

    // ── 四則演算と精度 ────────────────────────────────────────

    #[test]
    fn arithmetic_basic() {
        let a = MD::<128>::from_f64(1.5);
        let b = MD::<128>::from_f64(2.5);
        assert!(approx_eq(&(&a + &b), 4.0, TOL_HIGH), "a + b != 4.0, result={}", &a + &b);
        assert!(approx_eq(&(&b - &a), 1.0, TOL_HIGH), "a - b != 1.0, result={}", &a - &b);
        assert!(approx_eq(&(&a * &b), 3.75, TOL_HIGH), "a * b != 3.75, result={}", &a * &b);
        assert!(approx_eq(&(&b / &a), 5.0/3.0, TOL_HIGH), "a / b != 5/3, result={}", &a / &b);
        assert!(approx_eq(&(&b % &a), 1.0, TOL_HIGH), "a % b != 1.0, result={} (std={})", &a % &b, 2.5 % 1.5);
    }

    // ── 指数・対数 ────────────────────────────────────────────

    #[test]
    fn exp_ln_inverse() {
        // exp(ln(x)) = x
        for v in [0.5, 1.0, 2.0, 10.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().ln().exp();
            assert!(approx_eq(&(result - x), 0.0, TOL_HIGH),
                "exp(ln({v})) != {v}");
        }
    }

    #[test]
    fn ln_exp_inverse() {
        // ln(exp(x)) = x
        for v in [-1.0, 0.0, 1.0, 2.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().exp().ln();
            assert!(approx_eq(&(result - x), 0.0, TOL_HIGH),
                "ln(exp({v})) != {v}");
        }
    }

    #[test]
    fn exp_of_one_is_e() {
        let e = MD::<128>::one().exp();
        check_digits(&e, "271828182845904523536028747135266", 30);
    }

    #[test]
    fn ln_of_e_is_one() {
        let result = MD::<128>::e().ln();
        assert!(approx_eq(&(result - MD::<128>::one()), 0.0, TOL_HIGH));
    }

    #[test]
    fn log10_correct() {
        // log10(100) = 2
        let v = MD::<128>::from_f64(100.0);
        assert!(approx_eq(&(v.log10() - MD::<128>::from(2usize)), 0.0, TOL_HIGH));

        // log10(1000) = 3
        let v = MD::<128>::from_f64(1000.0);
        assert!(approx_eq(&(v.log10() - MD::<128>::from(3usize)), 0.0, TOL_HIGH));

        // log10(e) の定数と一致
        let v = MD::<128>::e().log10();
        let expected = MD::<128>::log10_e();
        assert!(approx_eq(&(v - expected), 0.0, TOL_HIGH));
    }

    // ── sqrt / hypot / abs ────────────────────────────────────

    #[test]
    fn sqrt_correct() {
        // sqrt(2) の定数と一致
        let v = MD::<128>::from(2usize).sqrt();
        let diff = v - MD::<128>::sqrt_2();
        assert!(approx_eq(&diff, 0.0, TOL_HIGH));

        // sqrt(x)^2 = x
        for v in [2.0, 3.0, 5.0, 7.0] {
            let x = MD::<128>::from_f64(v);
            let s = x.clone().sqrt();
            let diff = s.clone() * s - x;
            assert!(approx_eq(&diff, 0.0, TOL_HIGH), "sqrt({v})^2 != {v}");
        }
    }

    #[test]
    fn hypot_pythagorean() {
        // 3-4-5 直角三角形
        let a = MD::<128>::from(3usize);
        let b = MD::<128>::from(4usize);
        assert!(approx_eq(&a.hypot(b), 5.0, TOL_HIGH));
    }

    #[test]
    fn abs_correct() {
        assert!(approx_eq(&MD::<128>::from_f64(-3.5).abs(), 3.5, TOL_HIGH));
        assert!(approx_eq(&MD::<128>::from_f64(3.5).abs(), 3.5, TOL_HIGH));
    }

    // ── pow / powi ────────────────────────────────────────────

    #[test]
    fn pow_correct() {
        // 2^10 = 1024
        let base = MD::<128>::from(2usize);
        let exp  = MD::<128>::from(10usize);
        assert!(approx_eq(&(base.pow(exp) - MD::<128>::from_f64(1024.0)), 0.0, TOL_HIGH));
    }

    #[test]
    fn powi_correct() {
        let x = MD::<128>::from(3usize);
        let expected = MD::<128>::from(81usize); // from_f64 でなく from で精度を揃える
        assert!(approx_eq(&(x.powi(4) - expected), 0.0, TOL_HIGH));

        let x = MD::<128>::from(2usize);
        let half = MD::<128>::from_f64(1.0) / MD::<128>::from(2usize); // 演算で精度確定
        assert!(approx_eq(&(x.powi(-1) - half), 0.0, TOL_HIGH));
    }

    // ── 三角関数 ─────────────────────────────────────────────

    #[test]
    fn sin_known_values() {
        let tol = 1e-28;
        // sin(0) = 0
        assert!(approx_eq(&MD::<128>::zero().sin(), 0.0, tol));
        // sin(π/6) = 0.5
        let diff = MD::<128>::frac_pi_2().sin() - MD::<128>::one();
        assert!(approx_eq(&diff, 0.0, tol), "sin(π/2) != 1");
        // sin(π) ≈ 0
        let sin_pi = MD::<128>::pi().sin();
        assert!(to_f64(&sin_pi).abs() < 1e-28, "sin(π) should be ~0: {:e}", sin_pi);
    }

    #[test]
    fn cos_known_values() {
        let tol = 1e-28;
        // cos(0) = 1
        let diff = MD::<128>::zero().cos() - MD::<128>::one();
        assert!(approx_eq(&diff, 0.0, tol));
        // cos(π) = -1
        let diff = MD::<128>::pi().cos() + MD::<128>::one();
        assert!(approx_eq(&diff, 0.0, tol), "cos(π) != -1");
        // cos(π/2) ≈ 0
        let cos_pi_2 = MD::<128>::frac_pi_2().cos();
        assert!(to_f64(&cos_pi_2).abs() < 1e-28, "cos(π/2) should be ~0");
    }

    #[test]
    fn sin_cos_pythagorean_identity() {
        // sin²(x) + cos²(x) = 1
        for v in [0.1, 0.5, 1.0, 1.23456, 2.5, 3.0] {
            let x = MD::<128>::from_f64(v);
            let (s, c) = x.sin_cos();
            let sum = s.clone() * s + c.clone() * c;
            assert!(approx_eq(&(sum - MD::<128>::one()), 0.0, 1e-25),
                "sin²+cos² != 1 at x={v}");
        }
    }

    #[test]
    fn sin_cos_negative() {
        // sin(-x) = -sin(x), cos(-x) = cos(x)
        let x = MD::<128>::from_f64(1.23456);
        let sin_x  = x.clone().sin();
        let sin_nx = (-x.clone()).sin();
        let diff = sin_x + sin_nx;
        assert!(approx_eq(&diff, 0.0, 1e-25), "sin(-x) != -sin(x)");

        let cos_x  = x.clone().cos();
        let cos_nx = (-x).cos();
        let diff = cos_x - cos_nx;
        assert!(approx_eq(&diff, 0.0, 1e-25), "cos(-x) != cos(x)");
    }

    #[test]
    fn sin_cos_addition_formula() {
        // sin(a+b) = sin(a)cos(b) + cos(a)sin(b)
        let a = MD::<128>::from_f64(1.2);
        let b = MD::<128>::from_f64(0.8);
        let lhs = (a.clone() + b.clone()).sin();
        let rhs = a.clone().sin() * b.clone().cos() + a.cos() * b.sin();
        assert!(approx_eq(&(lhs - rhs), 0.0, 1e-25), "sin(a+b) identity failed");
    }

    #[test]
    fn tan_correct() {
        // tan(x) = sin(x)/cos(x)
        for v in [0.1, 0.5, 1.0, -1.0] {
            let x = MD::<128>::from_f64(v);
            let tan = x.clone().tan();
            let sin_cos = x.clone().sin() / x.cos();
            assert!(approx_eq(&(tan - sin_cos), 0.0, 1e-25),
                "tan != sin/cos at x={v}");
        }
        // tan(π/4) = 1
        let diff = MD::<128>::frac_pi_4().tan() - MD::<128>::one();
        assert!(approx_eq(&diff, 0.0, 1e-25), "tan(π/4) != 1");
    }

    // ── 逆三角関数 ────────────────────────────────────────────

    #[test]
    fn asin_inverse_of_sin() {
        // asin(sin(x)) = x for x ∈ (-π/2, π/2)
        for v in [-0.9, -0.5, 0.0, 0.5, 0.9] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().sin().asin();
            assert!(approx_eq(&(&result - &x), 0.0, 1e-24),
                "asin(sin({v})) != {v}, result={result}");
        }
        // sin(asin(x)) = x for x ∈ [-1, 1]
        for v in [-1.0, -0.999, -0.5, 0.0, 0.5, 0.999, 1.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().asin().sin();
            assert!(approx_eq(&(&result - &x), 0.0, 1e-24),
                "sin(asin({v})) != {v}, result={result}");
        }
    }

    #[test]
    fn acos_inverse_of_cos() {
        // acos(cos(x)) = x for x ∈ [0, π]
        for v in [0.0, 0.5, 1.0, 1.5, 2.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().cos().acos();
            assert!(approx_eq(&(&result - &x), 0.0, 1e-24),
                "cos(acos({v})) != {v}, result={result}");
        }
        // cos(acos(x)) = x for x ∈ [-1, 1]
        for v in [-1.0, -0.999, -0.5, 0.0, 0.5, 0.999, 1.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().acos().cos();
            assert!(approx_eq(&(&result - &x), 0.0, 1e-24),
                "cos(acos({v})) != {v}, result={result}");
        }
    }

    #[test]
    fn asin_acos_complementary() {
        // asin(x) + acos(x) = π/2
        for v in [-0.9, -0.5, 0.0, 0.5, 0.9] {
            let x = MD::<128>::from_f64(v);
            let sum = x.clone().asin() + x.acos();
            let diff = sum - MD::<128>::frac_pi_2();
            assert!(approx_eq(&diff, 0.0, 1e-25),
                "asin+acos != π/2 at x={v}");
        }
    }

    #[test]
    fn atan_known_values() {
        // atan(0) = 0
        assert!(approx_eq(&MD::<128>::zero().atan(), 0.0, 1e-28));
        // atan(1) = π/4
        let diff = MD::<128>::one().atan() - MD::<128>::frac_pi_4();
        assert!(approx_eq(&diff, 0.0, 1e-25), "atan(1) != π/4");
        // atan(-1) = -π/4
        let diff = (-MD::<128>::one()).atan() + MD::<128>::frac_pi_4();
        assert!(approx_eq(&diff, 0.0, 1e-25), "atan(-1) != -π/4");
    }

    #[test]
    fn atan_inverse_of_tan() {
        // atan(tan(x)) = x for x ∈ (-π/2, π/2)
        for v in [-0.7, -0.5, 0.0, 0.5, 0.7] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().tan().atan();
            assert!(approx_eq(&(&result - &x), 0.0, 1e-24),
                "atan(tan({v})) != {v}, result={result}");
        }
        let x = MD::<128>::from_f64(-1.0);
        let tan_x = x.clone().tan();
        let result = tan_x.atan();
        assert!(approx_eq(&(result - x), 0.0, 1e-24));
    }

    #[test]
    fn atan2_quadrants() {
        let tol = 1e-25;
        let pi = MD::<128>::pi();
        let pi_4 = MD::<128>::frac_pi_4();
        let pi_2 = MD::<128>::frac_pi_2();

        // 第1象限: atan2(1,1) = π/4
        let diff = MD::<128>::one().atan2(MD::<128>::one()) - pi_4.clone();
        assert!(approx_eq(&diff, 0.0, tol), "atan2(1,1) != π/4");

        // 第2象限: atan2(1,-1) = 3π/4
        let expected = pi_4.clone() * MD::<128>::from(3usize);
        let diff = MD::<128>::one().atan2(-MD::<128>::one()) - expected;
        assert!(approx_eq(&diff, 0.0, tol), "atan2(1,-1) != 3π/4");

        // 第3象限: atan2(-1,-1) = -3π/4
        let expected = -(pi_4.clone() * MD::<128>::from(3usize));
        let diff = (-MD::<128>::one()).atan2(-MD::<128>::one()) - expected;
        assert!(approx_eq(&diff, 0.0, tol), "atan2(-1,-1) != -3π/4");

        // 軸: atan2(0,1)=0, atan2(0,-1)=π, atan2(1,0)=π/2, atan2(-1,0)=-π/2
        assert!(approx_eq(&MD::<128>::zero().atan2(MD::<128>::one()), 0.0, tol));
        let diff = MD::<128>::zero().atan2(-MD::<128>::one()) - pi.clone();
        assert!(approx_eq(&diff, 0.0, tol), "atan2(0,-1) != π");
        let diff = MD::<128>::one().atan2(MD::<128>::zero()) - pi_2.clone();
        assert!(approx_eq(&diff, 0.0, tol), "atan2(1,0) != π/2");
        let diff = (-MD::<128>::one()).atan2(MD::<128>::zero()) + pi_2;
        assert!(approx_eq(&diff, 0.0, tol), "atan2(-1,0) != -π/2");
    }

    // ── 双曲線関数 ────────────────────────────────────────────

    #[test]
    fn sinh_cosh_identity() {
        // cosh²(x) - sinh²(x) = 1
        for v in [0.0, 0.5, 1.0, 2.0, -1.0] {
            let x = MD::<128>::from_f64(v);
            let c = x.clone().cosh();
            let s = x.sinh();
            let diff = c.clone() * c - s.clone() * s - MD::<128>::one();
            assert!(approx_eq(&diff, 0.0, 1e-25),
                "cosh²-sinh² != 1 at x={v}");
        }
    }

    #[test]
    fn sinh_known_values() {
        // sinh(0) = 0
        assert!(approx_eq(&MD::<128>::zero().sinh(), 0.0, 1e-28));
        // sinh(x) = -sinh(-x)
        let x = MD::<128>::from_f64(1.5);
        let diff = x.clone().sinh() + (-x).sinh();
        assert!(approx_eq(&diff, 0.0, 1e-25), "sinh(x) != -sinh(-x)");
    }

    #[test]
    fn cosh_known_values() {
        // cosh(0) = 1
        let diff = MD::<128>::zero().cosh() - MD::<128>::one();
        assert!(approx_eq(&diff, 0.0, 1e-28));
        // cosh(x) = cosh(-x) (偶関数)
        let x = MD::<128>::from_f64(1.5);
        let diff = x.clone().cosh() - (-x).cosh();
        assert!(approx_eq(&diff, 0.0, 1e-25), "cosh not even");
    }

    #[test]
    fn tanh_range() {
        // tanh(x) ∈ (-1, 1)
        for v in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            let t = to_f64(&MD::<128>::from_f64(v).tanh());
            assert!(t > -1.0 && t < 1.0, "tanh({v}) = {t} out of range");
        }
        // tanh(0) = 0
        assert!(approx_eq(&MD::<128>::zero().tanh(), 0.0, 1e-28));
    }

    #[test]
    fn asinh_inverse_of_sinh() {
        for v in [-2.0, -1.0, 0.0, 0.5, 1.0, 2.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().asinh().sinh();
            assert!(approx_eq(&(result - x), 0.0, 1e-24),
                "sinh(asinh({v})) != {v}");
        }
    }

    #[test]
    fn acosh_inverse_of_cosh() {
        for v in [1.0, 1.5, 2.0, 3.0] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().acosh().cosh();
            assert!(approx_eq(&(result - x), 0.0, 1e-24),
                "cosh(acosh({v})) != {v}");
        }
    }

    #[test]
    fn atanh_inverse_of_tanh() {
        for v in [-0.9, -0.5, 0.0, 0.5, 0.9] {
            let x = MD::<128>::from_f64(v);
            let result = x.clone().tanh().atanh();
            assert!(approx_eq(&(result - x), 0.0, 1e-24),
                "atanh(tanh({v})) != {v}");
        }
    }

    // ── 精度スケール確認: MD<64> vs MD<128> ──────────────────

    #[test]
    fn precision_64_vs_128() {
        // MD<128> の方が MD<64> より π を高精度で保持する
        let pi_64  = MD::<64>::pi();
        let pi_128 = MD::<128>::pi();

        let s64  = format!("{:.30e}", pi_64);
        let s128 = format!("{:.30e}", pi_128);

        let d64: String  = s64.chars().take_while(|&c| c != 'e').filter(|c| c.is_ascii_digit()).collect();
        let d128: String = s128.chars().take_while(|&c| c != 'e').filter(|c| c.is_ascii_digit()).collect();

        // MD<128> は MD<64> より多くの正しい桁を持つ
        let ref_pi = "314159265358979323846264338327";
        let correct_64  = d64.chars().zip(ref_pi.chars()).take_while(|(a,b)| a==b).count();
        let correct_128 = d128.chars().zip(ref_pi.chars()).take_while(|(a,b)| a==b).count();
        assert!(correct_128 > correct_64,
            "MD<128> should be more precise than MD<64>: {correct_64} vs {correct_128}");
    }
}
