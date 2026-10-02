"use client";

import { useEffect, useState, type ReactNode } from "react";

export type ExCat = "start" | "data" | "adv" | "real" | "all";

const TABS: { cat: ExCat; icon: string; label: string }[] = [
  { cat: "start", icon: "fas fa-rocket", label: "Getting Started" },
  { cat: "data", icon: "fas fa-database", label: "Data Structures" },
  { cat: "adv", icon: "fas fa-cogs", label: "Advanced" },
  { cat: "real", icon: "fas fa-globe", label: "Real World" },
  { cat: "all", icon: "fas fa-border-all", label: "All" },
];

const SUBS: Record<ExCat, { id: string; label: string }[]> = {
  start: [
    { id: "hello-world", label: "hello_world.brt" },
    { id: "variables", label: "variables" },
    { id: "control-flow", label: "control flow" },
    { id: "functions", label: "functions" },
  ],
  data: [
    { id: "structs", label: "structs" },
    { id: "enums", label: "enums" },
    { id: "collections", label: "collections" },
    { id: "strings", label: "strings" },
  ],
  adv: [
    { id: "traits", label: "traits & generics" },
    { id: "error-handling", label: "error handling" },
    { id: "async", label: "async" },
    { id: "pipeline", label: "pipelines" },
  ],
  real: [
    { id: "web-server", label: "web server" },
    { id: "data-processing", label: "data processing" },
    { id: "algorithms", label: "algorithms" },
    { id: "game", label: "game" },
  ],
  all: [
    { id: "hello-world", label: "hello world" },
    { id: "variables", label: "variables" },
    { id: "control-flow", label: "control flow" },
    { id: "functions", label: "functions" },
    { id: "structs", label: "structs" },
    { id: "enums", label: "enums" },
    { id: "collections", label: "collections" },
    { id: "strings", label: "strings" },
    { id: "traits", label: "traits" },
    { id: "error-handling", label: "errors" },
    { id: "async", label: "async" },
    { id: "pipeline", label: "pipelines" },
    { id: "web-server", label: "web server" },
    { id: "data-processing", label: "data" },
    { id: "algorithms", label: "algorithms" },
    { id: "game", label: "game" },
  ],
};

export default function ExampleTabs({ children }: { children: ReactNode }) {
  const [cat, setCat] = useState<ExCat>("start");

  useEffect(() => {
    document.querySelectorAll<HTMLElement>(".code-section[data-cat]").forEach((sec) => {
      const on = cat === "all" || sec.dataset.cat === cat;
      sec.classList.toggle("cat-hidden", !on);
      sec.classList.remove("cat-in");
      if (on) {
        void sec.offsetWidth;
        sec.classList.add("cat-in");
      }
    });
  }, [cat]);

  useEffect(() => {
    const q = new URLSearchParams(window.location.search).get("cat") as ExCat | null;
    const hash = window.location.hash.slice(1);
    if (hash) {
      const sec = document.getElementById(hash);
      const c = sec?.dataset.cat as ExCat | undefined;
      if (c) {
        setCat(c);
        requestAnimationFrame(() =>
          sec?.scrollIntoView({ behavior: "smooth", block: "start" })
        );
        return;
      }
    }
    if (q && TABS.some((t) => t.cat === q)) setCat(q);
  }, []);

  const jump = (id: string) => (e: React.MouseEvent) => {
    e.preventDefault();
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
    history.replaceState(null, "", `#${id}`);
  };

  return (
    <>
      <div className="ex-tabs-wrap">
        <div className="container">
          <div className="ex-tabs">
            {TABS.map((t) => (
              <button
                key={t.cat}
                className={`ex-tab${cat === t.cat ? " active" : ""}`}
                onClick={() => {
                  setCat(t.cat);
                  window.scrollTo({ top: 0, behavior: "smooth" });
                }}
              >
                <i className={t.icon} /> {t.label}
              </button>
            ))}
          </div>
          <div className="ex-sublinks">
            {SUBS[cat].map((s) => (
              <a key={s.id} href={`#${s.id}`} onClick={jump(s.id)}>
                {s.label}
              </a>
            ))}
          </div>
        </div>
      </div>
      {children}
    </>
  );
}
