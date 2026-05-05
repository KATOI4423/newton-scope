//! mod.rs
//!
//! 多倍長浮動小数点をformulacで使用可能にするためのラッパー関数群

pub(crate) mod dashu;
pub(crate) mod f128;

pub(crate) type MD<const N: usize> = dashu::MD<N>;
pub(crate) type F128 = f128::F128;
