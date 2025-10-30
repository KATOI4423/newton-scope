use formulac;
use num_complex::Complex;
use num_traits::{
    Float,
    FromPrimitive, ToPrimitive,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// formulacが生成する関数オブジェクトを保持する型
macro_rules! FORMULAC_RETURN_TYPE {
    () => { Box<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static> };
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
    funcs: [Func; 4],
}

impl Formulac {
    fn new() -> Self {
        Self {
            vars: formulac::Variables::new(),
            usrs: formulac::UserDefinedTable::new(),
            funcs: [Func::new(), Func::new(), Func::new(), Func::new()],
        }
    }

    fn set_vars(&mut self, vars: &[(&str, Complex<f64>)]) {
        self.vars.insert(vars);
    }

    fn set_usrs(&mut self, function_list: &[(&str, formulac::UserDefinedFunction)]) {
        for (key, func) in function_list {
            self.usrs.register(*key, func.clone());
        }
    }

    fn set_formula(&mut self, formula: &str) -> Result<(), String> {
        self.funcs[0] = Func::from(formulac::compile(
            formula, &["z"], &self.vars, &self.usrs)?
        );
        self.funcs[1] = Func::from(formulac::compile(
            &format!("diff({}, z)", formula),
            &["z"], &self.vars, &self.usrs)?
        );
        self.funcs[2] = Func::from(formulac::compile(
            &format!("diff({}, z, 2)", formula),
            &["z"], &self.vars, &self.usrs)?
        );
        self.funcs[3] = Func::from(formulac::compile(
            &format!("diff({}, z, 3)", formula),
            &["z"], &self.vars, &self.usrs)?
        );
        Ok(())
    }

    fn funcs(&self) -> &[Func; 4] {
        &self.funcs
    }
}


/// 複素数平面の情報を保持する構造体
struct Canvas<T> 
    where T: Float + FromPrimitive,
{
    center: num_complex::Complex<T>,
    scale:  T,
    size:   u16, // up to 65,535
}

impl<T: Float + FromPrimitive> Canvas<T> {
    fn new() -> Self {
        Self {
            center: num_complex::Complex::<T>::new(T::zero(), T::zero()),
            scale:  T::zero(),
            size:   512,
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

    fn set_size(&mut self, size: u16) {
        self.size = size;
    }

    fn size(&self) -> u16 {
        self.size
    }
}


/// フラクタル計算に使用する情報を保持する構造体
struct Fractal {
    formulac:   Formulac,
    canvas:     Canvas<f64>,
    max_iter:   u16, // up to 65,535
}

impl Fractal {
    fn new() -> Self {
        Self {
            formulac:   Formulac::new(),
            canvas:     Canvas::new(),
            max_iter:   64,
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

    fn set_max_iter(&mut self, max_iter: u16) {
        self.max_iter = max_iter;
    }

    fn max_iter(&self) -> u16 {
        self.max_iter
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
pub fn get_size() -> i32 {
    FRACTAL.lock().unwrap()
        .canvas().size().to_i32().unwrap() // The conversion u16 -> i32 never fails
}

#[tauri::command]
pub fn get_max_iter() -> i32 {
    FRACTAL.lock().unwrap()
        .max_iter().to_i32().unwrap() // The conversion u16 -> i32 never fails
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
