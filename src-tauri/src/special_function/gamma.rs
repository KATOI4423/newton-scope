//! gamma.rs
//!
//! 複素数引数のGamma関数
//! 参考： shikino, https://slpr.sakura.ne.jp/qp/

use formulac::core::Real;
use num_complex::{Complex, ComplexFloat};
use num_traits::{
    ToPrimitive, Zero
};

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
        let mut r = Self::from(COEF.last().unwrap());
        for coef in COEF[..iter_max].iter().rev() {
            r = r * w + coef;
        }

        s / (r * w)
    }

    fn gamma_by_stirling_series(&self) -> Self {
        let mut s = Self::ONE;
        let mut q = *self;
        if self.abs() < 9.0 {
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

        let mut r = Self::from(BERNOULLI.last().unwrap());
        for bernoulli in BERNOULLI[..BERNOULLI.len()-1].iter().rev() {
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
