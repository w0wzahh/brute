import type { Metadata } from "next";
import DocsShell from "@/components/DocsShell";
import HiCode from "@/components/HiCode";


export const metadata: Metadata = { title: 'Brute - Language Reference' };

const TOC = [
  {
    "id": "syntax",
    "title": "Syntax Overview"
  },
  {
    "id": "types",
    "title": "Type System"
  },
  {
    "id": "expressions",
    "title": "Expressions"
  },
  {
    "id": "statements",
    "title": "Statements"
  },
  {
    "id": "functions",
    "title": "Functions"
  },
  {
    "id": "structs",
    "title": "Structs"
  },
  {
    "id": "modules",
    "title": "Modules"
  },
  {
    "id": "error-handling",
    "title": "Error Handling"
  }
];

export default function Page() {
  return (
    <DocsShell toc={TOC}>
      <h1 className="doc-title">Brute Language Reference</h1>
      <p className="doc-sub">Comprehensive documentation of Brute's syntax, types, functions, and standard library.</p>
      <section className="window" id="syntax">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Syntax Overview</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Syntax Overview</h2>
                <p>Brute's syntax is designed to be familiar to developers coming from Rust, Python, or C/C++. It emphasizes readability while maintaining expressiveness.</p>
                
                <h3>Comments</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// This is a single-line comment\n\n/*\n  This is a\n  multi-line comment\n*/\n\n/// Documentation comment for functions, structs, etc.\n/// Supports markdown formatting"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="types">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Type System</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Type System</h2>
                <p>Brute features a static type system with type inference. The following are the primary built-in types:</p>
                
                <h3>Primitive Types</h3>
                <ul>
                  <li><code>{"int"}</code> - Signed integer (default is platform-dependent, typically 32 or 64-bit)</li>
                  <li><code>{"float"}</code> - Floating-point number (default is 64-bit)</li>
                  <li><code>{"bool"}</code> - Boolean type with values <code>{"true"}</code> and <code>{"false"}</code></li>
                  <li><code>{"char"}</code> - Single Unicode character</li>
                  <li><code>{"string"}</code> - UTF-8 encoded string</li>
                </ul>
                
                <h3>Type Declarations</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Type inference\nlet x = 5;               // x is inferred as int\nlet name = \"Alice\";      // name is inferred as string\n\n// Explicit type annotations\nlet y: float = 3.14;\nlet active: bool = true;\n\n// Type aliases\ntype UserId = int;\nlet user_id: UserId = 1001;"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="expressions">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Expressions</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Expressions</h2>
                <p>An expression in Brute evaluates to a value. Most constructs in Brute are expressions.</p>
                
                <h3>Literals</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Integer literals\n42      // Decimal\n0x2A    // Hexadecimal\n0b101010 // Binary\n0o52    // Octal\n\n// Float literals\n3.14\n2.71e2  // Scientific notation\n\n// Boolean literals\ntrue\nfalse\n\n// Character literal\n'A'\n\n// String literals\n\"Hello, World!\"\n\"Line 1\\nLine 2\"  // With escape sequences\n\n// Multiline strings\n\"\"\"\nThis is a multiline\nstring in Brute\n\"\"\""} />
                </div>
                
                <h3>Operators</h3>
                <h4>Arithmetic Operators</h4>
                <ul>
                  <li><code>{"+"}</code> - Addition</li>
                  <li><code>{"-"}</code> - Subtraction</li>
                  <li><code>{"*"}</code> - Multiplication</li>
                  <li><code>{"/"}</code> - Division</li>
                  <li><code>{"%"}</code> - Modulo (remainder)</li>
                  <li><code>{"**"}</code> - Exponentiation</li>
                </ul>
                
                <h4>Comparison Operators</h4>
                <ul>
                  <li><code>{"=="}</code> - Equal to</li>
                  <li><code>{"!="}</code> - Not equal to</li>
                  <li><code>{"<"}</code> - Less than</li>
                  <li><code>{">"}</code> - Greater than</li>
                  <li><code>{"<="}</code> - Less than or equal to</li>
                  <li><code>{">="}</code> - Greater than or equal to</li>
                </ul>
                
                <h4>Logical Operators</h4>
                <ul>
                  <li><code>{"&&"}</code> - Logical AND</li>
                  <li><code>{"||"}</code> - Logical OR</li>
                  <li><code>{"!"}</code> - Logical NOT</li>
                </ul>
              </div>
              </section>
              
              <section className="window" id="statements">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Statements</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Statements</h2>
                <p>Statements are instructions that perform some action but don't return a value.</p>
                
                <h3>Variable Declarations</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Immutable variable declaration\nlet x = 5;\n\n// Mutable variable declaration\nlet mut y = 10;\n\n// Constants\nconst MAX_ITEMS = 100;"} />
                </div>
                
                <h3>Control Flow</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// If statement\nif condition {\n    // Code executed if condition is true\n} else if another_condition {\n    // Code executed if another_condition is true\n} else {\n    // Code executed if no conditions are true\n}\n\n// Match statement\nmatch value {\n    pattern1 => expression1,\n    pattern2 => expression2,\n    _ => default_expression,\n}\n\n// While loop\nwhile condition {\n    // Loop body\n}\n\n// For loop\nfor item in collection {\n    // Loop body\n}\n\n// Range-based for loop\nfor i in 0..10 {\n    // Loop body, i takes values 0 through 9\n}"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="functions">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Functions</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Functions</h2>
                <p>Functions in Brute are defined using the <code>{"fn"}</code> keyword.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Basic function\nfn add(a: int, b: int) -> int {\n    return a + b;\n}\n\n// Function with no return value (void)\nfn print_hello() -> void {\n    println(\"Hello!\");\n}\n\n// Function with default parameters\nfn greet(name: string, greeting: string = \"Hello\") -> string {\n    return greeting + \", \" + name + \"!\";\n}\n\n// Function with variadic parameters\nfn sum(...numbers: int) -> int {\n    let total = 0;\n    for n in numbers {\n        total = total + n;\n    }\n    return total;\n}\n\n// Function with early return\nfn is_even(num: int) -> bool {\n    if num % 2 == 0 {\n        return true;\n    }\n    return false;\n}"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="structs">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Structs</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Structs</h2>
                <p>Structs are used to create custom data types that group related values together.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Struct definition\nstruct Point {\n    x: float,\n    y: float,\n    \n    // Method\n    fn distance_from_origin() -> float {\n        return (self.x * self.x + self.y * self.y).sqrt();\n    }\n    \n    // Static method (constructor)\n    fn new(x: float, y: float) -> Point {\n        return Point { x: x, y: y };\n    }\n}\n\n// Creating an instance\nlet p1 = Point { x: 3.0, y: 4.0 };\nlet p2 = Point::new(5.0, 12.0);\n\n// Accessing fields\nlet x_coord = p1.x;\n\n// Calling methods\nlet distance = p1.distance_from_origin();"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="modules">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Modules</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Modules</h2>
                <p>Modules are used to organize code into logical units and control visibility (public vs. private).</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Defining a module\nmodule math {\n    // Public function (can be accessed outside the module)\n    pub fn add(a: int, b: int) -> int {\n        return a + b;\n    }\n    \n    // Private function (only accessible within the module)\n    fn subtract(a: int, b: int) -> int {\n        return a - b;\n    }\n    \n    // Nested module\n    module advanced {\n        pub fn power(base: int, exponent: int) -> int {\n            return base ** exponent;\n        }\n    }\n}\n\n// Importing modules\nimport math;\nimport math.advanced;\n\n// Using imported modules\nlet sum = math.add(5, 3);\nlet pow = math.advanced.power(2, 8);"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="error-handling">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Error Handling</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Error Handling</h2>
                <p>Brute provides multiple mechanisms for error handling: try/catch blocks and Result types.</p>
                
                <h3>Try/Catch</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"try {\n    // Code that might throw an error\n    let result = risky_operation();\n} catch err {\n    // Code that handles the error\n    println(\"Error: \" + err.to_string());\n} finally {\n    // Code that always runs, whether there was an error or not\n    cleanup_resources();\n}"} />
                </div>
                
                <h3>Result Type</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Function returning a Result type\nfn divide(a: int, b: int) -> Result<int, string> {\n    if b == 0 {\n        return Err(\"Division by zero\");\n    }\n    return Ok(a / b);\n}\n\n// Using the Result\nlet result = divide(10, 2);\nif result.is_ok() {\n    println(\"Result: \" + result.unwrap().to_string());\n} else {\n    println(\"Error: \" + result.unwrap_err());\n}\n\n// Using the ? operator for error propagation\nfn calculate() -> Result<int, string> {\n    let a = divide(10, 2)?;  // Returns early if error\n    let b = divide(20, a)?;  // Returns early if error\n    return Ok(b);\n}"} />
                </div>
              </div>
              </section>
    </DocsShell>
  );
}
