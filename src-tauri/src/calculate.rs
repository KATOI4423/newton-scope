use formulac;
use num_complex::Complex;
use num_traits::{
    Float,
    FromPrimitive,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// formulacが生成する関数オブジェクトの型
type Func = dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static;

/// formulacの変数を保持する構造体
struct Formulac {
    vars: formulac::Variables,
    usrs: formulac::UserDefinedTable,
    func: Box<Func>,
}

impl Formulac {
    fn new() -> Self {
        Self {
            vars: formulac::Variables::new(),
            usrs: formulac::UserDefinedTable::new(),
            func: Box::new(|_| Complex::ZERO),
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
        self.func = Box::new(formulac::compile(formula, &["z"], &self.vars, &self.usrs)?);
        Ok(())
    }

    fn func(&self) -> &Func {
        self.func.as_ref()
    }
}

static FORMULAC: Lazy<Mutex<Formulac>> = Lazy::new(|| {
    Mutex::new(Formulac::new())
});

/// FORMULACの初期化関数
fn initialize_formulac() {
    let formulac = Formulac::new();

    *FORMULAC.lock().unwrap() = formulac;
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

    fn canvas(&self) -> &Canvas<f64> {
        &self.canvas
    }

    fn set_max_iter(&mut self, max_iter: u16) {
        self.max_iter = max_iter;
    }

    fn max_iter(&self) -> u16 {
        self.max_iter
    }
}


#[tauri::command]
pub fn initialize() {
    initialize_formulac();
}

/// 数式をformulacに設定する
/// 
/// # Returns:
/// - 成功: "OK"
/// - エラー: "<エラーメッセージ>"
#[tauri::command]
pub fn set_formula(formula: String) -> String {
    let mut f = FORMULAC.lock().unwrap();

    match f.set_formula(&formula) {
        Ok(_) => "OK".to_string(),
        Err(err) => err,
    }
}
