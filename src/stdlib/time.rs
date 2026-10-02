use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

lazy_static::lazy_static! {
    static ref START: Instant = Instant::now();
}

fn epoch_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn datetime_fields(ms: i64) -> (i64, i64, i64, i64, i64, i64, i64) {
    let days = ms.div_euclid(86_400_000);
    let rem  = ms.rem_euclid(86_400_000);
    let (y, m, d) = civil_from_days(days);
    let hour   = rem / 3_600_000;
    let minute = rem % 3_600_000 / 60_000;
    let second = rem % 60_000 / 1_000;
    // 1970-01-01 was a Thursday (4)
    let dow = (days + 4).rem_euclid(7);
    (y, m, d, hour, minute, second, dow)
}

fn millis_of(args: &[Value]) -> Result<i64> {
    match args.get(0) {
        Some(Value::Object { fields, .. }) => match fields.get("millis") {
            Some(Value::Int(ms)) => Ok(*ms),
            _ => Err(BruteError::RuntimeError("DateTime missing millis".into())),
        },
        _ => Err(BruteError::RuntimeError("method called on non-DateTime".into())),
    }
}

fn datetime_obj(ms: i64) -> Value {
    let mut fields = HashMap::new();
    fields.insert("millis".into(), Value::Int(ms));

    macro_rules! dt_method {
        ($name:expr, $body:expr) => {
            fields.insert($name.into(), Value::NativeFunction {
                name: concat!("DateTime::", $name).into(),
                func: $body,
            });
        };
    }

    dt_method!("year",   |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).0)));
    dt_method!("month",  |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).1)));
    dt_method!("day",    |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).2)));
    dt_method!("hour",   |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).3)));
    dt_method!("minute", |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).4)));
    dt_method!("second", |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).5)));
    dt_method!("day_of_week", |_, a| Ok(Value::Int(datetime_fields(millis_of(&a)?).6)));
    dt_method!("timestamp",   |_, a| Ok(Value::Int(millis_of(&a)?)));

    dt_method!("format", |_, a| {
        let (y, mo, d, h, mi, s, _) = datetime_fields(millis_of(&a)?);
        Ok(Value::String(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)))
    });
    dt_method!("format_date", |_, a| {
        let (y, mo, d, ..) = datetime_fields(millis_of(&a)?);
        Ok(Value::String(format!("{:04}-{:02}-{:02}", y, mo, d)))
    });
    dt_method!("format_time", |_, a| {
        let (_, _, _, h, mi, s, _) = datetime_fields(millis_of(&a)?);
        Ok(Value::String(format!("{:02}:{:02}:{:02}", h, mi, s)))
    });
    dt_method!("to_string", |_, a| {
        let (y, mo, d, h, mi, s, _) = datetime_fields(millis_of(&a)?);
        Ok(Value::String(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)))
    });

    // `dt.add(n, "days"|"hours"|"minutes"|"seconds"|"millis"|"weeks")`
    dt_method!("add", |_, a| {
        let ms = millis_of(&a)?;
        let n = match a.get(1) {
            Some(Value::Int(n))   => *n,
            Some(Value::Float(n)) => *n as i64,
            _ => 0,
        };
        let unit = a.get(2).map(|v| v.display()).unwrap_or_else(|| "millis".into());
        let mult: i64 = match unit.as_str() {
            "ms" | "millis" | "milliseconds"       => 1,
            "s" | "sec" | "secs" | "seconds"       => 1_000,
            "m" | "min" | "mins" | "minutes"       => 60_000,
            "h" | "hr" | "hrs" | "hours"           => 3_600_000,
            "d" | "day" | "days"                   => 86_400_000,
            "w" | "week" | "weeks"                 => 604_800_000,
            other => return Err(BruteError::ValueError(
                format!("unknown time unit '{}'", other))),
        };
        Ok(datetime_obj(ms.saturating_add(n.saturating_mul(mult))))
    });
    dt_method!("sub", |_, a| {
        let ms = millis_of(&a)?;
        let n = match a.get(1) {
            Some(Value::Int(n))   => *n,
            Some(Value::Float(n)) => *n as i64,
            _ => 0,
        };
        Ok(datetime_obj(ms.saturating_sub(n)))
    });
    // `dt.diff(other)` — milliseconds between two DateTimes
    dt_method!("diff", |_, a| {
        let ms = millis_of(&a)?;
        let other = millis_of(&a[1..]).unwrap_or(0);
        Ok(Value::Int(ms - other))
    });
    dt_method!("is_before", |_, a| {
        let ms = millis_of(&a)?;
        Ok(Value::Bool(ms < millis_of(&a[1..]).unwrap_or(i64::MAX)))
    });
    dt_method!("is_after", |_, a| {
        let ms = millis_of(&a)?;
        Ok(Value::Bool(ms > millis_of(&a[1..]).unwrap_or(i64::MIN)))
    });

    Value::Object { type_name: "DateTime".into(), fields }
}

fn timer_obj() -> Value {
    let start = epoch_millis();
    let mut fields = HashMap::new();
    fields.insert("start_ms".into(), Value::Int(start));
    fields.insert("elapsed".into(), Value::NativeFunction {
        name: "Timer::elapsed".into(),
        func: |_, a| {
            match a.get(0) {
                Some(Value::Object { fields, .. }) => match fields.get("start_ms") {
                    Some(Value::Int(s)) => Ok(Value::Int(epoch_millis() - s)),
                    _ => Err(BruteError::RuntimeError("Timer missing start".into())),
                },
                _ => Err(BruteError::RuntimeError("method called on non-Timer".into())),
            }
        },
    });
    // `timer.start()` — (re)start the timer. Values are immutable, so this
    // returns a fresh timer; the idiom `timer.start()` simply re-marks now.
    fields.insert("start".into(), Value::NativeFunction {
        name: "Timer::start".into(),
        func: |_, _| Ok(timer_obj()),
    });
    fields.insert("reset".into(), Value::NativeFunction {
        name: "Timer::reset".into(),
        func: |_, _| Ok(timer_obj()),
    });
    // `timer.stop()` — stop and report elapsed ms (timers are monotonic here).
    fields.insert("stop".into(), Value::NativeFunction {
        name: "Timer::stop".into(),
        func: |_, a| {
            match a.get(0) {
                Some(Value::Object { fields, .. }) => match fields.get("start_ms") {
                    Some(Value::Int(s)) => Ok(Value::Int(epoch_millis() - s)),
                    _ => Err(BruteError::RuntimeError("Timer missing start".into())),
                },
                _ => Err(BruteError::RuntimeError("method called on non-Timer".into())),
            }
        },
    });
    fields.insert("to_string".into(), Value::NativeFunction {
        name: "Timer::to_string".into(),
        func: |_, a| {
            match a.get(0) {
                Some(Value::Object { fields, .. }) => match fields.get("start_ms") {
                    Some(Value::Int(s)) => Ok(Value::String(format!("{} ms elapsed", epoch_millis() - s))),
                    _ => Err(BruteError::RuntimeError("Timer missing start".into())),
                },
                _ => Err(BruteError::RuntimeError("method called on non-Timer".into())),
            }
        },
    });
    Value::Object { type_name: "Timer".into(), fields }
}

fn duration_obj(ms: i64) -> Value {
    let mut fields = HashMap::new();
    fields.insert("millis".into(), Value::Int(ms));
    fields.insert("to_millis".into(), Value::NativeFunction {
        name: "Duration::to_millis".into(),
        func: |_, a| millis_of(&a).map(Value::Int),
    });
    fields.insert("to_secs".into(), Value::NativeFunction {
        name: "Duration::to_secs".into(),
        func: |_, a| millis_of(&a).map(|ms| Value::Int(ms / 1000)),
    });
    fields.insert("to_string".into(), Value::NativeFunction {
        name: "Duration::to_string".into(),
        func: |_, a| millis_of(&a).map(|ms| Value::String(format!("{}ms", ms))),
    });
    Value::Object { type_name: "Duration".into(), fields }
}

/// Returns a map of name → Value for the time module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("time::", $name).into(),
                func: $body,
            });
        };
    }

    f!("now", |_, _| Ok(Value::Int(epoch_millis())));
    f!("now_secs", |_, _| Ok(Value::Float(
        SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64()).unwrap_or(0.0)
    )));
    f!("monotonic", |_, _| Ok(Value::Int(START.elapsed().as_millis() as i64)));
    f!("sleep", |_, a| {
        let ms = match a.get(0) {
            Some(Value::Int(ms))   => *ms,
            Some(Value::Float(ms)) => *ms as i64,
            _ => 0,
        };
        std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
        Ok(Value::None)
    });
    f!("DateTime", |_, a| {
        match a.get(0) {
            Some(Value::Int(ms)) => Ok(datetime_obj(*ms)),
            _ => Ok(datetime_obj(epoch_millis())),
        }
    });
    f!("from_millis", |_, a| {
        match a.get(0) {
            Some(Value::Int(ms)) => Ok(datetime_obj(*ms)),
            _ => Ok(datetime_obj(epoch_millis())),
        }
    });
    // `Timer` is a namespace dict — `time.Timer()` and `time.Timer::create()`
    // both construct a timer (dicts are callable via `new`/`create`).
    let mut timer_ns = HashMap::new();
    timer_ns.insert("create".into(), Value::NativeFunction {
        name: "Timer::create".into(),
        func: |_, _| Ok(timer_obj()),
    });
    timer_ns.insert("new".into(), Value::NativeFunction {
        name: "Timer::new".into(),
        func: |_, _| Ok(timer_obj()),
    });
    m.insert("Timer".into(), Value::Dict(timer_ns));

    // `Duration` — `Duration::seconds(n)` / `millis(n)` / `minutes(n)` …
    fn dur_arg(a: &[Value]) -> i64 {
        match a.get(0) {
            Some(Value::Int(n))   => *n,
            Some(Value::Float(n)) => *n as i64,
            _ => 0,
        }
    }
    let mut dur_ns = HashMap::new();
    dur_ns.insert("millis".into(), Value::NativeFunction {
        name: "Duration::millis".into(), func: |_, a| Ok(duration_obj(dur_arg(&a))) });
    dur_ns.insert("from_millis".into(), Value::NativeFunction {
        name: "Duration::from_millis".into(), func: |_, a| Ok(duration_obj(dur_arg(&a))) });
    dur_ns.insert("seconds".into(), Value::NativeFunction {
        name: "Duration::seconds".into(), func: |_, a| Ok(duration_obj(dur_arg(&a).saturating_mul(1_000))) });
    dur_ns.insert("minutes".into(), Value::NativeFunction {
        name: "Duration::minutes".into(), func: |_, a| Ok(duration_obj(dur_arg(&a).saturating_mul(60_000))) });
    dur_ns.insert("hours".into(), Value::NativeFunction {
        name: "Duration::hours".into(), func: |_, a| Ok(duration_obj(dur_arg(&a).saturating_mul(3_600_000))) });
    m.insert("Duration".into(), Value::Dict(dur_ns));

    f!("format_ms", |_, a| {
        let ms = match a.get(0) { Some(Value::Int(i)) => *i, _ => 0 };
        if ms < 1000 { Ok(Value::String(format!("{}ms", ms))) }
        else if ms < 60_000 { Ok(Value::String(format!("{:.2}s", ms as f64 / 1000.0))) }
        else { Ok(Value::String(format!("{:.2}m", ms as f64 / 60_000.0))) }
    });

    // `format_current_time(fmt?)` — strftime-style %Y %m %d %H %M %S
    f!("format_current_time", |_, a| {
        let (y, mo, d, h, mi, s, _) = datetime_fields(epoch_millis());
        let fmt_str = a.get(0).map(|v| v.display())
            .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".into());
        let out = fmt_str
            .replace("%Y", &format!("{:04}", y))
            .replace("%m", &format!("{:02}", mo))
            .replace("%d", &format!("{:02}", d))
            .replace("%H", &format!("{:02}", h))
            .replace("%M", &format!("{:02}", mi))
            .replace("%S", &format!("{:02}", s));
        Ok(Value::String(out))
    });

    // `measure(f, args...)` — run `f`, return `{ time: ms, result: value }`
    f!("measure", |i, a| {
        let f = a.get(0).cloned()
            .ok_or_else(|| BruteError::ArgumentError("measure() needs a function".into()))?;
        let call_args: Vec<Value> = a.iter().skip(1).cloned().collect();
        let t0 = epoch_millis();
        let result = i.call_value(f, call_args)?;
        let mut m: HashMap<String, Value> = HashMap::new();
        m.insert("time".into(),   Value::Int(epoch_millis() - t0));
        m.insert("result".into(), result);
        Ok(Value::Dict(m))
    });

    m
}
