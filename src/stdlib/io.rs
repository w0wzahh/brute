use std::collections::HashMap;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

/// Returns a map of name → Value for the io module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    m.insert("println".into(), Value::NativeFunction {
        name: "io::println".into(),
        func: |_interp, args| {
            let parts: Vec<String> = args.iter().map(|v| v.display()).collect();
            println!("{}", parts.join(" "));
            Ok(Value::None)
        },
    });

    m.insert("print".into(), Value::NativeFunction {
        name: "io::print".into(),
        func: |_interp, args| {
            let parts: Vec<String> = args.iter().map(|v| v.display()).collect();
            print!("{}", parts.join(" "));
            Ok(Value::None)
        },
    });

    m.insert("read_line".into(), Value::NativeFunction {
        name: "io::read_line".into(),
        func: |_interp, _args| {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)
                .map_err(|e| BruteError::IOException(e.to_string()))?;
            Ok(Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
        },
    });

    m.insert("input".into(), Value::NativeFunction {
        name: "io::input".into(),
        func: |_interp, args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt.display());
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)
                .map_err(|e| BruteError::IOException(e.to_string()))?;
            Ok(Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
        },
    });

    // io.open(path, mode) -> File object.
    // mode: "r" (read), "w" (write/truncate), "a" (append). Default "r".
    m.insert("open".into(), Value::NativeFunction {
        name: "io::open".into(),
        func: |_interp, args| {
            let path = args.get(0).map(|v| v.display())
                .ok_or_else(|| BruteError::ArgumentError("open() requires a path".into()))?;
            let mode = args.get(1).map(|v| v.display()).unwrap_or_else(|| "r".into());
            if !matches!(mode.as_str(), "r" | "w" | "a") {
                return Err(BruteError::ValueError(
                    format!("invalid open mode '{}'; expected r, w or a", mode)));
            }
            if mode == "w" {
                std::fs::File::create(&path)
                    .map_err(|e| BruteError::IOException(e.to_string()))?;
            } else if mode == "a" {
                std::fs::OpenOptions::new().create(true).append(true).open(&path)
                    .map_err(|e| BruteError::IOException(e.to_string()))?;
            } else if !std::path::Path::new(&path).exists() {
                return Err(BruteError::IOException(format!("file not found: {}", path)));
            }

            let mut fields = HashMap::new();
            fields.insert("path".into(), Value::String(path));
            fields.insert("mode".into(), Value::String(mode));
            fields.insert("read_text".into(), Value::NativeFunction {
                name: "File::read_text".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    std::fs::read_to_string(&path).map(Value::String)
                        .map_err(|e| BruteError::IOException(e.to_string()))
                },
            });
            fields.insert("write_text".into(), Value::NativeFunction {
                name: "File::write_text".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    let text = a.get(1).map(|v| v.display()).unwrap_or_default();
                    std::fs::write(&path, text)
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    Ok(Value::None)
                },
            });
            fields.insert("append_text".into(), Value::NativeFunction {
                name: "File::append_text".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    let text = a.get(1).map(|v| v.display()).unwrap_or_default();
                    let mut f = std::fs::OpenOptions::new().append(true).open(&path)
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    use std::io::Write;
                    f.write_all(text.as_bytes())
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    Ok(Value::None)
                },
            });
            fields.insert("read_lines".into(), Value::NativeFunction {
                name: "File::read_lines".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    let s = std::fs::read_to_string(&path)
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    Ok(Value::List(s.lines().map(|l| Value::String(l.into())).collect()))
                },
            });
            fields.insert("read_bytes".into(), Value::NativeFunction {
                name: "File::read_bytes".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    let b = std::fs::read(&path)
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    String::from_utf8(b).map(Value::String)
                        .map_err(|e| BruteError::ValueError(format!("not UTF-8: {}", e)))
                },
            });
            fields.insert("write_bytes".into(), Value::NativeFunction {
                name: "File::write_bytes".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    let text = a.get(1).map(|v| v.display()).unwrap_or_default();
                    std::fs::write(&path, text.as_bytes())
                        .map_err(|e| BruteError::IOException(e.to_string()))?;
                    Ok(Value::None)
                },
            });
            fields.insert("exists".into(), Value::NativeFunction {
                name: "File::exists".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    Ok(Value::Bool(std::path::Path::new(&path).exists()))
                },
            });
            fields.insert("close".into(), Value::NativeFunction {
                name: "File::close".into(),
                func: |_i, _a| Ok(Value::None),
            });
            fields.insert("to_string".into(), Value::NativeFunction {
                name: "File::to_string".into(),
                func: |_i, a| {
                    let path = obj_field(&a, "path")?;
                    Ok(Value::String(format!("File(\"{}\")", path)))
                },
            });
            Ok(Value::Object { type_name: "File".into(), fields })
        },
    });

    // `io.File` — same constructor as `open`, exposed as a namespace so both
    // `io.File("p", "w")` and `io.File::open("p", "w")` work.
    if let Some(open_fn) = m.get("open").cloned() {
        let mut file_ns = HashMap::new();
        file_ns.insert("new".into(), open_fn.clone());
        file_ns.insert("open".into(), open_fn);
        m.insert("File".into(), Value::Dict(file_ns));
    }

    // `io.Path` — `io.Path::new("dir/file.txt")` → path object.
    let mut path_ns = HashMap::new();
    path_ns.insert("new".into(), Value::NativeFunction {
        name: "Path::new".into(),
        func: |_i, a| Ok(path_obj(a.get(0).map(|v| v.display()).unwrap_or_default())),
    });
    m.insert("Path".into(), Value::Dict(path_ns));

    // ── filesystem helpers ───────────────────────────────────────────────
    m.insert("exists".into(), Value::NativeFunction {
        name: "io::exists".into(),
        func: |_i, a| Ok(Value::Bool(std::path::Path::new(
            &a.get(0).map(|v| v.display()).unwrap_or_default()).exists())),
    });
    m.insert("is_file".into(), Value::NativeFunction {
        name: "io::is_file".into(),
        func: |_i, a| Ok(Value::Bool(std::path::Path::new(
            &a.get(0).map(|v| v.display()).unwrap_or_default()).is_file())),
    });
    m.insert("is_dir".into(), Value::NativeFunction {
        name: "io::is_dir".into(),
        func: |_i, a| Ok(Value::Bool(std::path::Path::new(
            &a.get(0).map(|v| v.display()).unwrap_or_default()).is_dir())),
    });
    m.insert("create_dir".into(), Value::NativeFunction {
        name: "io::create_dir".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            std::fs::create_dir_all(&p).map(|_| Value::None)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });
    m.insert("remove_file".into(), Value::NativeFunction {
        name: "io::remove_file".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            std::fs::remove_file(&p).map(|_| Value::None)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });
    m.insert("remove_dir".into(), Value::NativeFunction {
        name: "io::remove_dir".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            std::fs::remove_dir_all(&p).map(|_| Value::None)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });
    m.insert("list_dir".into(), Value::NativeFunction {
        name: "io::list_dir".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_else(|| ".".into());
            let entries = std::fs::read_dir(&p)
                .map_err(|e| BruteError::IOException(e.to_string()))?;
            let names: Vec<Value> = entries.filter_map(|e| e.ok())
                .map(|e| Value::String(e.file_name().to_string_lossy().into_owned()))
                .collect();
            Ok(Value::List(names))
        },
    });
    m.insert("read_file".into(), Value::NativeFunction {
        name: "io::read_file".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            std::fs::read_to_string(&p).map(Value::String)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });
    m.insert("write_file".into(), Value::NativeFunction {
        name: "io::write_file".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            let text = a.get(1).map(|v| v.display()).unwrap_or_default();
            std::fs::write(&p, text).map(|_| Value::None)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });
    m.insert("append_file".into(), Value::NativeFunction {
        name: "io::append_file".into(),
        func: |_i, a| {
            let p = a.get(0).map(|v| v.display()).unwrap_or_default();
            let text = a.get(1).map(|v| v.display()).unwrap_or_default();
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p)
                .map_err(|e| BruteError::IOException(e.to_string()))?;
            use std::io::Write;
            f.write_all(text.as_bytes()).map(|_| Value::None)
                .map_err(|e| BruteError::IOException(e.to_string()))
        },
    });

    m
}

fn path_obj(path: String) -> Value {
    let mut fields = HashMap::new();
    fields.insert("path".into(), Value::String(path));
    fields.insert("join".into(), Value::NativeFunction {
        name: "Path::join".into(),
        func: |_i, a| {
            let base = obj_field(&a, "path")?;
            let seg = a.get(1).map(|v| v.display()).unwrap_or_default();
            Ok(path_obj(std::path::Path::new(&base).join(&seg)
                .to_string_lossy().into_owned()))
        },
    });
    fields.insert("exists".into(), Value::NativeFunction {
        name: "Path::exists".into(),
        func: |_i, a| {
            let p = obj_field(&a, "path")?;
            Ok(Value::Bool(std::path::Path::new(&p).exists()))
        },
    });
    fields.insert("parent".into(), Value::NativeFunction {
        name: "Path::parent".into(),
        func: |_i, a| {
            let p = obj_field(&a, "path")?;
            Ok(path_obj(std::path::Path::new(&p).parent()
                .map(|q| q.to_string_lossy().into_owned()).unwrap_or_default()))
        },
    });
    fields.insert("file_name".into(), Value::NativeFunction {
        name: "Path::file_name".into(),
        func: |_i, a| {
            let p = obj_field(&a, "path")?;
            Ok(Value::String(std::path::Path::new(&p).file_name()
                .map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()))
        },
    });
    fields.insert("to_string".into(), Value::NativeFunction {
        name: "Path::to_string".into(),
        func: |_i, a| obj_field(&a, "path").map(Value::String),
    });
    Value::Object { type_name: "Path".into(), fields }
}

fn obj_field(args: &[Value], name: &str) -> Result<String> {
    match args.get(0) {
        Some(Value::Object { fields, .. }) => {
            fields.get(name).map(|v| v.display())
                .ok_or_else(|| BruteError::RuntimeError(format!("object has no field '{}'", name)))
        }
        _ => Err(BruteError::RuntimeError("method called on non-object".into())),
    }
}
