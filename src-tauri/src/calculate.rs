use formulac;
use num_complex::Complex;
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
