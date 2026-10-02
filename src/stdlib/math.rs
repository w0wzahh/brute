use std::collections::HashMap;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

fn num(args: &[Value], i: usize) -> Result<f64> {
    match args.get(i) {
        Some(Value::Int(n))   => Ok(*n as f64),
        Some(Value::Float(n)) => Ok(*n),
        other => Err(BruteError::TypeError(format!(
            "expected a number, got {}",
            other.map(|v| v.type_name()).unwrap_or("none")))),
    }
}

fn int(args: &[Value], i: usize) -> Result<i64> {
    match args.get(i) {
        Some(Value::Int(n)) => Ok(*n),
        other => Err(BruteError::TypeError(format!(
            "expected an int, got {}",
            other.map(|v| v.type_name()).unwrap_or("none")))),
    }
}

/// Returns a map of name → Value for the math module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("math::", $name).into(),
                func: $body,
            });
        };
    }
    macro_rules! unary {
        ($name:expr, $op:expr) => {
            f!($name, |_, a| Ok(Value::Float(($op)(num(&a, 0)?))));
        };
    }

    unary!("sqrt",  |x: f64| x.sqrt());
    unary!("cbrt",  |x: f64| x.cbrt());
    unary!("abs",   |x: f64| x.abs());
    unary!("floor", |x: f64| x.floor());
    unary!("ceil",  |x: f64| x.ceil());
    unary!("round", |x: f64| x.round());
    unary!("trunc", |x: f64| x.trunc());
    unary!("sin",   |x: f64| x.sin());
    unary!("cos",   |x: f64| x.cos());
    unary!("tan",   |x: f64| x.tan());
    unary!("asin",  |x: f64| x.asin());
    unary!("acos",  |x: f64| x.acos());
    unary!("atan",  |x: f64| x.atan());
    unary!("exp",   |x: f64| x.exp());
    unary!("ln",    |x: f64| x.ln());
    unary!("log2",  |x: f64| x.log2());
    unary!("log10", |x: f64| x.log10());
    unary!("signum",|x: f64| x.signum());

    f!("pow", |_, a| Ok(Value::Float(num(&a, 0)?.powf(num(&a, 1)?))));
    f!("atan2", |_, a| Ok(Value::Float(num(&a, 0)?.atan2(num(&a, 1)?))));
    f!("hypot", |_, a| Ok(Value::Float(num(&a, 0)?.hypot(num(&a, 1)?))));
    f!("log", |_, a| Ok(Value::Float(num(&a, 0)?.log(num(&a, 1)?))));
    f!("min", |_, a| Ok(Value::Float(num(&a, 0)?.min(num(&a, 1)?))));
    f!("max", |_, a| Ok(Value::Float(num(&a, 0)?.max(num(&a, 1)?))));
    f!("clamp", |_, a| Ok(Value::Float(num(&a, 0)?.clamp(num(&a, 1)?, num(&a, 2)?))));

    f!("factorial", |_, a| {
        let n = int(&a, 0)?;
        if n < 0 { return Err(BruteError::ValueError("factorial of negative".into())); }
        Ok(Value::Int((1..=n).product()))
    });
    f!("gcd", |_, a| {
        fn g(a: i64, b: i64) -> i64 { if b == 0 { a.abs() } else { g(b, a % b) } }
        Ok(Value::Int(g(int(&a, 0)?, int(&a, 1)?)))
    });
    f!("lcm", |_, a| {
        fn g(a: i64, b: i64) -> i64 { if b == 0 { a.abs() } else { g(b, a % b) } }
        let (x, y) = (int(&a, 0)?, int(&a, 1)?);
        if x == 0 || y == 0 { return Ok(Value::Int(0)); }
        Ok(Value::Int((x / g(x, y)).abs() * y.abs()))
    });
    f!("is_prime", |_, a| {
        let n = int(&a, 0)?;
        if n < 2 { return Ok(Value::Bool(false)); }
        let mut i = 2i64;
        while i * i <= n { if n % i == 0 { return Ok(Value::Bool(false)); } i += 1; }
        Ok(Value::Bool(true))
    });
    f!("random", |_, _| Ok(Value::Float(rand::random::<f64>())));
    f!("random_int", |_, a| {
        let (lo, hi) = (int(&a, 0)?, int(&a, 1)?);
        if hi <= lo { return Err(BruteError::ValueError("random_int: hi must exceed lo".into())); }
        use rand::Rng;
        Ok(Value::Int(rand::thread_rng().gen_range(lo..hi)))
    });

    // Constants
    m.insert("PI".into(),      Value::Float(std::f64::consts::PI));
    m.insert("E".into(),       Value::Float(std::f64::consts::E));
    m.insert("TAU".into(),     Value::Float(std::f64::consts::TAU));
    m.insert("INF".into(),     Value::Float(f64::INFINITY));
    m.insert("NAN".into(),     Value::Float(f64::NAN));
    m.insert("SQRT_2".into(),  Value::Float(std::f64::consts::SQRT_2));

    m
}
