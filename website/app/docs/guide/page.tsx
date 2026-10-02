import type { Metadata } from "next";
import DocsShell from "@/components/DocsShell";
import HiCode from "@/components/HiCode";


export const metadata: Metadata = { title: 'Brute - Getting Started Guide' };

const TOC = [
  {
    "id": "installation",
    "title": "Installation"
  },
  {
    "id": "hello-world",
    "title": "Hello World"
  },
  {
    "id": "variables",
    "title": "Variables and Types"
  },
  {
    "id": "control-flow",
    "title": "Control Flow"
  },
  {
    "id": "functions",
    "title": "Functions"
  },
  {
    "id": "collections",
    "title": "Collections"
  },
  {
    "id": "structs",
    "title": "Structs and Methods"
  }
];

export default function Page() {
  return (
    <DocsShell toc={TOC}>
      <h1 className="doc-title">Getting Started with Brute</h1>
      <p className="doc-sub">Welcome to Brute! This guide will help you get started with the Brute programming language.</p>
      <section className="window" id="installation">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Installation</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Installation</h2>
                <p>Currently, Brute is in development, but you can build it from source:</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">bash</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="bash" code={"# Clone the repository\ngit clone https://github.com/w0wzahh/brute.git\ncd brute\n\n# Build the compiler\ncargo build --release\n\n# Add Brute to your PATH\n# For Linux/Mac\nexport PATH=\"$PATH:$(pwd)/target/release\"\n# For Windows\n# Add the path to your PATH environment variable"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="hello-world">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Hello World</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Hello World</h2>
                <p>Let's start with a simple "Hello, World!" program. Create a file named <code>{"hello.brt"}</code> with the following content:</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"fn main() -> void {\n    println(\"Hello, World!\");\n}"} />
                </div>
                
                <p>To run this program:</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">bash</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="bash" code={"brute run hello.brt"} />
                </div>
                
                <p>Congratulations! You've just written and run your first Brute program.</p>
              </div>
              </section>
              
              <section className="window" id="variables">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Variables and Types</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Variables and Types</h2>
                <p>Brute supports variable declarations with type inference:</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Variable declaration with type inference\nlet name = \"World\";\n\n// Explicit type annotation\nlet age: int = 30;\n\n// Mutable variables\nlet mut counter = 0;\ncounter += 1; // This is allowed because counter is mutable"} />
                </div>
                
                <p>Brute has several built-in types:</p>
                
                <ul>
                  <li><code>{"int"}</code>: Integer numbers</li>
                  <li><code>{"float"}</code>: Floating point numbers</li>
                  <li><code>{"bool"}</code>: Boolean values</li>
                  <li><code>{"string"}</code>: Text strings</li>
                  <li><code>{"char"}</code>: Single characters</li>
                </ul>
              </div>
              </section>
              
              <section className="window" id="control-flow">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Control Flow</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Control Flow</h2>
                
                <h3>If Statements</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"let x = 42;\n\nif x > 100 {\n    println(\"x is greater than 100\");\n} else if x > 50 {\n    println(\"x is greater than 50 but not greater than 100\");\n} else {\n    println(\"x is 50 or less\");\n}"} />
                </div>
                
                <h3>Loops</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// While loop\nlet mut i = 0;\nwhile i < 5 {\n    println(\"i is \" + i.to_string());\n    i += 1;\n}\n\n// For loop\nfor j in range(0, 5) {\n    println(\"j is \" + j.to_string());\n}\n\n// Iterating over collections\nlet fruits = [\"apple\", \"banana\", \"cherry\"];\nfor fruit in fruits {\n    println(\"I like \" + fruit);\n}"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="functions">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Functions</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Functions</h2>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Simple function\nfn greet(name: string) -> void {\n    println(\"Hello, \" + name + \"!\");\n}\n\n// Function with return value\nfn add(a: int, b: int) -> int {\n    return a + b;\n}\n\n// Function with default parameter\nfn greet_with_default(name: string = \"World\") -> void {\n    println(\"Hello, \" + name + \"!\");\n}\n\n// Using the functions\ngreet(\"Alice\");\nlet sum = add(5, 7);\nprintln(\"Sum: \" + sum.to_string());\ngreet_with_default(); // Uses default parameter"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="collections">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Collections</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Collections</h2>
                
                <h3>Lists</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating a list\nlet numbers = [1, 2, 3, 4, 5];\n\n// Accessing elements\nlet first = numbers[0];\n\n// Adding elements\nnumbers.push(6);\n\n// List methods\nlet length = numbers.len();\nlet contains_three = numbers.contains(3);\n\n// List comprehension\nlet squares = [x * x for x in numbers];"} />
                </div>
                
                <h3>Dictionaries</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating a dictionary\nlet user = {\n    \"name\": \"Alice\",\n    \"age\": 30,\n    \"is_admin\": true\n};\n\n// Accessing elements\nlet name = user[\"name\"];\n\n// Adding elements\nuser[\"email\"] = \"alice@example.com\";\n\n// Dictionary methods\nlet keys = user.keys();\nlet has_email = user.contains_key(\"email\");"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="structs">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Structs and Methods</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Structs and Methods</h2>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Defining a struct\nstruct Rectangle {\n    width: float,\n    height: float,\n    \n    // Method\n    fn area() -> float {\n        return self.width * self.height;\n    }\n    \n    // Static method (constructor)\n    fn new(w: float, h: float) -> Rectangle {\n        return Rectangle { width: w, height: h };\n    }\n}\n\n// Creating an instance\nlet rect1 = Rectangle { width: 10.0, height: 5.0 };\nlet rect2 = Rectangle::new(20.0, 15.0);\n\n// Calling methods\nlet area1 = rect1.area();\nlet area2 = rect2.area();\n\nprintln(\"Area 1: \" + area1.to_string());\nprintln(\"Area 2: \" + area2.to_string());"} />
                </div>
              </div>
              </section>
    </DocsShell>
  );
}
