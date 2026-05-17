//! gamma.rs
//!
//! 複素数引数のGamma関数
//! 参考： shikino, https://slpr.sakura.ne.jp/qp/

use formulac::core::{
    ComplexMath,
    Real,
};
use num_complex::Complex;
use num_traits::{
    One,
    ToPrimitive,
    Zero,
};
use once_cell::sync::Lazy;
use crate::multi_precision::{
    F106,
};

pub(crate) trait Gamma {
    fn gamma(&self) -> Self;
    fn gamma_by_taylor_expansion(&self) -> Self;
    fn gamma_by_stirling_series(&self) -> Self;
}


impl Gamma for Complex<f64> {
    fn gamma(&self) -> Self {
        if self.im.is_zero() {
            return Self::from(special::Gamma::gamma(self.re));
        }

        let z = if self.re.is_sign_negative() { Self::ONE - self } else { *self };
        const THRESHOLD_FOR_TAYLOR_EXPANSION_OF_INV_GAMMA: f64 = 1.45;
        let result = if self.im.abs() < THRESHOLD_FOR_TAYLOR_EXPANSION_OF_INV_GAMMA {
            // w = z - floor(Re(z)) としてシフトするので、虚部の大きさだけを見る
            z.gamma_by_taylor_expansion()
        } else {
            z.gamma_by_stirling_series()
        };

        if self.re.is_sign_negative() {
            const PI: f64 = std::f64::consts::PI;
            PI / (result * (PI * z).sin())
        } else {
            result
        }
    }

    fn gamma_by_taylor_expansion(&self) -> Self {
        const COEF: [f64; 30] = [
             1.0E0,                     0.57721566490153286E0,         -0.65587807152025388E0,
            -0.42002635034095236E-1,    0.16653861138229149E0,         -0.42197734555544337E-1,
            -0.96219715278769736E-2,    0.72189432466630995E-2,        -0.11651675918590651E-2,
            -0.21524167411495097E-3,    0.12805028238811619E-3,        -0.20134854780788239E-4,
            -0.12504934821426707E-5,    0.11330272319816959E-5,        -0.20563384169776071E-6,
             0.61160951044814158E-8,    0.50020076444692229E-8,        -0.11812745704870201E-8,
             0.10434267116911005E-9,    0.77822634399050713E-11,       -0.36968056186422057E-11,
             0.51003702874544760E-12,  -0.20583260535665068E-13,       -0.53481225394230180E-14,
             0.12267786282382608E-14,  -0.11812593016974588E-15,        0.11866922547516003E-17,
             0.14123806553180318E-17,  -0.22987456844353702E-18,        0.17144063219273374E-19,
        ];

        let mut n = self.re.floor(); // この関数が呼ばれる段階で、Re(z)は正となっている
        let w = self - n;
        let mut s = Self::ONE;
        while n > 0.0 {
            s *= self - n;
            n -= 1.0;
        }

        // 経験則により展開に必要な項数を求める
        let iter_max = if let Some(m) = (11.3 * w.norm() + 13.0).to_usize() {
            std::cmp::min(COEF.len(), m)
        } else {
            COEF.len()
        };
        let mut r = Self::zero();
        for coef in COEF[..iter_max].iter().rev() {
            r = r * w + coef;
        }

        s / (r * w)
    }

    fn gamma_by_stirling_series(&self) -> Self {
        let mut s = Self::ONE;
        let mut q = *self;
        if self.abs().re < 9.0 {
            for i in 0..8 {
                s *= self + Self::from(i as f64);
            }
            s = s.inv();
            q += 8.0;
        }

        const BERNOULLI: [f64; 8] = [
             0.83333333333333333E-1,   -0.27777777777777778E-2,         0.79365079365079365E-3,
            -0.59523809523809524E-3,    0.84175084175084175E-3,        -0.19175269175269175E-2,
             0.64102564102564103E-2,   -0.29550653594771242E-1
        ];

        let q1 = q.inv();
        let q2 = q1 * q1;

        let mut r = Self::zero();
        for bernoulli in BERNOULLI[..BERNOULLI.len()].iter().rev() {
            r = r * q2 + bernoulli;
        }

        const LN2PI2: f64 = 0.91893853320467274;
        s * ((q - 0.5) * q.ln() - q + LN2PI2 + r * q1).exp()
    }
}

#[cfg(test)]
mod gamma_tests {
    use super::*;
    use num_complex::Complex;

    const EPS: f64 = 1e-12;

    fn assert_close(actual: Complex<f64>, expected: Complex<f64>) {
        let err = (actual - expected).norm();
        assert!(
            err < EPS,
            "actual={:?}, expected={:?}, err={}",
            actual,
            expected,
            err
        );
    }

    fn assert_close_real(actual: f64, expected: f64) {
        let err = (actual - expected).abs();
        assert!(
            err < EPS,
            "actual={}, expected={}, err={}",
            actual,
            expected,
            err
        );
    }

    // ------------------------------------------------------------
    // 実数値テスト
    // ------------------------------------------------------------

    #[test]
    fn gamma_real_positive_integer() {
        // Γ(5) = 4! = 24
        let z = Complex::new(5.0, 0.0);
        assert_close_real(z.gamma().re, 24.0);
    }

    #[test]
    fn gamma_real_half() {
        // Γ(1/2) = sqrt(pi)
        let z = Complex::new(0.5, 0.0);

        assert_close_real(
            z.gamma().re,
            std::f64::consts::PI.sqrt(),
        );
    }

    #[test]
    fn gamma_real_negative_half() {
        // Γ(-1/2) = -2 sqrt(pi)
        let z = Complex::new(-0.5, 0.0);

        assert_close_real(
            z.gamma().re,
            -2.0 * std::f64::consts::PI.sqrt(),
        );
    }

    #[test]
    fn gamma_real_one() {
        // Γ(1) = 1
        let z = Complex::new(1.0, 0.0);
        assert_close_real(z.gamma().re, 1.0);
    }

    #[test]
    fn gamma_real_two() {
        // Γ(2) = 1
        let z = Complex::new(2.0, 0.0);
        assert_close_real(z.gamma().re, 1.0);
    }

    // ------------------------------------------------------------
    // 複素数テスト
    // ------------------------------------------------------------

    #[test]
    fn gamma_complex_i() {
        // mpmath:
        // gamma(1j)
        // = -0.15494982830181068512
        //   -0.49801566811835604271j

        let z = Complex::new(0.0, 1.0);

        let expected = Complex::new(
            -0.15494982830181068,
            -0.49801566811835604,
        );

        assert_close(z.gamma(), expected);
    }

    #[test]
    fn gamma_complex_one_plus_i() {
        // Γ(1+i) = i Γ(i)

        let z = Complex::new(1.0, 1.0);

        let expected = Complex::new(
            0.49801566811835604,
            -0.15494982830181068,
        );

        assert_close(z.gamma(), expected);
    }

    #[test]
    fn gamma_complex_half_plus_i() {
        // mpmath:
        // gamma(0.5 + 1j)
        // = 0.300694617260656
        //   -0.424967879433124j

        let z = Complex::new(0.5, 1.0);

        let expected = Complex::new(
            0.300694617260656,
            -0.424967879433124,
        );

        assert_close(z.gamma(), expected);
    }

    // ------------------------------------------------------------
    // 関数方程式テスト
    // Γ(z+1)=zΓ(z)
    // ------------------------------------------------------------

    #[test]
    fn gamma_functional_equation() {
        let z = Complex::new(0.3, 0.7);

        let lhs = (z + Complex::ONE).gamma();
        let rhs = z * z.gamma();

        assert_close(lhs, rhs);
    }

    // ------------------------------------------------------------
    // reflection formula
    // Γ(z)Γ(1-z)=π/sin(πz)
    // ------------------------------------------------------------

    #[test]
    fn gamma_reflection_formula() {
        let z = Complex::new(0.3, 0.4);

        let lhs = z.gamma() * (Complex::<f64>::ONE - z).gamma();

        let rhs =
            Complex::new(std::f64::consts::PI, 0.0)
            / (Complex::new(std::f64::consts::PI, 0.0) * z).sin();

        assert_close(lhs, rhs);
    }

    // ------------------------------------------------------------
    // 極近傍
    // ------------------------------------------------------------

    #[test]
    fn gamma_near_pole() {
        let z = Complex::new(1e-6, 0.0);

        let lhs = (z + Complex::<f64>::ONE).gamma();
        let rhs = z * z.gamma();

        assert_close(lhs, rhs);
    }
}

fn gamma_integer(x: &F106) -> F106 {
    let one = F106::one();
    let mut s = one;
    let mut x = *x;
    while x > F106::one() {
        x -= one;
        s *= x;
    }
    s
}

/// inverse-gamma Taylor expansion coefficients
/// generated from quadruple precision constants
///
/// value ≈ hi + lo
fn inv_gamma_coef() -> [F106; 50] {
    const INV_GAMMA_COEF: [(f64, f64); 50] = [
        ( 1.000000000000000000e+00,  0.000000000000000000e+00),
        ( 5.772156649015328656e-01, -5.181208092144232593e-18),
        (-6.558780715202538635e-01, -1.749373486981120312e-17),
        (-4.200263503409523608e-02,  2.530531390996643948e-18),
        ( 1.665386113822914463e-01, -7.344714560540111002e-18),
        (-4.219773455554433868e-02, -3.801578805081137342e-18),
        (-9.621971527876973845e-03,  2.817625388242814055e-19),
        ( 7.218943246663099652e-03, -1.086973281615804665e-19),
        (-1.165167591859065056e-03, -5.707631752570445493e-20),
        (-2.152416741149509785e-04,  5.623730179826113764e-21),

        ( 1.280502823881161878e-04, -1.658270351633880036e-21),
        (-2.013485478078823923e-05,  5.747511091195082930e-22),
        (-1.250493482142670584e-06, -7.436759428834190168e-23),
        ( 1.133027231981695942e-06, -5.896304502421642262e-23),
        (-2.056338416977607204e-07,  1.008915331276999828e-23),
        ( 6.116095104481415645e-09,  1.726871991431183416e-25),
        ( 5.002007644469222621e-09,  3.090034884503090040e-25),
        (-1.181274570487020117e-09, -2.708196738571736388e-26),
        ( 1.043426711691100490e-10,  2.000141614184131919e-27),
        ( 7.782263439905071477e-12, -2.226367630111949794e-28),

        (-3.696805618642205832e-12,  1.235611007020211626e-28),
        ( 5.100370287454475853e-13,  1.260519534395386322e-29),
        (-2.058326053566506601e-14, -1.822211586559391921e-30),
        (-5.348122539423018146e-15,  1.635549321167774938e-31),
        ( 1.226778628238260748e-15,  4.228577489051679281e-32),
        (-1.181259301697458821e-16,  5.137588729970714080e-33),
        ( 1.186692254751600271e-18,  6.135992038730181541e-35),
        ( 1.412380655318031613e-18,  1.682499517350196628e-34),
        (-2.298745684435370171e-19, -3.508305106384529101e-36),
        ( 1.714406321927337392e-20,  4.157805743001037976e-37),

        ( 1.337351730493693094e-22,  2.086359187137905099e-39),
        (-2.054233551766672735e-22, -5.410276390745497182e-39),
        ( 2.736030048607999869e-23, -2.473061016072119742e-40),
        (-1.732356445910516603e-24, -3.566281210967995167e-41),
        (-2.360619024499287259e-26, -2.796693020553820050e-43),
        ( 1.864982941717294435e-26, -4.234172055300139949e-44),
        (-2.218095624207197335e-27,  1.313023134427779992e-43),
        ( 1.297781974947993641e-28,  2.741078452842095450e-45),
        ( 1.180697474966528442e-30, -3.566123286443566321e-47),
        (-1.124584349277088032e-30, -5.798822300059873326e-47),

        ( 1.277085175140866298e-31, -9.402271220915617661e-48),
        (-7.391451169615140773e-33, -4.949473070456238638e-50),
        ( 1.134750257554215693e-35,  6.744071198892938754e-52),
        ( 4.639134641058722146e-35, -1.979889930196180596e-51),
        (-5.347336818439198852e-36, -2.324051669541375025e-53),
        ( 3.207995923613352669e-37, -4.739822018083457936e-54),
        (-4.445829736550756912e-39,  2.957070718169682294e-56),
        (-1.311174518881988763e-39,  5.101484278704784278e-56),
        ( 1.647033352543813823e-40,  6.323969846466998291e-57),
        (-1.056233178503581241e-41,  2.289849733941304583e-58),
    ];
    static RETVAL: Lazy<[F106; 50]> = Lazy::new(
        || INV_GAMMA_COEF.map(|(hi, lo)| F106::new_add(hi, lo))
    );

    *RETVAL
}


/// Bernoulli-related Stirling coefficients
///
/// B₂/(2·1),
/// B₄/(4·3),
/// ...
fn stirling_bernoulli() -> [F106; 20] {
    const STIRLING_BERNOULLI: [(f64, f64); 20] = [
        ( 8.333333333333333333e-02,  4.625929269271485e-18),
        (-2.777777777777777778e-03, -1.156482317317871e-19),
        ( 7.936507936507936508e-04,  3.176373552203626e-20),
        (-5.952380952380952381e-04, -2.470770123568688e-20),
        ( 8.417508417508417508e-04,  3.623593539723816e-20),
        (-1.917526917526917527e-03, -7.854234347662994e-20),
        ( 6.410256410256410256e-03,  2.663310212085505e-19),
        (-2.955065359477124183e-02, -1.228192748731323e-18),
        ( 1.796443723688305731e-01,  7.441320389305239e-18),
        (-1.392432216905901117e+00, -5.752545299092528e-17),

        ( 1.340286404416839199e+01,  5.511680729221867e-16),
        (-1.568482846260020173e+02, -6.443210311073056e-15),
        ( 2.193103333333333333e+03,  9.001245777822950e-14),
        (-3.610877125372498936e+04, -1.480297879241738e-12),
        ( 6.914722688513130671e+05,  2.835219823321245e-11),
        (-1.523822153940741619e+07, -6.244210102019328e-10),
        ( 3.829007513914141414e+08,  1.568921847091223e-08),
        (-1.088226603578439109e+10, -4.452774123091521e-07),
        ( 3.473202837650022523e+11,  1.421998421138102e-05),
        (-1.236960214226927446e+13, -5.061723241998752e-04),
    ];

    static RETVAL: Lazy<[F106; 20]> = Lazy::new(
        || STIRLING_BERNOULLI.map(|(hi, lo)| F106::new_add(hi, lo))
    );

    *RETVAL
}

impl Gamma for Complex<F106> {
    fn gamma(&self) -> Self {
        if self.im.is_zero() && self.re.fract().is_zero() {
            if self.re <= F106::zero() {
                return Complex::new(F106::infinity(), F106::zero());
            }

            // 計算結果が巨大数となりオーバーフローするので、整数で計算する範囲を制限する
            if self.re <= F106::from_f64(50.0) {
                return Complex::from(gamma_integer(&self.re));
            }
        }

        let z = if self.re.is_sign_negative() { Self::from(F106::one()) - self } else { *self };
        let threshold_for_taylor_expansion_of_inv_gamma = F106::from_f64(1.35);
        let result = if self.im.abs() < threshold_for_taylor_expansion_of_inv_gamma {
            // w = z - floor(Re(z)) としてシフトするので、虚部の大きさだけを見る
            z.gamma_by_taylor_expansion()
        } else {
            z.gamma_by_stirling_series()
        };

        if self.re.is_sign_negative() {
            let pi = Complex::from(F106::pi());
            pi / (result * (pi * z).sin())
        } else {
            result
        }
    }

    fn gamma_by_taylor_expansion(&self) -> Self {
        let coef = inv_gamma_coef();

        let mut n = self.re.floor(); // この関数が呼ばれる段階で、Re(z)は正となっている
        let w = self - n;
        let mut s = Self::one();
        while n > F106::zero() {
            s *= self - n;
            n -= F106::one();
        }

        // 経験則により展開に必要な項数を求める
        let abs_w = w.abs().re;
        let m = if abs_w < F106::from_f64(0.42) {
            (F106::from_f64(35.0) * abs_w + F106::from_f64(20.0) + F106::one()).floor()
        } else {
            (F106::from_f64(15.5) * abs_w + F106::from_f64(28.2) + F106::one()).floor()
        };
        let iter_max = if m > F106::from_f64(50.0) {
            50
        } else {
            m.to_i32() as usize
        };

        let mut r = Self::zero();
        for c in coef[..iter_max].iter().rev() {
            r = r * w + c;
        }

        s / (r * w)
    }

    fn gamma_by_stirling_series(&self) -> Self {
        let mut s = Self::one();
        let mut q = *self;
        if self.abs().re < F106::from_f64(18.0) {
            for i in 0..17 {
                s *= self + Self::from(F106::from_f64(i as f64));
            }
            s = s.inv();
            q += F106::from_f64(17.0);
        }

        let bernoulli = stirling_bernoulli();

        let q1 = q.inv();
        let q2 = q1 * q1;

        let mut r = Self::zero();
        for b in bernoulli[..bernoulli.len()].iter().rev() {
            r = r * q2 + b;
        }

        let ln2pi2 = F106::new_add(9.189385332046727417e-01, -5.932760173701456037e-17,);
        s * ((q - F106::from_f64(0.5)) * q.ln() - q + ln2pi2 + r * q1).exp()
    }
}

#[cfg(test)]
mod gamma_dd_tests {
    use super::*;
    use num_complex::Complex;

    fn f(x: f64) -> F106 {
        F106::from_f64(x)
    }

    fn c(re: f64, im: f64) -> Complex<F106> {
        Complex::new(f(re), f(im))
    }

    fn assert_close_real(actual: F106, expected: F106, eps: f64) {
        let err = (actual - expected).abs();

        assert!(
            err < f(eps),
            "actual={:?}, expected={:?}, err={:?}",
            actual,
            expected,
            err,
        );
    }

    fn assert_close_complex(
        actual: Complex<F106>,
        expected: Complex<F106>,
        eps: f64,
    ) {
        let err = (actual - expected).abs();

        assert!(
            err.re < f(eps),
            "actual={:?}, expected={:?}, err={:?}",
            actual,
            expected,
            err,
        );
    }

    // ------------------------------------------------------------
    // basic real values
    // ------------------------------------------------------------

    #[test]
    fn gamma_1() {
        let z = c(1.0, 0.0);

        assert_close_real(
            z.gamma().re,
            f(1.0),
            1e-30,
        );
    }

    #[test]
    fn gamma_2() {
        let z = c(2.0, 0.0);

        assert_close_real(
            z.gamma().re,
            f(1.0),
            1e-30,
        );
    }

    #[test]
    fn gamma_5() {
        let z = c(5.0, 0.0);

        // Γ(5)=24
        assert_close_real(
            z.gamma().re,
            f(24.0),
            1e-28,
        );
    }

    #[test]
    fn gamma_half() {
        let z = c(0.5, 0.0);

        let expected = F106::pi().sqrt();

        assert_close_real(
            z.gamma().re,
            expected,
            1e-16,
        );
    }

    #[test]
    fn gamma_minus_half() {
        let z = c(-0.5, 0.0);

        let expected =
            -f(2.0) * F106::pi().sqrt();

        assert_close_real(
            z.gamma().re,
            expected,
            1e-15,
        );
    }

    // ------------------------------------------------------------
    // poles
    // ------------------------------------------------------------

    #[test]
    fn gamma_zero_is_inf() {
        let z = c(0.0, 0.0);

        let result = z.gamma();

        assert!(result.re.is_infinite());
    }

    #[test]
    fn gamma_negative_integer_is_inf() {
        let z = c(-3.0, 0.0);

        let result = z.gamma();

        assert!(result.re.is_infinite());
    }

    // ------------------------------------------------------------
    // complex known values
    // ------------------------------------------------------------

    #[test]
    fn gamma_i() {
        let z = c(0.0, 1.0);

        // mpmath 50 digits
        let expected = Complex::new(
            F106::new_add(
                -1.549498283018106854e-1,
                -1.197987421339115826e-17,
            ),
            F106::new_add(
                -4.980156681183560427e-1,
                -2.366356090576772302e-17,
            ),
        );

        assert_close_complex(
            z.gamma(),
            expected,
            1e-16,
        );
    }

    #[test]
    fn gamma_1_plus_i() {
        let z = c(1.0, 1.0);

        let expected = Complex::new(
            F106::new_add(
                 4.980156681183560427e-1,
                 2.366356090576772302e-17,
            ),
            F106::new_add(
                -1.549498283018106854e-1,
                -1.197987421339115826e-17,
            ),
        );

        assert_close_complex(
            z.gamma(),
            expected,
            1e-16,
        );
    }

    // ------------------------------------------------------------
    // recurrence relation
    // Γ(z+1)=zΓ(z)
    // ------------------------------------------------------------

    #[test]
    fn gamma_functional_equation() {
        let z = c(0.3, 0.7);

        let lhs = (z + Complex::from(f(1.0))).gamma();
        let rhs = z * z.gamma();

        assert_close_complex(
            lhs,
            rhs,
            1e-27,
        );
    }

    // ------------------------------------------------------------
    // reflection formula
    // Γ(z)Γ(1-z)=π/sin(πz)
    // ------------------------------------------------------------

    #[test]
    fn gamma_reflection_formula() {
        let z = c(0.3, 0.4);

        let lhs =
            z.gamma()
            * (Complex::from(f(1.0)) - z).gamma();

        let pi = Complex::from(F106::pi());

        let rhs =
            pi / (pi * z).sin();

        assert_close_complex(
            lhs,
            rhs,
            1e-16,
        );
    }

    // ------------------------------------------------------------
    // near pole
    // ------------------------------------------------------------

    #[test]
    fn gamma_near_zero() {
        let z = c(1e-20, 0.0);

        let lhs = (z + Complex::from(f(1.0))).gamma();
        let rhs = z * z.gamma();

        assert_close_complex(
            lhs,
            rhs,
            1e-24,
        );
    }

    // ------------------------------------------------------------
    // Taylor/Stirling boundary
    // ------------------------------------------------------------

    #[test]
    fn gamma_taylor_region() {
        let z = c(0.2, 0.5);

        let result = z.gamma();

        assert!(result.re.is_finite());
        assert!(result.im.is_finite());
    }

    #[test]
    fn gamma_stirling_region() {
        let z = c(10.0, 20.0);

        let result = z.gamma();

        assert!(result.re.is_finite());
        assert!(result.im.is_finite());
    }

    // ------------------------------------------------------------
    // large argument
    // ------------------------------------------------------------

    #[test]
    fn gamma_large_argument() {
        let z = c(50.0, 0.0);

        let result = z.gamma();

        assert!(result.re.is_finite());
    }
}
