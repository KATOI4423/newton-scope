use formulac;
use num_complex::Complex;
use num_traits::{
    Float,
    FromPrimitive,
};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::sync::{
    Arc,
    Mutex,
};

/// 初期値
mod default {
    pub const FORMULA: &str = "z^3 - 1";
    pub const CANVAS_ZOOM_LEVEL: i32 = 0;
    pub const CANVAS_SIZE: u16 = 512;
    pub const FRACTAL_MAX_ITER: u16 = 128;
}

/// 静的ディスパッチ用ラッパ
struct FuncHolder<F>
where
    F: Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static,
{
    func: Arc<F>,
}

impl<F> FuncHolder<F>
where
    F: Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static,
{
    fn call(&self, args: &[Complex<f64>]) -> Complex<f64> {
        (self.func)(args)
    }
}

/// Formulacが生成する匿名関数を保持する
type Func = Box<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static>;

/// formulacの変数を保持する構造体
struct Formulac
{
    vars: formulac::Variables,
    usrs: formulac::UserDefinedTable,
    f: Func,
    df: Func,
}

impl Formulac {
    fn new() -> Self {
        Self {
            vars: formulac::Variables::new(),
            usrs: formulac::UserDefinedTable::new(),
            f: Box::new(|_: &[Complex<f64>]| Complex::ZERO),
            df: Box::new(|_: &[Complex<f64>]| Complex::ZERO),
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
        let f = formulac::compile(formula, &["z"], &self.vars, &self.usrs)?;
        let df = formulac::compile(
            &format!("diff({}, z)", formula), &["z"], &self.vars, &self.usrs
        )?;

        let f_arc = Arc::new(f);
        let df_arc = Arc::new(df);

        self.f = Box::new({
            let f_holder = FuncHolder { func: f_arc.clone() };
            move |args| f_holder.call(args)
        });
        self.df = Box::new({
            let df_holder = FuncHolder { func: df_arc.clone() };
            move |args| df_holder.call(args)
        });

        Ok(())
    }

    fn func(&self) -> &Func {
        &self.f
    }

    fn deriv(&self) -> &Func {
        &self.df
    }
}


/// 複素数平面の情報を保持する構造体
struct Canvas<T> 
    where T: Float + FromPrimitive,
{
    center: num_complex::Complex<T>,
    zoom_level:  i32,
}

impl<T: Float + FromPrimitive> Canvas<T> {
    fn new() -> Self {
        Self {
            center: num_complex::Complex::<T>::new(T::zero(), T::zero()),
            zoom_level: default::CANVAS_ZOOM_LEVEL,
        }
    }

    fn set_center(&mut self, re: T, im: T) {
        self.center.re = re;
        self.center.im = im;
    }

    fn center(&self) -> num_complex::Complex<T> {
        self.center
    }

    fn zoom(&mut self, level: i32) {
        self.zoom_level += level;
    }

    fn scale(&self) -> f64 {
        const STEP: f64 = 1.0 / 8.0;
        2.0f64.powf(self.zoom_level as f64 * STEP)
    }
}


/// フラクタル計算に使用する情報を保持する構造体
struct Fractal {
    formulac:   Formulac,
    canvas:     Canvas<f64>,
    max_iter:   u16,
}


impl Fractal {
    fn new() -> Self {
        Self {
            formulac:   Formulac::new(),
            canvas:     Canvas::new(),
            max_iter:   default::FRACTAL_MAX_ITER,
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

    fn canvas_mut(&mut self) -> &mut Canvas<f64> {
        &mut self.canvas
    }

    fn set_max_iter(&mut self, max_iter: u16) {
        self.max_iter = max_iter;
    }

    fn max_iter(&self) -> u16 {
        self.max_iter
    }
}


static FRACTAL: Lazy<Mutex<Fractal>> = Lazy::new(|| {
    Mutex::new(Fractal::new())
});

/// FRACTALの初期化関数
#[tauri::command]
pub fn initialize() {
    let mut fractal = Fractal::new();
    fractal.formulac_mut().set_formula(default::FORMULA).unwrap();

    *FRACTAL.lock().unwrap() = fractal;
}

#[tauri::command]
pub fn get_default_formula() -> String {
    default::FORMULA.to_string()
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
    format!("{}", format_with_decimal(FRACTAL.lock().unwrap().canvas().scale()))
}

#[tauri::command]
pub fn get_default_size() -> i32 {
    default::CANVAS_SIZE.into()
}

#[tauri::command]
pub fn get_default_max_iter() -> i32 {
    default::FRACTAL_MAX_ITER.into()
}

/// 数式をformulacに設定する
/// 
/// # Returns:
/// - 成功: "OK"
/// - エラー: "<エラーメッセージ>"
#[tauri::command]
pub async fn set_formula(formula: String) -> String {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut fractal = FRACTAL.lock().unwrap();
        match fractal.formulac_mut().set_formula(&formula) {
            Ok(_) => "OK".to_string(),
            Err(e) => e.to_string()
        }
    })
    .await;

    match result {
        Ok(ok) => ok,
        Err(e) => e.to_string(),
    }
}

#[tauri::command]
pub fn set_max_iter(max_iter: u16) {
    FRACTAL.lock().unwrap().set_max_iter(max_iter);
}

/// 中心座標を移動させる
#[tauri::command]
pub fn move_view(dx: f64, dy: f64) {
    let mut fractal = FRACTAL.lock().unwrap();
    let scale = fractal.canvas().scale();
    let center = fractal.canvas().center();
    const WIDTH: f64 = 2.0; // [-1: 1]の幅

    fractal.canvas_mut().set_center(
        center.re - dx * scale * WIDTH,
        center.im + dy * scale * WIDTH
    );
}

/// 縮尺を変更する
#[tauri::command]
pub fn zoom_view(level: i32) {
    let mut fractal = FRACTAL.lock().unwrap();
    fractal.canvas_mut().zoom(level);
}

#[tauri::command]
pub fn generate_test_data(tile_size: usize, max_iter: u16) -> Vec<u16> {
    let (center, scale) = {
        let fractal = FRACTAL.lock().unwrap();
        (fractal.canvas().center(), fractal.canvas().scale())
    };
    let mut data = vec![0u16; tile_size * tile_size];
    const PI2: f64 = std::f64::consts::PI * 2.0;

    for y in 0..tile_size {
        let y_val = PI2 * (y as f64 / tile_size as f64) * scale + center.im;
        let a_y = y_val.sin() + 1.0;
        for x in 0..tile_size {
            let x_val = PI2 * (x as f64 / tile_size as f64) * scale + center.re;
            let a_x = x_val.sin() + 1.0;
            let val = max_iter as f64 * a_x * a_y / 4.0;
            data[y * tile_size + x] = val as u16;
        }
    }

    data
}
