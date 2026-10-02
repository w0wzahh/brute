use std::collections::HashMap;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

fn s(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.display()).unwrap_or_default()
}

/// Returns a map of name → Value for the string module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("string::", $name).into(),
                func: $body,
            });
        };
    }

    f!("to_uppercase", |_, a| Ok(Value::String(s(&a, 0).to_uppercase())));
    f!("to_lowercase", |_, a| Ok(Value::String(s(&a, 0).to_lowercase())));
    f!("trim",         |_, a| Ok(Value::String(s(&a, 0).trim().to_string())));
    f!("trim_start",   |_, a| Ok(Value::String(s(&a, 0).trim_start().to_string())));
    f!("trim_end",     |_, a| Ok(Value::String(s(&a, 0).trim_end().to_string())));
    f!("reverse",      |_, a| Ok(Value::String(s(&a, 0).chars().rev().collect())));
    f!("capitalize",   |_, a| {
        let st = s(&a, 0);
        let mut c = st.chars();
        Ok(Value::String(match c.next() {
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }))
    });
    f!("repeat", |_, a| {
        let n = match a.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
        Ok(Value::String(s(&a, 0).repeat(n.max(0) as usize)))
    });
    f!("split", |_, a| {
        let sep = s(&a, 1);
        Ok(Value::List(s(&a, 0).split(&*sep).map(|p| Value::String(p.to_string())).collect()))
    });
    f!("join", |_, a| {
        let sep = s(&a, 1);
        match a.get(0) {
            Some(Value::List(v)) => Ok(Value::String(
                v.iter().map(|x| x.display()).collect::<Vec<_>>().join(&sep))),
            other => Ok(Value::String(other.map(|v| v.display()).unwrap_or_default())),
        }
    });
    f!("replace", |_, a| Ok(Value::String(
        s(&a, 0).replace(&*s(&a, 1), &s(&a, 2)))));
    f!("pad_start", |_, a| {
        let width = match a.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
        let pad = a.get(2).map(|v| v.display()).unwrap_or_else(|| " ".into());
        let pad = pad.chars().next().unwrap_or(' ');
        let st = s(&a, 0);
        let len = st.chars().count();
        if len >= width { Ok(Value::String(st)) }
        else { Ok(Value::String(pad.to_string().repeat(width - len) + &st)) }
    });
    f!("pad_end", |_, a| {
        let width = match a.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
        let pad = a.get(2).map(|v| v.display()).unwrap_or_else(|| " ".into());
        let pad = pad.chars().next().unwrap_or(' ');
        let st = s(&a, 0);
        let len = st.chars().count();
        if len >= width { Ok(Value::String(st)) }
        else { Ok(Value::String(format!("{}{}", st, pad.to_string().repeat(width - len)))) }
    });
    f!("starts_with", |_, a| Ok(Value::Bool(s(&a, 0).starts_with(&*s(&a, 1)))));
    f!("ends_with",   |_, a| Ok(Value::Bool(s(&a, 0).ends_with(&*s(&a, 1)))));
    f!("contains",    |_, a| Ok(Value::Bool(s(&a, 0).contains(&*s(&a, 1)))));
    f!("index_of", |_, a| {
        Ok(match s(&a, 0).find(&*s(&a, 1)) {
            Some(i) => Value::Int(i as i64),
            None    => Value::Int(-1),
        })
    });
    f!("chars", |_, a| Ok(Value::List(
        s(&a, 0).chars().map(Value::Char).collect())));
    f!("bytes", |_, a| Ok(Value::List(
        s(&a, 0).bytes().map(|b| Value::Int(b as i64)).collect())));
    f!("lines", |_, a| Ok(Value::List(
        s(&a, 0).lines().map(|l| Value::String(l.to_string())).collect())));
    f!("is_numeric", |_, a| {
        Ok(Value::Bool(!s(&a, 0).is_empty() && s(&a, 0).chars().all(|c| c.is_ascii_digit())))
    });
    f!("is_alpha", |_, a| {
        Ok(Value::Bool(!s(&a, 0).is_empty() && s(&a, 0).chars().all(|c| c.is_alphabetic())))
    });
    f!("is_alphanumeric", |_, a| {
        Ok(Value::Bool(!s(&a, 0).is_empty() && s(&a, 0).chars().all(|c| c.is_alphanumeric())))
    });
    // format("{0} + {1}", a, b) — also supports {} in order
    f!("format", |_, a| {
        let mut out = s(&a, 0);
        let mut positional = 0usize;
        let mut result = String::new();
        let mut chars = out.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let mut token = String::new();
                while let Some(&n) = chars.peek() {
                    chars.next();
                    if n == '}' { break; }
                    token.push(n);
                }
                let idx: usize = if token.is_empty() {
                    let i = positional; positional += 1; i
                } else {
                    token.parse().unwrap_or(usize::MAX)
                };
                if let Some(v) = a.get(idx + 1) {
                    result.push_str(&v.display());
                } else {
                    result.push('{'); result.push_str(&token); result.push('}');
                }
            } else {
                result.push(c);
            }
        }
        out = result;
        Ok(Value::String(out))
    });

    m
}
