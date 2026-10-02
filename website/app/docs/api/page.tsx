import type { Metadata } from "next";
import DocsShell from "@/components/DocsShell";
import HiCode from "@/components/HiCode";


export const metadata: Metadata = { title: 'Brute - API Documentation' };

const TOC = [
  {
    "id": "core",
    "title": "Core Library"
  },
  {
    "id": "standard",
    "title": "Standard Library"
  },
  {
    "id": "io",
    "title": "I/O Module"
  },
  {
    "id": "collections",
    "title": "Collections Module"
  },
  {
    "id": "math",
    "title": "Math Module"
  },
  {
    "id": "string",
    "title": "String Module"
  },
  {
    "id": "thread",
    "title": "Thread Module"
  },
  {
    "id": "time",
    "title": "Time Module"
  }
];

export default function Page() {
  return (
    <DocsShell toc={TOC}>
      <h1 className="doc-title">Brute API Documentation</h1>
      <p className="doc-sub">Complete API reference for the Brute standard library and core modules.</p>
      <section className="window" id="core">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Core Library</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Core Library</h2>
                <p>The core library provides fundamental types and functions that form the foundation of the language.</p>
                
                <h3>Basic Types</h3>
                <div className="api-table">
                  <table>
                    <thead>
                      <tr>
                        <th>Type</th>
                        <th>Description</th>
                        <th>Examples</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <td><code>{"int"}</code></td>
                        <td>Signed integer type</td>
                        <td><code>{"42"}</code>, <code>{"-7"}</code>, <code>{"0"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"float"}</code></td>
                        <td>Floating-point number</td>
                        <td><code>{"3.14"}</code>, <code>{"-0.5"}</code>, <code>{"2.71e2"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"bool"}</code></td>
                        <td>Boolean type</td>
                        <td><code>{"true"}</code>, <code>{"false"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"char"}</code></td>
                        <td>Single Unicode character</td>
                        <td><code>{"'A'"}</code>, <code>{"'1'"}</code>, <code>{"'\\n'"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"string"}</code></td>
                        <td>UTF-8 encoded string</td>
                        <td><code>{"\"Hello\""}</code>, <code>{"\"\""}</code>, <code>{"\"Line\\nBreak\""}</code></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
                
                <h3>Core Functions</h3>
                <div className="api-table">
                  <table>
                    <thead>
                      <tr>
                        <th>Function</th>
                        <th>Description</th>
                        <th>Example</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <td><code>{"print(value)"}</code></td>
                        <td>Prints a value to standard output without a newline</td>
                        <td><code>{"print(\"Hello\")"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"println(value)"}</code></td>
                        <td>Prints a value to standard output with a newline</td>
                        <td><code>{"println(\"Hello, World!\")"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"input(prompt)"}</code></td>
                        <td>Reads a line from standard input with an optional prompt</td>
                        <td><code>{"let name = input(\"Enter your name: \")"}</code></td>
                      </tr>
                      <tr>
                        <td><code>{"typeof(value)"}</code></td>
                        <td>Returns the type of a value as a string</td>
                        <td><code>{"let type = typeof(42)"}</code> // "int"</td>
                      </tr>
                      <tr>
                        <td><code>{"panic(message)"}</code></td>
                        <td>Terminates the program with an error message</td>
                        <td><code>{"panic(\"Something went wrong!\")"}</code></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
              </section>
              
              <section className="window" id="standard">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Standard Library</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Standard Library</h2>
                <p>The standard library provides a comprehensive set of utilities and data structures for everyday programming tasks.</p>
                
                <h3>Result Type</h3>
                <p>The <code>{"Result<T, E>"}</code> type is used for error handling and represents either a success value of type <code>{"T"}</code> or an error value of type <code>{"E"}</code>.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating Results\nlet success: Result<int, string> = Ok(42);\nlet failure: Result<int, string> = Err(\"Something went wrong\");\n\n// Methods on Result\nif result.is_ok() {\n    let value = result.unwrap();  // Gets the value (panics if it's an Err)\n}\n\nif result.is_err() {\n    let error = result.unwrap_err();  // Gets the error (panics if it's an Ok)\n}\n\n// Pattern matching on Result\nmatch result {\n    Ok(value) => println(\"Success: \" + value.to_string()),\n    Err(error) => println(\"Error: \" + error),\n}\n\n// Using the ? operator for error propagation\nfn process() -> Result<int, string> {\n    let value = risky_operation()?;  // Returns early if error\n    return Ok(value * 2);\n}"} />
                </div>
                
                <h3>Option Type</h3>
                <p>The <code>{"Option<T>"}</code> type represents a value that may or may not be present.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating Options\nlet some_value: Option<int> = Some(42);\nlet no_value: Option<int> = None;\n\n// Methods on Option\nif option.is_some() {\n    let value = option.unwrap();  // Gets the value (panics if it's None)\n}\n\nif option.is_none() {\n    // Handle the None case\n}\n\n// Pattern matching on Option\nmatch option {\n    Some(value) => println(\"Value: \" + value.to_string()),\n    None => println(\"No value present\"),\n}\n\n// Using unwrap_or to provide a default value\nlet value = option.unwrap_or(0);"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="io">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">I/O Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>I/O Module</h2>
                <p>The I/O module provides functions and types for input/output operations.</p>
                
                <h3>File Operations</h3>
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"import io;\n\n// Reading a file\nlet contents = io.read_file(\"example.txt\");\n\n// Writing to a file\nio.write_file(\"output.txt\", \"Hello, World!\");\n\n// Appending to a file\nio.append_file(\"log.txt\", \"New log entry\");\n\n// Reading a file line by line\nfor line in io.read_lines(\"data.txt\") {\n    println(line);\n}\n\n// Using the File type for more control\nlet file = io.File::open(\"example.txt\", \"r\");\nlet line = file.read_line();\nfile.close();"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="collections">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Collections Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Collections Module</h2>
                <p>The collections module provides various data structures for storing and manipulating collections of data.</p>
                
                <h3>List</h3>
                <p>Lists are ordered collections of elements.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating a list\nlet numbers = [1, 2, 3, 4, 5];\n\n// Accessing elements\nlet first = numbers[0];  // Indexing starts at 0\nlet last = numbers[numbers.len() - 1];\n\n// Methods\nnumbers.push(6);        // Add an element at the end\nnumbers.pop();          // Remove and return the last element\nnumbers.insert(0, 0);   // Insert at a specific index\nnumbers.remove(2);      // Remove at a specific index\nlet length = numbers.len();  // Get the length\nlet contains = numbers.contains(3);  // Check if element exists\nnumbers.clear();        // Remove all elements\n\n// Iterating\nfor number in numbers {\n    println(number.to_string());\n}"} />
                </div>
                
                <h3>Dictionary</h3>
                <p>Dictionaries are collections of key-value pairs.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"// Creating a dictionary\nlet user = {\n    \"name\": \"Alice\",\n    \"age\": 30,\n    \"is_admin\": true\n};\n\n// Accessing elements\nlet name = user[\"name\"];\nlet age = user.get(\"age\");  // Returns an Option\n\n// Methods\nuser[\"email\"] = \"alice@example.com\";  // Add or update a key-value pair\nuser.remove(\"is_admin\");     // Remove a key-value pair\nlet has_key = user.contains_key(\"name\");  // Check if key exists\nlet keys = user.keys();      // Get a list of keys\nlet values = user.values();  // Get a list of values\nlet length = user.len();     // Get the number of entries\nuser.clear();               // Remove all entries\n\n// Iterating\nfor key in user.keys() {\n    println(key + \": \" + user[key].to_string());\n}\n\nfor (key, value) in user {\n    println(key + \": \" + value.to_string());\n}"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="math">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Math Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Math Module</h2>
                <p>The math module provides mathematical functions and constants.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"import math;\n\n// Constants\nlet pi = math.PI;\nlet e = math.E;\n\n// Basic functions\nlet abs_value = math.abs(-5);           // Absolute value\nlet power = math.pow(2, 8);             // Power\nlet square_root = math.sqrt(16);        // Square root\nlet round_val = math.round(3.7);        // Round to nearest integer\nlet floor_val = math.floor(3.7);        // Round down\nlet ceil_val = math.ceil(3.2);          // Round up\n\n// Trigonometric functions\nlet sin_val = math.sin(math.PI / 2);    // Sine\nlet cos_val = math.cos(math.PI);        // Cosine\nlet tan_val = math.tan(math.PI / 4);    // Tangent\n\n// Other functions\nlet log_val = math.log(100, 10);        // Logarithm\nlet ln_val = math.ln(math.E);           // Natural logarithm\nlet exp_val = math.exp(2);              // Exponential"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="string">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">String Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>String Module</h2>
                <p>The string module provides functions for working with strings.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"import string;\n\n// String creation\nlet str1 = \"Hello\";\nlet str2 = string.from_char_code(65);  // \"A\"\n\n// String operations\nlet length = str1.len();               // Length in characters\nlet concat = str1 + \", World!\";        // Concatenation\nlet upper = str1.to_upper();           // Convert to uppercase\nlet lower = str1.to_lower();           // Convert to lowercase\nlet char_at = str1[0];                 // Get character at index\nlet substring = str1.substring(1, 3);  // Get substring (inclusive, exclusive)\nlet trimmed = \"  text  \".trim();       // Remove leading/trailing whitespace\n\n// Searching and replacing\nlet contains = str1.contains(\"el\");            // Check if contains substring\nlet starts_with = str1.starts_with(\"He\");      // Check if starts with substring\nlet ends_with = str1.ends_with(\"lo\");          // Check if ends with substring\nlet index = str1.index_of(\"l\");                // Find index of first occurrence\nlet last_index = str1.last_index_of(\"l\");      // Find index of last occurrence\nlet replaced = str1.replace(\"l\", \"L\");         // Replace all occurrences\n\n// Splitting and joining\nlet parts = \"a,b,c\".split(\",\");                // Split into array\nlet joined = string.join([\"a\", \"b\", \"c\"], \"-\"); // Join array with delimiter"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="thread">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Thread Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Thread Module</h2>
                <p>The thread module provides facilities for concurrent and parallel programming.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"import thread;\n\n// Creating threads\nlet handle = thread.spawn(fn() -> void {\n    println(\"Hello from thread!\");\n});\n\n// Waiting for threads to finish\nhandle.join();\n\n// Thread sleep\nthread.sleep(1000);  // Sleep for 1000 milliseconds\n\n// Using a mutex for thread synchronization\nlet mutex = thread.Mutex::new(0);\n\nlet handle1 = thread.spawn(fn() -> void {\n    let mut value = mutex.lock();\n    *value = *value + 1;\n});\n\nlet handle2 = thread.spawn(fn() -> void {\n    let mut value = mutex.lock();\n    *value = *value + 1;\n});\n\nhandle1.join();\nhandle2.join();\n\nlet final_value = *mutex.lock();  // 2"} />
                </div>
              </div>
              </section>
              
              <section className="window" id="time">
                <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">Time Module</span><span className="win-actions"><span></span><span></span><span></span></span></div>
                <div className="win-body">
                <h2>Time Module</h2>
                <p>The time module provides functions for working with dates and times.</p>
                
                <div className="code-block">
                  <div className="code-header">
                    <span className="code-lang">rust</span>
                    <button className="copy-btn"><i className="far fa-copy"></i></button>
                  </div>
                  <HiCode lang="rust" code={"import time;\n\n// Getting current time\nlet now = time.now();\nlet timestamp = time.timestamp();  // Unix timestamp in seconds\n\n// Creating time from components\nlet date = time.Date::new(2023, 1, 31);\nlet date_time = time.DateTime::new(2023, 1, 31, 12, 30, 0);\n\n// Formatting dates\nlet formatted = date.format(\"%Y-%m-%d\");  // \"2023-01-31\"\n\n// Parsing dates\nlet parsed = time.parse(\"2023-01-31\", \"%Y-%m-%d\");\n\n// Date/time operations\nlet tomorrow = date.add_days(1);\nlet next_month = date.add_months(1);\nlet next_year = date.add_years(1);\nlet diff = date_time - time.now();  // Time difference in seconds\n\n// Getting components\nlet year = date.year();\nlet month = date.month();\nlet day = date.day();\nlet hour = date_time.hour();\nlet minute = date_time.minute();\nlet second = date_time.second();\nlet weekday = date.weekday();  // 0 = Sunday, 6 = Saturday"} />
                </div>
              </div>
              </section>
    </DocsShell>
  );
}
