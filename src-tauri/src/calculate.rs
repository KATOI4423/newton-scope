use formulac;
use num_complex::Complex;
use num_traits::{
    Float,
    FromPrimitive,
};
use once_cell::sync::Lazy;
use serde::{
    Serialize, Deserialize,
};
use std::sync::{
    Arc,
    Mutex,
};

use crate::btm;

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
pub type Func = Arc<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static>;

/// formulacの変数を保持する構造体
struct Formulac
{
    vars: formulac::Variables,
    usrs: formulac::UserDefinedTable,
    f: Func,
    df: Func,
}

impl Formulac {
    fn new() -> Self { std::default::Default::default() }

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

        self.f = Arc::new({
            let f_holder = FuncHolder { func: f_arc.clone() };
            move |args| f_holder.call(args)
        });
        self.df = Arc::new({
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

impl Default for Formulac {
    fn default() -> Self {
        Self {
            vars: formulac::Variables::new(),
            usrs: formulac::UserDefinedTable::new(),
            f: Arc::new(|_: &[Complex<f64>]| Complex::ZERO),
            df: Arc::new(|_: &[Complex<f64>]| Complex::ZERO),
        }
    }
}

/// 複素数平面の情報を保持する構造体
#[derive(Serialize, Deserialize)]
struct Canvas<T>
    where T: Float + FromPrimitive,
{
    center: num_complex::Complex<T>,
    zoom_level:  i32,
    size: u16,
}

impl<T: Float + FromPrimitive> Canvas<T> {
    fn new() -> Self { std::default::Default::default() }

    fn set_center(&mut self, re: T, im: T) {
        self.center.re = re;
        self.center.im = im;
    }

    fn set_size(&mut self, size: u16) {
        self.size = size;
    }

    /// # ズーム後にマウス位置が動かないようにズームする
    ///
    /// ## Params
    ///  - level: ズーム段階
    ///  - mouse_x_ratio: マウスのx座標 [0.0, 1.0]
    ///  - mouse_y_ratio: マウスのy座標 [0.0, 1.0]
    fn zoom_around_point(&mut self, level: i32, mouse_x_ratio: f64, mouse_y_ratio: f64) {
        let old_width = self.width();

        self.zoom(level);
        let new_width = self.width();

        let d_width = old_width - new_width;

        // 座標中心は0.5なので、マウスの座標の偏差によって補正する
        let delta = Complex::new(
            d_width * T::from_f64(mouse_x_ratio - 0.5).unwrap(),
            d_width * T::from_f64(0.5 - mouse_y_ratio).unwrap()
        );

        self.center = self.center + delta;
    }

    fn center(&self) -> num_complex::Complex<T> {
        self.center
    }

    fn size(&self) -> u16 {
        self.size
    }

    fn zoom(&mut self, level: i32) {
        self.zoom_level += level;
    }

    fn zoom_step() -> f64 {
        const STEP:f64 = 1.0 / 8.0;
        STEP
    }

    fn width(&self) -> T {
        // 2.0 * 2.0^(-zoom_level * zoom_step) = 2.0^(-zoom_level * zoom_step + 1)
        //  └> [-1.0, 1.0].width() = 2.0
        T::from_f64(2.0f64.powf(-self.zoom_level as f64 * Self::zoom_step() + 1.0))
            .unwrap_or_else(|| T::nan())
    }

    fn scale(&self) -> T {
        T::from_f64(2.0f64.powf(self.zoom_level as f64 * Self::zoom_step()))
            .unwrap_or_else(|| T::nan())
    }
}

impl<T: Float + FromPrimitive> Default for Canvas<T>
{
    fn default() -> Self {
        Self {
            center: num_complex::Complex::<T>::new(T::zero(), T::zero()),
            zoom_level: default::CANVAS_ZOOM_LEVEL,
            size: default::CANVAS_SIZE,
        }
    }
}


/// フラクタル計算に使用する情報を保持する構造体
#[derive(Serialize, Deserialize)]
struct Fractal {
    #[serde(skip)] // 数式文字列の情報のみで良いだめ、Formulacはserializeしない
    formulac:   Formulac,
    canvas:     Canvas<f64>,
    max_iter:   u16,
}


impl Fractal {
    fn new() -> Self { std::default::Default::default() }

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

impl Default for Fractal {
    fn default() -> Self {
        Self {
            formulac:   Formulac::new(),
            canvas:     Canvas::new(),
            max_iter:   default::FRACTAL_MAX_ITER,
        }
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
pub fn get_size() -> i32 {
    FRACTAL.lock().unwrap()
        .canvas().size().into()
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

#[tauri::command]
pub fn set_size(size: u16) {
    FRACTAL.lock().unwrap()
        .canvas_mut().set_size(size);
}

/// 中心座標を移動させる
#[tauri::command]
pub fn move_view(dx: f64, dy: f64) {
    let mut fractal = FRACTAL.lock().unwrap();
    let width = fractal.canvas().width();
    let center = fractal.canvas().center();

    fractal.canvas_mut().set_center(
        center.re - dx * width,
        center.im + dy * width
    );
}

/// # 縮尺を変更する
///
/// # Returns:
/// - 成功: "OK"
/// - エラー: "<エラーメッセージ>"
#[tauri::command]
pub fn zoom_view(level: i32, x: f64, y: f64) {
    let mut fractal = FRACTAL.lock().unwrap();

    fractal.canvas_mut().zoom_around_point(level, x, y);
}

/// # 指定された矩形領域のデータのみを生成して返す
///
/// ## Params
///  - x: 矩形領域の左上のX座標（canvas全体に対するオフセット）
///  - y: 矩形領域の左上のY座標（canvas全体に対するオフセット）
///  - w: 矩形領域の幅
///  - h: 矩形領域の高さ
#[tauri::command]
pub async fn render_tile(x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u16>, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let info = {
            let fractal = FRACTAL.lock().unwrap();
            btm::CalcInfo::new(
                x, y, w, h,
                fractal.max_iter(),
                fractal.canvas().size() as f64, // キャスト回数削減のため、最初からf64で取る
                fractal.canvas().center(),
                fractal.canvas().width(),
                fractal.formulac().func().clone(),
                fractal.formulac().deriv().clone(),
                Complex::ONE, // TODO: UIから変更できるようにする？
            )
        };

        btm::calc_rect(info)
    }).await;

    match result {
        Ok(data) => Ok(data),
        Err(e) => Err(e.to_string())
    }
}
