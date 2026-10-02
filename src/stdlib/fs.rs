use std::collections::HashMap;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

fn p(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.display()).unwrap_or_default()
}

fn io_result<T>(r: std::io::Result<T>, map: impl Fn(T) -> Value) -> Result<Value> {
    r.map(map).map_err(|e| BruteError::IOException(e.to_string()))
}

/// Returns a map of name → Value for the fs module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("fs::", $name).into(),
                func: $body,
            });
        };
    }

    f!("read_file",  |_, a| io_result(std::fs::read_to_string(p(&a, 0)), Value::String));
    f!("read_bytes", |_, a| io_result(std::fs::read(p(&a, 0)),
        |b| Value::List(b.into_iter().map(|x| Value::Int(x as i64)).collect())));
    f!("write_file", |_, a| {
        io_result(std::fs::write(p(&a, 0), p(&a, 1)), |_| Value::None)
    });
    f!("append_file", |_, a| {
        use std::io::Write;
        io_result(std::fs::OpenOptions::new().create(true).append(true).open(p(&a, 0))
            .and_then(|mut file| file.write_all(p(&a, 1).as_bytes())), |_| Value::None)
    });
    f!("exists",     |_, a| Ok(Value::Bool(std::path::Path::new(&p(&a, 0)).exists())));
    f!("is_file",    |_, a| Ok(Value::Bool(std::path::Path::new(&p(&a, 0)).is_file())));
    f!("is_dir",     |_, a| Ok(Value::Bool(std::path::Path::new(&p(&a, 0)).is_dir())));
    f!("remove",     |_, a| {
        let path = p(&a, 0);
        let pth = std::path::Path::new(&path);
        let r = if pth.is_dir() { std::fs::remove_dir_all(pth) } else { std::fs::remove_file(pth) };
        io_result(r, |_| Value::None)
    });
    f!("mkdir",      |_, a| io_result(std::fs::create_dir(p(&a, 0)), |_| Value::None));
    f!("mkdir_all",  |_, a| io_result(std::fs::create_dir_all(p(&a, 0)), |_| Value::None));
    f!("copy",       |_, a| io_result(std::fs::copy(p(&a, 0), p(&a, 1)),
        |n| Value::Int(n as i64)));
    f!("rename",     |_, a| io_result(std::fs::rename(p(&a, 0), p(&a, 1)), |_| Value::None));
    f!("file_size",  |_, a| io_result(std::fs::metadata(p(&a, 0)).map(|d| d.len()),
        |n| Value::Int(n as i64)));
    f!("list_dir", |_, a| {
        io_result(std::fs::read_dir(p(&a, 0)), |rd| {
            Value::List(rd.filter_map(|e| e.ok())
                .map(|e| Value::String(e.path().to_string_lossy().into_owned()))
                .collect())
        })
    });
    f!("cwd", |_, _| io_result(std::env::current_dir(),
        |d| Value::String(d.to_string_lossy().into_owned())));
    f!("file_name", |_, a| Ok(Value::String(
        std::path::Path::new(&p(&a, 0)).file_name()
            .map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())));
    f!("extension", |_, a| Ok(Value::String(
        std::path::Path::new(&p(&a, 0)).extension()
            .map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())));
    f!("join_path", |_, a| {
        let mut pb = std::path::PathBuf::from(p(&a, 0));
        if let Some(rest) = a.get(1) { pb.push(rest.display()); }
        Ok(Value::String(pb.to_string_lossy().into_owned()))
    });

    m
}
