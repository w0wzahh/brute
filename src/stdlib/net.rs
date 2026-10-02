use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

lazy_static::lazy_static! {
    static ref STREAMS:   Mutex<HashMap<i64, TcpStream>>   = Mutex::new(HashMap::new());
    static ref LISTENERS: Mutex<HashMap<i64, TcpListener>> = Mutex::new(HashMap::new());
}
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 { NEXT_ID.fetch_add(1, Ordering::Relaxed) }

fn text(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.display()).unwrap_or_default()
}

fn handle(args: &[Value], i: usize) -> Result<i64> {
    match args.get(i) {
        Some(Value::Int(h)) => Ok(*h),
        other => Err(BruteError::TypeError(format!(
            "expected a handle (int), got {}",
            other.map(|v| v.type_name()).unwrap_or("none")))),
    }
}

/// Returns a map of name → Value for the net module.
/// Networking is intentionally small: DNS, TCP streams/listeners and a
/// minimal HTTP/1.1 GET (plaintext — no TLS).
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("net::", $name).into(),
                func: $body,
            });
        };
    }

    f!("resolve", |_, a| {
        let host = text(&a, 0);
        let port = match a.get(1) { Some(Value::Int(p)) => *p as u16, _ => 80 };
        let addrs = (host.as_str(), port).to_socket_addrs()
            .map_err(|e| BruteError::IOException(format!("resolve '{}': {}", host, e)))?;
        Ok(Value::List(addrs.map(|s| Value::String(s.ip().to_string())).collect()))
    });

    f!("tcp_connect", |_, a| {
        let host = text(&a, 0);
        let port = match a.get(1) { Some(Value::Int(p)) => *p as u16, _ => 80 };
        let stream = TcpStream::connect((host.as_str(), port))
            .map_err(|e| BruteError::IOException(format!("connect {}:{}: {}", host, port, e)))?;
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(30))).ok();
        let id = next_id();
        STREAMS.lock().unwrap().insert(id, stream);
        Ok(Value::Int(id))
    });

    f!("tcp_send", |_, a| {
        let h = handle(&a, 0)?;
        let data = text(&a, 1);
        let mut map = STREAMS.lock().unwrap();
        let stream = map.get_mut(&h)
            .ok_or_else(|| BruteError::RuntimeError(format!("bad stream handle {}", h)))?;
        stream.write_all(data.as_bytes())
            .and_then(|_| stream.flush())
            .map_err(|e| BruteError::IOException(e.to_string()))?;
        Ok(Value::Int(data.len() as i64))
    });

    f!("tcp_recv", |_, a| {
        let h = handle(&a, 0)?;
        let cap = match a.get(1) { Some(Value::Int(n)) => (*n).max(1) as usize, _ => 65536 };
        let mut map = STREAMS.lock().unwrap();
        let stream = map.get_mut(&h)
            .ok_or_else(|| BruteError::RuntimeError(format!("bad stream handle {}", h)))?;
        let mut buf = vec![0u8; cap];
        let n = stream.read(&mut buf)
            .map_err(|e| BruteError::IOException(e.to_string()))?;
        Ok(Value::String(String::from_utf8_lossy(&buf[..n]).into_owned()))
    });

    f!("tcp_close", |_, a| {
        let h = handle(&a, 0)?;
        STREAMS.lock().unwrap().remove(&h);
        Ok(Value::None)
    });

    f!("tcp_listen", |_, a| {
        let port = match a.get(0) { Some(Value::Int(p)) => *p as u16, _ => 0 };
        let listener = TcpListener::bind(("0.0.0.0", port))
            .map_err(|e| BruteError::IOException(format!("listen :{}: {}", port, e)))?;
        let id = next_id();
        LISTENERS.lock().unwrap().insert(id, listener);
        Ok(Value::Int(id))
    });

    f!("tcp_accept", |_, a| {
        let h = handle(&a, 0)?;
        let listener = LISTENERS.lock().unwrap().remove(&h)
            .ok_or_else(|| BruteError::RuntimeError(format!("bad listener handle {}", h)))?;
        let (stream, _addr) = listener.accept()
            .map_err(|e| BruteError::IOException(e.to_string()))?;
        LISTENERS.lock().unwrap().insert(h, listener);
        let id = next_id();
        STREAMS.lock().unwrap().insert(id, stream);
        Ok(Value::Int(id))
    });

    // Minimal HTTP/1.1 GET over plaintext TCP (no TLS).
    f!("http_get", |_, a| {
        let url = text(&a, 0);
        let stripped = url.strip_prefix("http://")
            .ok_or_else(|| BruteError::ValueError(
                "only http:// URLs are supported (no TLS)".into()))?;
        let (host_port, path) = match stripped.find('/') {
            Some(i) => (&stripped[..i], &stripped[i..]),
            None    => (stripped, "/"),
        };
        let (host, port) = match host_port.find(':') {
            Some(i) => (&host_port[..i], host_port[i + 1..].parse::<u16>().unwrap_or(80)),
            None    => (host_port, 80),
        };
        let mut stream = TcpStream::connect((host, port))
            .map_err(|e| BruteError::IOException(format!("connect {}:{}: {}", host, port, e)))?;
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let req = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: brute/0.5\r\nConnection: close\r\n\r\n",
            path, host_port);
        stream.write_all(req.as_bytes())
            .map_err(|e| BruteError::IOException(e.to_string()))?;
        let mut resp = Vec::new();
        stream.read_to_end(&mut resp)
            .map_err(|e| BruteError::IOException(e.to_string()))?;
        Ok(Value::String(String::from_utf8_lossy(&resp).into_owned()))
    });

    m
}
