"use client";

import type { ReactNode } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";

export type TocItem = { id: string; title: string };

const DOC_LINKS = [
  { href: "/docs", icon: "fas fa-home", label: "Overview" },
  { href: "/docs/guide", icon: "fas fa-rocket", label: "Getting Started" },
  { href: "/docs/reference", icon: "fas fa-book-open", label: "Reference" },
  { href: "/docs/api", icon: "fas fa-plug", label: "Stdlib API" },
];

export default function DocsShell({
  toc,
  children,
}: {
  toc?: TocItem[];
  children: ReactNode;
}) {
  const pathname = usePathname();

  return (
    <>
      <div className="ex-tabs-wrap">
        <div className="container">
          <div className="ex-tabs">
            {DOC_LINKS.map((l) => (
              <Link
                key={l.href}
                href={l.href}
                className={`ex-tab${pathname === l.href ? " active" : ""}`}
              >
                <i className={l.icon} /> {l.label}
              </Link>
            ))}
            <a
              href="https://github.com/w0wzahh/brute"
              target="_blank"
              rel="noreferrer"
              className="ex-tab ex-gh"
            >
              <i className="fab fa-github" /> GitHub
            </a>
          </div>
          {toc && toc.length > 0 && (
            <div className="ex-sublinks toc-chips">
              {toc.map((t) => (
                <a key={t.id} href={`#${t.id}`}>
                  {t.title}
                </a>
              ))}
            </div>
          )}
        </div>
      </div>
      <div className="container">
        <main className="docs-main">{children}</main>
      </div>
    </>
  );
}
