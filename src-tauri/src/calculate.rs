use formulac;
use num_complex::Complex;
use num_traits::{
    Float,
    FromPrimitive, ToPrimitive,
};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::sync::Mutex;

/// formulacが生成する関数オブジェクトを保持する型
macro_rules! FORMULAC_RETURN_TYPE {
    () => { Box<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static> };
}

/// 初期値
mod default {
    pub static FORMULAC_FUNCS_LEN: usize = 8;
    pub static CANVAS_SCALE: f64 = 1.0;
    pub static CANVAS_SIZE: u16 = 512;
    pub static FRACTAL_MAX_ITER: u16 = 128;
}

/// formulacが生成する関数オブジェクトを静的ディスパッチで配列に格納するために、Enumを定義する
enum Func {
    F(FORMULAC_RETURN_TYPE!()),
}

impl Func {
    #[inline(always)]
    fn call(&self, args: &[Complex<f64>]) -> Complex<f64> {
        match self {
            Self::F(func) => func(args)
        }
    }

    fn new() -> Self {
        Self::F(Box::new(formulac::compile(
            "z", &["z"],
            &formulac::Variables::new(),
            &formulac::UserDefinedTable::new()
        ).unwrap()))
    }

    fn from(func: impl Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static) -> Self {
        Func::F(Box::new(func))
    }
}

/// formulacの変数を保持する構造体
struct Formulac {
    vars: formulac::Variables,
    usrs: formulac::UserDefinedTable,
    funcs: [Func; default::FORMULAC_FUNCS_LEN],
}

impl Formulac {
    fn new() -> Self {
        Self {
            vars: formulac::Variables::new(),
            usrs: formulac::UserDefinedTable::new(),
            funcs: core::array::from_fn(|_| Func::new()),
        }
    }

    #[allow(dead_code)]
    fn set_vars(&mut self, vars: &[(&str, Complex<f64>)]) {
        self.vars.insert(vars);
    }

    #[allow(dead_code)]
    fn set_usrs(&mut self, function_list: &[(&str, formulac::UserDefinedFunction)]) {
        for (key, func) in function_list {
            self.usrs.register(*key, func.clone());
        }
    }

    fn set_formula(&mut self, formula: &str) -> Result<(), String> {
        let funcs: Result<Vec<Func>, String> = (0..default::FORMULAC_FUNCS_LEN)
            .into_par_iter()
            .map(|i| {
                match i {
                    0 => Ok(Func::from(
                        formulac::compile(formula, &["z"], &self.vars, &self.usrs)?
                    )),
                    i => Ok(Func::from(
                        formulac::compile(
                    &format!("diff({}, z, {})", formula, i),
                    &["z"], &self.vars, &self.usrs)?
                    ))
                }
            })
            .collect();

        self.funcs = match funcs?.try_into() {
            Ok(array) => array,
            Err(_) => unreachable!("length never changed"),
        };

        Ok(())
    }

    fn funcs(&self) -> &[Func; default::FORMULAC_FUNCS_LEN] {
        &self.funcs
    }
}


/// 複素数平面の情報を保持する構造体
struct Canvas<T> 
    where T: Float + FromPrimitive,
{
    center: num_complex::Complex<T>,
    scale:  T,
}

impl<T: Float + FromPrimitive> Canvas<T> {
    fn new() -> Self {
        Self {
            center: num_complex::Complex::<T>::new(T::zero(), T::zero()),
            scale:  T::from(default::CANVAS_SCALE).unwrap(),
        }
    }

    fn set_center(&mut self, re: T, im: T) {
        self.center.re = re;
        self.center.im = im;
    }

    fn center(&self) -> num_complex::Complex<T> {
        self.center
    }

    fn set_scale(&mut self, scale: T) {
        self.scale = scale;
    }

    fn scale(&self) -> T {
        self.scale
    }
}


/// フラクタル計算に使用する情報を保持する構造体
struct Fractal {
    formulac:   Formulac,
    canvas:     Canvas<f64>,
}


impl Fractal {
    fn new() -> Self {
        Self {
            formulac:   Formulac::new(),
            canvas:     Canvas::new(),
        }
    }

    fn formulac(&self) -> &Formulac {
        &self.formulac
    }

    fn formulac_mut(&mut self) ->&mut Formulac {
        &mut self.formulac
    }

    fn canvas(&self) -> &Canvas<f64> {
        &self.canvas
    }

    fn canvas_mut(&mut self) -> &Canvas<f64> {
        &mut self.canvas
    }
}

/// 初期数式
static FORMULA: &str = "z^3 - 1";

static FRACTAL: Lazy<Mutex<Fractal>> = Lazy::new(|| {
    Mutex::new(Fractal::new())
});

/// FRACTALの初期化関数
#[tauri::command]
pub fn initialize() {
    let mut fractal = Fractal::new();
    fractal.formulac_mut().set_formula(FORMULA).unwrap();

    *FRACTAL.lock().unwrap() = fractal;
}

#[tauri::command]
pub fn get_default_formula() -> String {
    FORMULA.to_string()
}

/// 指数表記の際に、小数点がない場合は ".0" を追加する
fn format_with_decimal(x: f64) -> String {
    let s = format!("{:e}", x);
    if s.contains('.') {
        s
    } else {
        s.replacen('e', ".0e", 1)
    }
}

#[tauri::command]
pub fn get_center_str() -> String {
    let center = FRACTAL.lock().unwrap()
        .canvas().center();
    format!("({re}, {im})",
        re = format_with_decimal(center.re),
        im = format_with_decimal(center.im)
    )
}

#[tauri::command]
pub fn get_scale_str() -> String {
    format!("{}", format_with_decimal(FRACTAL.lock().unwrap().canvas.scale()))
}

#[tauri::command]
pub fn get_default_size() -> i32 {
    default::CANVAS_SIZE.into()
}

#[tauri::command]
pub fn get_default_max_iter() -> i32 {
    default::FRACTAL_MAX_ITER.into()
}

#[tauri::command]
pub fn get_coeffs() -> Vec<f32> {
    let f = FRACTAL.lock().unwrap();
    let scale = f.canvas().scale();
    let center = f.canvas().center();
    let funcs = f.formulac().funcs();
    let mut s = 1.0;

    let mut coeffs: Vec<f32> = Vec::with_capacity(funcs.len() * 2);
    for func in funcs {
        let coeff = func.call(&[center]) * s;
        coeffs.push(coeff.re.to_f32()
            .expect(&format!("Failed to cast to f32")));
        coeffs.push(coeff.im.to_f32()
            .expect(&format!("Failed to cast to f32")));
        s *= scale;
    }

    coeffs
}

/// 数式をformulacに設定する
/// 
/// # Returns:
/// - 成功: "OK"
/// - エラー: "<エラーメッセージ>"
#[tauri::command]
pub fn set_formula(formula: String) -> String {
    let mut fractal = FRACTAL.lock().unwrap();

    match fractal.formulac_mut().set_formula(&formula) {
        Ok(_) => "OK".to_string(),
        Err(err) => err,
    }
}
