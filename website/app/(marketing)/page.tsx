import Link from "next/link";
import Ticker from "@/components/Ticker";
import Window from "@/components/Window";
import SecHead from "@/components/SecHead";
import HiCode from "@/components/HiCode";

const STATS = [
  { n: "18", l: "CORE FEATURES" },
  { n: "12", l: "STDLIB MODULES" },
  { n: "16", l: "EXAMPLE PROGRAMS" },
  { n: "0", l: "GARBAGE PAUSES" },
];

const MINI = [
  { icon: "fas fa-shield-halved", label: "Memory safe" },
  { icon: "fas fa-bolt", label: "No GC" },
  { icon: "fas fa-infinity", label: "Async built-in" },
  { icon: "fas fa-code-branch", label: "Pipelines" },
];

const CARDS = [
  {
    href: "/docs/guide",
    cls: "",
    icon: "fas fa-rocket",
    title: "Get Started",
    desc: "Install Brute, write hello_world.brt, run it. Five minutes, start to finish.",
    go: "Open the guide",
  },
  {
    href: "/examples",
    cls: "c-pink",
    icon: "fas fa-code",
    title: "See Examples",
    desc: "16 real programs — variables to web servers — in syntax you already know.",
    go: "Browse examples",
  },
  {
    href: "/docs/reference",
    cls: "c-teal",
    icon: "fas fa-book-open",
    title: "Language Reference",
    desc: "Every keyword, type, and operator. The full spec, readable by humans.",
    go: "Read the spec",
  },
  {
    href: "/docs/api",
    cls: "c-violet",
    icon: "fas fa-plug",
    title: "Stdlib API",
    desc: "Math, strings, collections, async, net — the batteries that are included.",
    go: "Explore stdlib",
  },
  {
    href: "/features",
    cls: "c-blue",
    icon: "fas fa-star",
    title: "Feature List",
    desc: "Safety, zero-cost abstractions, hybrid memory, and 15 more reasons.",
    go: "All 18 features",
  },
  {
    href: "/about",
    cls: "c-green",
    icon: "fas fa-user",
    title: "About the Dev",
    desc: "Who built this, why it exists, and where it's heading next.",
    go: "Meet w0wzahh",
  },
];

export default function Home() {
  return (
    <>
      {/* ── Hero ── */}
      <section className="hero">
        <div className="container">
          <div className="hero-inner">
            <div className="hero-split">
              <div>
                <h1 className="hero-in hero-in-1">
                  BRUTE
                  <br />
                  <span className="hollow">STRENGTH.</span>
                </h1>
                <p className="tagline hero-in hero-in-2">
                  <span className="hl">Safety</span> of Rust, syntax of{" "}
                  <span className="hl">Python</span>, speed of{" "}
                  <span className="hl">C</span>.
                </p>
                <p className="sub hero-in hero-in-3">
                  A compiled, statically-typed language that refuses to pick a
                  fight — clear syntax, no garbage collector, real error
                  messages.
                </p>
                <div className="hero-buttons hero-in hero-in-4">
                  <Link href="/docs/guide" className="btn btn-primary">
                    <i className="fas fa-play" /> Get Started
                  </Link>
                  <a
                    href="https://github.com/w0wzahh/brute"
                    target="_blank"
                    rel="noreferrer"
                    className="btn"
                  >
                    <i className="fab fa-github" /> GitHub
                  </a>
                </div>
                <div className="hero-install hero-in hero-in-5">
                  <div className="term-chip">
                    <span className="term-prompt">❯</span>
                    <code>cargo install brute-lang</code>
                    <button className="copy-btn" aria-label="Copy command">
                      <i className="fas fa-copy" />
                    </button>
                  </div>
                  <a
                    href="https://github.com/w0wzahh/brute/releases/latest"
                    target="_blank"
                    rel="noreferrer"
                    className="btn btn-sm dl-btn"
                  >
                    <i className="fas fa-download" /> Download binary
                  </a>
                </div>
              </div>

              <div className="hero-code">
              <div className="code-window">
                <div className="win-bar">
                  <span className="win-dots">
                    <i />
                    <i />
                    <i />
                  </span>
                  <span className="win-title">hello_world.brt</span>
                  <span className="win-actions">
                    <span />
                    <span />
                    <span />
                  </span>
                </div>
                <HiCode
                  lang="rust"
                  code={`// your first brute program
fn main() {
    let name = "world";

    match name {
        "world" => println("hello, $\{name}!"),
        _       => println("hi anyway"),
    }

    [1, 2, 3, 4, 5]
        |> map(*2)
        |> filter(>4)
        |> println(); // [6, 8, 10]
}`}
                />
              </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Ticker
        items={[
          "MEMORY SAFE",
          "ZERO COST ABSTRACTIONS",
          "ASYNC WITHOUT THE PAIN",
          "ONE BINARY, NO RUNTIME",
          "ERRORS THAT HELP",
          "PIPES, NOT PYRAMIDS",
        ]}
      />

      {/* ── Stats ── */}
      <section className="stats">
        <div className="container">
          <div className="sec-head" style={{ marginBottom: 22 }}>
            <span className="sec-no">01</span>
            <h2 style={{ margin: 0 }}>The numbers</h2>
          </div>
          <div className="stats-row">
            {STATS.map((s, i) => (
              <div key={i} className="stat-card">
                <span className="stat-num" data-count={s.n}>
                  0
                </span>
                <h3>{s.l}</h3>
              </div>
            ))}
          </div>
          <div className="mini-feats">
            {MINI.map((m) => (
              <Link key={m.label} href="/features" className="mini-chip">
                <i className={m.icon} /> {m.label}
              </Link>
            ))}
            <Link href="/features" className="mini-chip more">
              All 18 features <i className="fas fa-arrow-right" />
            </Link>
          </div>
        </div>
      </section>

      {/* ── Explore hub ── */}
      <section className="documentation">
        <div className="container">
          <div className="sec-head" style={{ marginBottom: 8 }}>
            <span className="sec-no">02</span>
            <h2 style={{ margin: 0 }}>Pick a window</h2>
          </div>
          <p style={{ marginBottom: 4 }}>
            Everything&apos;s a panel. Click through — no endless scrolling here.
          </p>
          <div className="explore-grid">
            {CARDS.map((c) => (
              <Link key={c.href} href={c.href} className={`explore-card ${c.cls}`}>
                <span className="ex-ico">
                  <i className={c.icon} />
                </span>
                <h3>{c.title}</h3>
                <p>{c.desc}</p>
                <span className="ex-go">{c.go}</span>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* ── CTA ── */}
      <section style={{ padding: "0 0 72px" }}>
        <div className="container">
          <div className="sec-head" style={{ marginBottom: 18 }}>
            <span className="sec-no">03</span>
            <h2 style={{ margin: 0 }}>Open source</h2>
          </div>
          <Window title="contribute.exe">
            <div className="cta-body" style={{ padding: 0 }}>
              <div>
                <h3 style={{ margin: "0 0 6px" }}>Break things, politely.</h3>
                <p style={{ maxWidth: 540 }}>
                  Brute is MIT-licensed and actively built. Issues, PRs, and
                  feature ideas all welcome — the compiler won&apos;t judge you.
                </p>
              </div>
              <div className="cta-actions">
                <a
                  href="https://github.com/w0wzahh/brute"
                  target="_blank"
                  rel="noreferrer"
                  className="btn btn-primary btn-sm"
                >
                  <i className="fab fa-github" /> Star on GitHub
                </a>
                <Link href="/docs/guide" className="btn btn-sm">
                  <i className="fas fa-book" /> Read the docs
                </Link>
              </div>
            </div>
          </Window>
        </div>
      </section>
    </>
  );
}
