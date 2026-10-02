import type { Metadata } from "next";

export const metadata: Metadata = { title: 'Brute - Features' };

export default function Page() {
  return (
    <>
      <section className="hero">
          <div className="container">
            <div className="hero-inner">
              <h1>BRUTE <span className="hollow">FEATURES</span></h1>
              <p className="sub">Everything Brute ships with — safety, speed, and a standard library that doesn't make you beg.</p>
            </div>
          </div>
        </section>
      
        
        <section id="features" className="features">
          <div className="container">
            <div className="window">
              <div className="win-bar"><span className="win-dots"><i></i><i></i><i></i></span><span className="win-title">features.exe</span><span className="win-actions"><span></span><span></span><span></span></span></div>
              <div className="win-body">
            <h2>Features</h2>
            <p>Brute combines the best aspects of modern programming languages to deliver performance, safety, and developer productivity.</p>
            
            <div className="features-grid">
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-shield-alt"></i>
                </div>
                <h3>Memory Safety</h3>
                <p>Inspired by Rust, Brute provides strong memory safety guarantees without the complexity of a garbage collector.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-tachometer-alt"></i>
                </div>
                <h3>High Performance</h3>
                <p>LLVM-based optimizing compiler that generates efficient native code with performance comparable to C/C++.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-code"></i>
                </div>
                <h3>Modern Syntax</h3>
                <p>Clean, Python-inspired syntax that emphasizes readability while maintaining the expressiveness of systems languages.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-pen"></i>
                </div>
                <h3>Built-in Formatter</h3>
                <p>Integrated code formatter ensures consistent style across your codebase, eliminating style debates.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-bolt"></i>
                </div>
                <h3>Async Support</h3>
                <p>First-class async/await syntax provides seamless asynchronous programming without callback hell.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-exclamation-triangle"></i>
                </div>
                <h3>Error Handling</h3>
                <p>Combination of Result types and try/catch blocks gives you flexibility in error handling approaches.</p>
              </div>
      
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-puzzle-piece"></i>
                </div>
                <h3>Traits &amp; Interfaces</h3>
                <p>Powerful trait system allows for composition-based code organization with clear interfaces.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-layer-group"></i>
                </div>
                <h3>List Comprehensions</h3>
                <p>Python-style list comprehensions enable powerful, concise data transformations.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-code-branch"></i>
                </div>
                <h3>Pipeline Operator</h3>
                <p>Functional programming with pipeline operator for clean, readable data transformations.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-check-double"></i>
                </div>
                <h3>Optional Chaining</h3>
                <p>Safely access properties deep in object hierarchies without worrying about null references.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-sitemap"></i>
                </div>
                <h3>Pattern Matching</h3>
                <p>Comprehensive pattern matching with destructuring, guards, and or-patterns for expressive code.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-box"></i>
                </div>
                <h3>Generics</h3>
                <p>First-class generic types and functions for type-safe, reusable code components.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-cubes"></i>
                </div>
                <h3>Module System</h3>
                <p>Robust module system with explicit imports and exports for better code organization.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-network-wired"></i>
                </div>
                <h3>Concurrency</h3>
                <p>Built-in concurrency primitives make parallel programming straightforward and safe.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-plug"></i>
                </div>
                <h3>Foreign Function Interface</h3>
                <p>Seamless interoperability with C/C++ libraries through a simple and safe FFI.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-vial"></i>
                </div>
                <h3>Built-in Testing</h3>
                <p>Integrated testing framework makes writing and running tests frictionless.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-clipboard-check"></i>
                </div>
                <h3>Static Analysis</h3>
                <p>Advanced static analysis catches bugs and ensures code quality before runtime.</p>
              </div>
              
              <div className="feature-card">
                <div className="feature-icon">
                  <i className="fas fa-archive"></i>
                </div>
                <h3>Package Management</h3>
                <p>Integrated package manager simplifies dependency management and sharing code.</p>
              </div>
            </div>
              </div>
            </div>
          </div>
        </section>
    </>
  );
}
