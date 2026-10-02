<div align="center">
  <img src="assets/icon.svg" alt="Brute logo" width="140" />

  <h1>Brute</h1>

  <p><strong>Rust's safety. Python's readability. C's speed. Pick all three.</strong></p>

  <p>
    <a href="https://github.com/w0wzahh/brute/releases"><img src="https://img.shields.io/badge/version-0.5.0-facc00?style=for-the-badge" alt="Version" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/github/license/w0wzahh/brute?style=for-the-badge&color=4fd1b5" alt="License" /></a>
    <a href="https://github.com/w0wzahh/brute/stargazers"><img src="https://img.shields.io/github/stars/w0wzahh/brute?style=for-the-badge&color=ff6b9d" alt="Stars" /></a>
    <a href="https://github.com/w0wzahh/brute/issues"><img src="https://img.shields.io/github/issues/w0wzahh/brute?style=for-the-badge&color=a78bfa" alt="Issues" /></a>
    <img src="https://img.shields.io/badge/written%20in-Rust-e8855a?style=for-the-badge&logo=rust&logoColor=white" alt="Written in Rust" />
  </p>

  <p>
    <a href="#quick-start">Quick Start</a> ·
    <a href="#features">Features</a> ·
    <a href="#cli">CLI</a> ·
    <a href="#language-tour">Language Tour</a> ·
    <a href="#documentation">Docs</a>
  </p>
</div>

---

Brute is a compiled, statically-typed programming language written in Rust. It
pairs an ownership-style memory model with a clean, forgiving syntax — no
garbage collector, no runtime bloat, and error messages that actually help.
Programs can be **compiled to native executables** via the LLVM backend or
**interpreted** for fast iteration.

```rust
fn main() -> void {
    let name = "world";

    match name {
        "world" => println("hello, ${name}!"),
        _       => println("hi anyway"),
    }

    [1, 2, 3, 4, 5]
        |> map(*2)
        |> filter(>4)
        |> println();  // [6, 8, 10]
}
```

## Quick Start

```bash
# Clone and build the compiler
git clone https://github.com/w0wzahh/brute.git
cd brute

# Windows
build.bat
# Linux / macOS
./build.sh

# Or install into cargo's bin dir
cargo install --path .
```

```bash
# Run your first program
brute run examples/hello_world.brt
```

Prebuilt binaries are attached to the
[GitHub releases](https://github.com/w0wzahh/brute/releases) — grab the one for
your platform, put it on your `PATH`, and go.

## Features

- **Memory safety without a GC** — ownership-inspired model, no runtime pauses
- **Static typing + inference** — `let x = 5` just works; annotations when you want them
- **Real generics** — `struct Point<T>`, `Result<T, E>`, monomorphized at compile time
- **Traits & impl blocks** — polymorphism without class boilerplate
- **Pattern matching** — `match` with ranges, literals, guards, and `_` catch-alls
- **`async`/`await`** — built-in runtime, futures, `sleep`, `timeout`
- **Concurrency primitives** — threads, `Mutex`, `RwLock`, `CondVar`, `Arc`, channels
- **Pipeline operator** — `value |> f |> g` instead of nesting pyramids
- **Batteries-included stdlib** — io, collections, string, math, fs, net, time, crypto
- **Crypto built in** — SHA-256/512, HMAC, AES-256-GCM, secure RNG
- **LLVM backend** — optimizing native codegen (`--emit-llvm` for IR)
- **Dual mode** — compile for production, interpret for prototyping
- **Real tooling** — formatter (`brute format`), type checker (`brute check`)
- **Readable errors** — colored, source-located diagnostics
- **Strict mode** — `--strict-types` for the pedantic among us
- **Cross-platform** — Windows, Linux, macOS
- **Zero dependencies at runtime** — one binary per program
- **MIT licensed** — take it, break it, ship it

## CLI

| Command | Description |
|---|---|
| `brute run file.brt` | Interpret and run a program |
| `brute run-async file.brt` | Run with the async runtime |
| `brute compile file.brt [-o out]` | Compile to a native binary |
| `brute check file.brt` | Type-check without running |
| `brute format file.brt [-i]` | Pretty-print (or rewrite in-place) |
| `brute info` | Compiler/version info |

Global flags: `-v` verbose · `-o <none|less|default|aggressive>` optimization ·
`-d` debug info · `-t` timings · `--emit-llvm` write `.ll` IR · `--strict-types`
· `--no-color`.

## Language Tour

### Structs & generics

```rust
struct Point<T> {
    x: T,
    y: T,

    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }

    pub fn distance_from_origin(this) -> float {
        return ((this.x * this.x + this.y * this.y) as float).sqrt();
    }
}

let p = Point::new(3.0, 4.0);
println("Distance: " + p.distance_from_origin().to_string());  // 5
```

### Error handling with `Result` + `match`

```rust
fn divide(a: int, b: int) -> Result<float, string> {
    if b == 0 {
        return Result::Err("division by zero");
    }
    return Result::Ok(a as float / b as float);
}

match divide(10, 2) {
    Result::Ok(v)  => println("Result: " + v.to_string()),
    Result::Err(e) => println("Error: " + e),
}
```

### Async / await

```rust
async fn fetch_data(url: string) -> string {
    await sleep(1000);
    return "data from " + url;
}

fn main() -> void {
    let runtime = AsyncRuntime::new();
    println(runtime.block_on(fetch_data("https://example.com")));
}
```

### Threads & shared state

```rust
let counter = Arc::new(Mutex::new(0));
let c2 = counter.clone();

let handle = spawn(fn() -> void {
    for i in 0..5 {
        let mut n = c2.lock();
        *n += 1;
    }
});

handle.join();
println("final: " + counter.lock().to_string());
```

### Pipelines

```rust
let result = "  hello world  "
    |> str_trim
    |> str_uppercase
    |> (s => s + "!")
    |> (s => s.len());
```

Full syntax walkthroughs live on the
[website](https://github.com/w0wzahh/brute/tree/main/website) — the
`website/` folder is a self-contained Next.js build.

## Examples

Eleven ready-to-run programs in [`examples/`](examples/):

| File | Shows |
|---|---|
| `hello_world.brt` | Basics — functions, printing, interpolation |
| `features.brt` | Core language tour |
| `advanced_features.brt` | Generics, traits, enums |
| `traits_and_generics.brt` | Trait bounds and dispatch |
| `collections_example.brt` | `HashMap`, arrays, iteration |
| `async_programming.brt` | `async`/`await` + the runtime |
| `pipeline_operator.brt` | `|>` chains |
| `io_operations.brt` | stdin/stdout, file I/O |
| `time_module.brt` | Timers, timestamps |
| `advanced.brt` | Everything combined |
| `hello.brt` | Minimal hello |

## Standard Library

`io` · `collections` · `string` · `math` · `fs` · `net` · `time` ·
`async_runtime` · `crypto` · `concurrent`

## Project Structure

```
brute/
├── src/            # compiler & interpreter (Rust)
│   └── stdlib/     # standard library modules
├── examples/       # sample .brt programs
├── docs/           # language docs (markdown)
├── website/        # official site — Next.js + TypeScript
├── assets/         # logo & art
├── build.bat       # Windows build
└── build.sh        # Linux/macOS build
```

## Documentation

- [`docs/getting_started.md`](docs/getting_started.md) — install + first program
- [`docs/language_guide.md`](docs/language_guide.md) — concepts, in order
- [`docs/language_reference.md`](docs/language_reference.md) — the full spec
- [`docs/language_grammar.md`](docs/language_grammar.md) — grammar reference

## Contributing

Issues and PRs are welcome — it's a solo-built project that's actively growing.
Check [`docs/`](docs/) to get oriented, and keep the neobrutalism loud.

## License

[MIT](LICENSE) — do whatever you want with it.

---

<div align="center">
  <sub>Built by <a href="https://github.com/w0wzahh">w0wzahh</a></sub>
</div>
