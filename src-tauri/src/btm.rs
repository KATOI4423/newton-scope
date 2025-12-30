use num_complex::Complex;
use num_traits::Zero;
use rayon::prelude::*;
use std::ops::{
    Add, AddAssign,
};
use std::collections::VecDeque;

use crate::calculate::Func;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: i64,
    pub y: i64,
}

impl Coordinates {
    const fn all_directions() -> [Self; 8] {
        [
            Self { x: -1, y: -1 }, Self { x: 0, y: -1 }, Self { x: 1, y: -1 },
            Self { x: -1, y:  0 },                       Self { x: 1, y:  0 },
            Self { x: -1, y:  1 }, Self { x:  0, y: 1 }, Self { x: 1, y:  1 }
        ]
    }

    /// # 指定された矩形領域内の座標かどうかを判定する
    ///
    /// ## Params
    ///  - min: 矩形領域の最小座標の組
    ///  - max: 矩形領域の最大座標の組
    ///
    /// # Returns
    ///  - true: `min <= self < max`
    ///  - false: `self < min` or `max <= self`
    fn is_in_rect(&self, min: &Self, max: &Self) -> bool {
        (min.x <= self.x && self.x < max.x) && (min.y <= self.y && self.y < max.y)
    }
}

impl Add for Coordinates {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Coordinates {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub struct CalcInfo
{
    pub start:  Coordinates,
    pub rect:   Coordinates,
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
            rect:  Coordinates{ x: w as i64, y: h as i64 },
            max_itr, size, center, range, func, deriv,
            coeff,
        }
    }

    fn x_axis(&self) -> Vec<f64> {
        (0..self.rect.x).into_iter().map(|idx|
            ((self.start.x + idx) as f64 / self.size - 0.50) * self.range + self.center.re
        ).collect()
    }

    fn y_axis(&self) -> Vec<f64> {
        (0..self.rect.y).into_iter().map(|idx|
            ((self.start.y + idx) as f64 / self.size - 0.50) * self.range + self.center.im
        ).collect()
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

fn push_boundary(
    boundaries: &mut VecDeque<Coordinates>,
    is_pushed: &mut Vec<Vec<bool>>,
    x: usize, y: usize,
) {
    if is_pushed[y][x] {
        return;
    }

    is_pushed[y][x] = true;
    boundaries.push_back(Coordinates { x: x as i64, y: y as i64 });
}

fn calc_edge(
    rect: &mut Vec<Vec<u16>>,
    is_pushed: &mut Vec<Vec<bool>>,
    boundaries: &mut VecDeque<Coordinates>,
    info: &CalcInfo,
    x_axis: &Vec<f64>,
    y_axis: &Vec<f64>,
) {
    let x_max = info.rect.x as usize - 1;
    let y_max = info.rect.y as usize - 1;

    rect[0][0] = calc_escape_time(
        Complex::new(x_axis[0], y_axis[0]),
        info.coeff, &info.func, &info.deriv, info.max_itr
    );
    rect[y_max][0] = calc_escape_time(
        Complex::new(x_axis[0], y_axis[y_max]),
        info.coeff, &info.func, &info.deriv, info.max_itr
    );

    for idx in 1..=x_max {
        rect[0][idx] = calc_escape_time(
            Complex::new(x_axis[idx], y_axis[0]),
            info.coeff, &info.func, &info.deriv, info.max_itr
        );
        if rect[0][idx] != rect[0][idx - 1] {
            push_boundary(boundaries, is_pushed, idx, 0);
        }

        rect[y_max][idx] = calc_escape_time(
            Complex::new(x_axis[idx], y_axis[y_max]),
            info.coeff, &info.func, &info.deriv, info.max_itr
        );
        if rect[y_max][idx] != rect[y_max][idx - 1] {
            push_boundary(boundaries, is_pushed, idx, y_max);
        }
    }

    for idx in 1..y_max {
        rect[idx][0] = calc_escape_time(
            Complex::new(x_axis[0], y_axis[idx]),
            info.coeff, &info.func, &info.deriv, info.max_itr
        );
        if rect[idx][0] != rect[idx - 1][0] {
            push_boundary(boundaries, is_pushed, 0, idx);
        }

        rect[idx][x_max] = calc_escape_time(
            Complex::new(x_axis[x_max], y_axis[idx]),
            info.coeff, &info.func, &info.deriv, info.max_itr
        );
        if rect[idx][x_max] != rect[idx - 1][x_max] {
            push_boundary(boundaries, is_pushed, x_max, idx);
        }
    }
}

fn track_boundary(
    rect: &mut Vec<Vec<u16>>,
    is_pushed: &mut Vec<Vec<bool>>,
    boundaries: &mut VecDeque<Coordinates>,
    info: &CalcInfo,
    x_axis: &Vec<f64>,
    y_axis: &Vec<f64>,
) {
    const MIN: Coordinates = Coordinates { x: 1, y: 1 }; // { x:0, y:0 } は `calc_edge` で計算済みのため、{ 1, 1 } から計算に使用する
    let max = Coordinates { x: info.rect.x, y: info.rect.y };

    while let Some(boundary) = boundaries.pop_back() {
        for d in Coordinates::all_directions() {
            let target = boundary + d;
            if !target.is_in_rect(&MIN, &max) {
                continue;
            }

            let (x, y) = (target.x as usize, target.y as usize);
            if rect[y][x] == 0 /* default value */ {
                rect[y][x] = calc_escape_time(
                    Complex::new(x_axis[x], y_axis[y]),
                    info.coeff, &info.func, &info.deriv, info.max_itr
                );
            }
            if !is_pushed[y][x] && (rect[y][x] != rect[y][x - 1]) {
                push_boundary(boundaries, is_pushed, x, y);
            }
        }
    }
}

fn fill_in_the_rest(rect: &mut Vec<Vec<u16>>)
{
    for y in 1..(rect.len() - 1) {
        let mut boundary = rect[y][0];
        for x in 1..(rect[y].len() - 1) {
            match rect[y][x] {
                0
                    => rect[y][x] = boundary,
                value if value == boundary
                    => (), // nothing to do
                other
                    => boundary = other, // 境界値を更新
            }
        }
    }
}

fn from(matrix: &mut Vec<Vec<u16>>) -> Vec<u16>
{
    match matrix.len() {
        0 => Vec::new(),
        _ => {
            let mut ret = Vec::with_capacity(matrix.len() * matrix[0].len());
            for col in matrix.iter() {
                for val in col.iter() {
                    ret.push(*val);
                }
            }
            ret
        }
    }
}

pub fn calc_rect(info: CalcInfo) -> Vec<u16>
{
    let mut boundaries = VecDeque::new();
    let mut is_pushed = vec![vec![false; info.rect.x.try_into().unwrap()]; info.rect.y.try_into().unwrap()];
    let mut rect = vec![vec![0; info.rect.x.try_into().unwrap()]; info.rect.y.try_into().unwrap()];
    let x = info.x_axis();
    let y = info.y_axis();

    calc_edge(&mut rect, &mut is_pushed, &mut boundaries, &info, &x, &y);
    track_boundary(&mut rect, &mut is_pushed, &mut boundaries, &info, &x, &y);
    fill_in_the_rest(&mut rect);

    from(&mut rect)
}