//! mod.rs
//!
//! 多倍長浮動小数点をformulacで使用可能にするためのラッパー関数群

pub(crate) mod dashu;
pub(crate) mod twofloat;

pub(crate) type MD<const N: usize> = dashu::MD<N>;
pub(crate) type F106 = twofloat::F106;
