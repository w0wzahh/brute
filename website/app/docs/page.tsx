import type { Metadata } from "next";
import Link from "next/link";
import DocsShell from "@/components/DocsShell";
import Window from "@/components/Window";
import HiCode from "@/components/HiCode";

export const metadata: Metadata = { title: "Documentation" };

const TILES = [
  {
    href: "/docs/guide",
    cls: "",
    icon: "fas fa-rocket",
    title: "Getting Started",
    desc: "Install the toolchain, compile your first .brt file, and learn the core syntax — from variables to collections.",
  },
  {
    href: "/docs/reference",
    cls: "c-teal",
    icon: "fas fa-book-open",
    title: "Language Reference",
    desc: "The full spec: types, operators, control flow, generics, error handling, and the memory model.",
  },
  {
    href: "/docs/api",
    cls: "c-violet",
    icon: "fas fa-plug",
    title: "Stdlib API",
    desc: "All 12 standard-library modules — math, string, collections, async, net, crypto, and more.",
  },
];

export default function DocsHub() {
  return (
    <DocsShell>
      <h1 className="doc-title">Brute Documentation</h1>
      <p className="doc-sub">
        Everything you need to go from <code>cargo install</code> to shipping
        real programs. Pick a door.
      </p>

      <div className="docs-hub-grid">
        {TILES.map((t) => (
          <Link key={t.href} href={t.href} className={`explore-card ${t.cls}`}>
            <span className="ex-ico">
              <i className={t.icon} />
            </span>
            <h3>{t.title}</h3>
            <p>{t.desc}</p>
            <span className="ex-go">Open</span>
          </Link>
        ))}
      </div>

      <div style={{ marginTop: 34 }}>
        <Window title="quickstart.sh">
          <p style={{ marginBottom: 12 }}>
            Thirty-second version — this is all it takes:
          </p>
          <div className="code-block">
            <div className="code-header">
              <span className="code-lang">bash</span>
              <button className="copy-btn" aria-label="Copy code">
                <i className="fas fa-copy" />
              </button>
            </div>
            <HiCode
              lang="bash"
              code={`# install and run your first program
cargo install brute-lang
brute new hello && cd hello
brute run`}
            />
          </div>
          <p>
            Stuck?{" "}
            <a
              href="https://github.com/w0wzahh/brute/issues"
              target="_blank"
              rel="noreferrer"
              style={{ fontWeight: 700 }}
            >
              Open an issue
            </a>{" "}
            — or browse <Link href="/examples" style={{ fontWeight: 700 }}>the examples</Link> and steal them.
          </p>
        </Window>
      </div>
    </DocsShell>
  );
}
