use bitvec::prelude::*;
use num_complex::Complex;
use num_traits::Zero;
use rayon::prelude::*;
use std::ops::{
    Add,
};
use std::collections::VecDeque;
use crate::calculate::Func;

const UNCALCULATED: u16 = u16::MAX;

type PushedFlags = BitSlice<u8, Lsb0>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: i64,
    pub y: i64,
}

impl Coordinates {
    #[inline]
    const fn all_directions() -> [Self; 8] {
        [
            Self { x: -1, y: -1 }, Self { x: 0, y: -1 }, Self { x: 1, y: -1 },
            Self { x: -1, y:  0 },                       Self { x: 1, y:  0 },
            Self { x: -1, y:  1 }, Self { x:  0, y: 1 }, Self { x: 1, y:  1 }
        ]
    }

    /// # 指定された矩形内の座標かどうかを判定する
    ///
    /// ## Returns
    ///  - true: `{0, 0} <= self < {width, height}`
    ///  - false: `self < {0, 0}` or `{width, height} <= self`
    #[inline]
    fn is_in_rect(&self, width: i64, height: i64) -> bool {
        (0 <= self.x && self.x < width) && (0 <= self.y && self.y < height)
    }

    /// # 2次元配列用の引数[y][x]の組から、1次元配列用の引数[idx]を計算する
    #[inline]
    fn to_index(&self, width: i64) -> usize {
        (self.y * width + self.x) as usize
    }
}

impl Add for Coordinates {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

pub struct CalcInfo
{
    pub start:  Coordinates,
    pub width:  u32,
    pub height: u32,
    pub max_itr:u16,
    pub size:   f64,
    pub center: Complex<f64>,
    pub range:  f64,
    pub func:   Func,
    pub deriv:  Func,
    pub coeff:  Complex<f64>
}

impl CalcInfo
{
    /// ## Params
    ///  - x, y: top-left coordinates of rectangle
    ///  - w, h: width and height of rectangle
    ///  - max_iter: max iterator counter of newton-method loop
    ///  - size: the number of pixels in the complex plane axis
    ///  - center: center coordinates of whole complex plane
    ///  - range: the range value of whole complex plane axis (Δx = Δy)
    pub fn new(
        x: u32, y: u32, w: u32, h: u32,
        max_itr: u16, size: f64, center: Complex<f64>, range: f64,
        func: Func, deriv: Func,
        coeff: Complex<f64>,
    ) -> Self {
        Self {
            start: Coordinates{ x: x as i64, y: y as i64 },
            width: w, height: h,
            max_itr, size, center, range, func, deriv,
            coeff,
        }
    }

    #[inline]
    fn get_complex(&self, x: i64, y: i64) -> Complex<f64> {
        Complex::new(
            ((self.start.x + x) as f64 / self.size - 0.50) * self.range + self.center.re,
            ((self.start.y + y) as f64 / self.size - 0.50) * self.range + self.center.im
        )
    }
}

fn newton_method(z: Complex<f64>, a: Complex<f64>, func: &Func, deriv: &Func) -> Complex<f64>
{
    z - func(&[z]) / deriv(&[z]) * a
}

fn is_same(lhs: Complex<f64>, rhs: Complex<f64>, relative_error: f64) -> bool
{
    let delta = lhs - rhs;

    if !(lhs.re.is_zero() && lhs.im.is_zero()) {
        (delta / lhs).norm() < relative_error
    } else if !(rhs.re.is_zero() && rhs.im.is_zero()) {
        (delta / rhs).norm() < relative_error
    } else {
        true
    }
}

fn calc_escape_time(z: Complex<f64>, a: Complex<f64>, func: &Func, deriv: &Func, max_itr: u16) -> u16
{
    let mut z1 = z;
    const EPSILON: f64 = 10e-10;

    for n in 0..max_itr {
        let z2 = newton_method(z1, a, func, deriv);

        if !z2.is_finite() {
            return n;
        }
        if is_same(z1, z2, EPSILON) {
            return n;
        }

        z1 = z2;
    }

    max_itr
}

/// # 値をセットし、境界条件ならqueueに追加
#[inline]
fn update_boundary(
    buffer: &mut [u16],
    is_pushed: &mut PushedFlags,
    queue: &mut VecDeque<Coordinates>,
    idx: usize,
    prev_idx: usize,
    coord: Coordinates,
    val: u16,
) {
    buffer[idx] = val;
    if !is_pushed[idx] && (val != buffer[prev_idx]) {
        is_pushed.set(idx, true);
        queue.push_back(coord);
    }
}

fn calc_edge(
    buffer: &mut [u16],
    is_pushed: &mut PushedFlags,
    boundaries: &mut VecDeque<Coordinates>,
    info: &CalcInfo,
) {
    let w: i64 = info.width as i64;
    let h = info.height as i64;

    // 上辺 (y=0)
    let idx_start = 0;
    buffer[idx_start] = calc_escape_time(info.get_complex(0, 0), info.coeff, &info.func, &info.deriv, info.max_itr);
    for x in 1..w {
        let idx = x as usize;
        let val = calc_escape_time(info.get_complex(x, 0), info.coeff, &info.func, &info.deriv, info.max_itr);
        update_boundary(buffer, is_pushed, boundaries, idx, idx - 1, Coordinates { x, y: 0 }, val);
    }

    // 下辺 (y=h-1)
    let y_bottom = h - 1;
    let offset_bottom = (y_bottom * w) as usize;
    buffer[offset_bottom] = calc_escape_time(info.get_complex(0, y_bottom), info.coeff, &info.func, &info.deriv, info.max_itr);
    for x in 1..w {
        let idx = offset_bottom + x as usize;
        let val = calc_escape_time(info.get_complex(x, y_bottom), info.coeff, &info.func, &info.deriv, info.max_itr);
        update_boundary(buffer, is_pushed, boundaries, idx, idx - 1, Coordinates { x, y: y_bottom }, val);
    }

    // 右辺・左辺 (y=1..h-1)
    for y in 1..(h - 1) {
        for x in [0, w - 1] {
            let idx = ((y * w) + x) as usize;
            let idx_above = idx - w as usize ; // 上のマスと比較
            let val = calc_escape_time(info.get_complex(x, y), info.coeff, &info.func, &info.deriv, info.max_itr);
            update_boundary(buffer, is_pushed, boundaries, idx, idx_above, Coordinates { x, y }, val);
        }
    }
}

fn track_boundary(
    buffer: &mut [u16],
    is_pushed: &mut PushedFlags,
    boundaries: &mut VecDeque<Coordinates>,
    info: &CalcInfo,
) {
    let w = info.width as i64;
    let h = info.height as i64;

    while let Some(boundary) = boundaries.pop_back() {
        let boundary_val = buffer[boundary.to_index(w)];

        for d in Coordinates::all_directions() {
            let target = boundary + d;
            if !target.is_in_rect(w, h) {
                continue;
            }

            let idx = target.to_index(w);
            if buffer[idx] == UNCALCULATED {
                buffer[idx] = calc_escape_time(info.get_complex(target.x, target.y), info.coeff, &info.func, &info.deriv, info.max_itr);
            }
            if (buffer[idx] != boundary_val) && !is_pushed[idx] {
                is_pushed.set(idx, true);
                boundaries.push_back(target);
            }
        }
    }
}

fn fill_in_the_rest(buffer: &mut [u16], width: u32, height: u32)
{
    let w = width as usize;
    let h = height as usize;

    for y in 1..(h - 1) {
        let row_start = y * w;
        let mut fill_value = buffer[row_start];

        for x in 1..(w - 1) {
            let idx = row_start + x;
            match buffer[idx] {
                UNCALCULATED => buffer[idx] = fill_value,
                other => fill_value = other, // 計算済みのセルに当たったら、使用する値を更新
            }
        }
    }
}

pub fn calc_rect(info: CalcInfo) -> Vec<u16>
{
    let w = info.width as usize;
    let h = info.height as usize;
    let len = w * h;

    let mut boundaries = VecDeque::new();
    let mut is_pushed = bitvec![u8, Lsb0; 0 /* false  */; len];
    let mut buffer = vec![UNCALCULATED; len];

    calc_edge(&mut buffer, &mut is_pushed, &mut boundaries, &info);
    track_boundary(&mut buffer, &mut is_pushed, &mut boundaries, &info);
    fill_in_the_rest(&mut buffer, info.width, info.height);

    buffer
}