import Link from "next/link";
import Ticker from "./Ticker";

export default function Footer() {
  return (
    <>
      <Ticker
        band="pink"
        reverse
        items={[
          "BRUTE",
          "OPEN SOURCE",
          "MIT LICENSE",
          "EST. 2025",
          "cargo build --release",
          "*.brt",
        ]}
      />
      <footer>
        <div className="container">
          <div className="footer-content">
            <div className="footer-column">
              <h3>Brute</h3>
              <p>
                A programming language combining features from Rust, Python, and
                C/C++.
              </p>
              <p>Released under the MIT License.</p>
            </div>
            <div className="footer-column">
              <h3>Site</h3>
              <ul className="footer-links">
                <li><Link href="/features">Features</Link></li>
                <li><Link href="/examples">Examples</Link></li>
                <li><Link href="/about">About</Link></li>
              </ul>
            </div>
            <div className="footer-column">
              <h3>Docs</h3>
              <ul className="footer-links">
                <li><Link href="/docs">Overview</Link></li>
                <li><Link href="/docs/guide">Guide</Link></li>
                <li><Link href="/docs/reference">Reference</Link></li>
                <li><Link href="/docs/api">API</Link></li>
              </ul>
            </div>
            <div className="footer-column">
              <h3>Developer</h3>
              <ul className="footer-links">
                <li><a href="https://github.com/w0wzahh">w0wzahh</a></li>
                <li><a href="https://github.com/w0wzahh/brute">Repository</a></li>
              </ul>
            </div>
          </div>
          <div className="footer-bottom">
            <p>&copy; 2025 Brute Language. Developed by w0wzahh. All Rights Reserved.</p>
          </div>
        </div>
      </footer>
      <button className="back-to-top" aria-label="Back to top">
        <i className="fas fa-arrow-up" />
      </button>
    </>
  );
}
