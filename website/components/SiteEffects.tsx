"use client";

import { useEffect } from "react";
import { usePathname } from "next/navigation";

/**
 * Global delegated behaviors — replaces the old main.js.
 * Re-initializes per-route since App Router swaps content client-side.
 */
export default function SiteEffects() {
  const pathname = usePathname();

  useEffect(() => {
    /* ── Header shadow + back-to-top visibility ─────────────── */
    const header = document.querySelector("header");
    const btt = document.querySelector(".back-to-top");
    const onScroll = () => {
      header?.classList.toggle("scrolled", window.scrollY > 24);
      btt?.classList.toggle("visible", window.scrollY > 500);
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();

    const onBtt = (e: Event) => {
      if ((e.target as HTMLElement).closest(".back-to-top"))
        window.scrollTo({ top: 0, behavior: "smooth" });
    };
    document.addEventListener("click", onBtt);

    /* ── Copy buttons (delegated) ───────────────────────────── */
    const onCopy = (e: Event) => {
      const btn = (e.target as HTMLElement).closest(".copy-btn");
      if (!btn) return;
      const scope = btn.closest(".code-block, .term-chip");
      const code = scope?.querySelector("code");
      if (!code) return;
      navigator.clipboard.writeText(code.textContent || "").then(() => {
        btn.classList.add("copied");
        const icon = btn.querySelector("i");
        const prev = icon?.className;
        if (icon) icon.className = "fas fa-check";
        setTimeout(() => {
          btn.classList.remove("copied");
          if (icon && prev) icon.className = prev;
        }, 1600);
      });
    };
    document.addEventListener("click", onCopy);

    /* ── Legacy .tabs (data-target) ─────────────────────────── */
    const onTab = (e: Event) => {
      const btn = (e.target as HTMLElement).closest(".tab-btn");
      if (!btn) return;
      const tabs = btn.closest(".tabs");
      const target = btn.getAttribute("data-target");
      tabs?.querySelectorAll(".tab-btn").forEach((b) => b.classList.remove("active"));
      tabs?.querySelectorAll(".tab-content").forEach((c) => c.classList.remove("active"));
      btn.classList.add("active");
      if (target) document.getElementById(target)?.classList.add("active");
    };
    document.addEventListener("click", onTab);

    /* ── Stat numbers — count up ────────────────────────────── */
    const statIO = new IntersectionObserver(
      (entries) =>
        entries.forEach((en) => {
          if (!en.isIntersecting) return;
          const num = en.target as HTMLElement;
          const target = parseInt(num.dataset.count || "0", 10);
          const t0 = performance.now();
          const tick = (t: number) => {
            const k = Math.min(1, (t - t0) / 1000);
            num.textContent = String(Math.round(target * (1 - Math.pow(1 - k, 3))));
            if (k < 1) requestAnimationFrame(tick);
          };
          requestAnimationFrame(tick);
          statIO.unobserve(num);
        }),
      { threshold: 0.4 }
    );
    document
      .querySelectorAll(".stat-num[data-count]")
      .forEach((n) => statIO.observe(n));

    /* ── TOC chips scrollspy ────────────────────────────────── */
    const tocLinks = document.querySelectorAll(".toc-chips a[href^='#']");
    let spy: IntersectionObserver | null = null;
    if (tocLinks.length) {
      const targets = Array.from(tocLinks)
        .map((a) => document.getElementById(a.getAttribute("href")!.slice(1)))
        .filter(Boolean) as HTMLElement[];
      spy = new IntersectionObserver(
        (entries) =>
          entries.forEach((en) => {
            if (en.isIntersecting)
              tocLinks.forEach((a) =>
                a.classList.toggle(
                  "active",
                  a.getAttribute("href") === "#" + en.target.id
                )
              );
          }),
        { rootMargin: "-20% 0px -65% 0px" }
      );
      targets.forEach((t) => spy!.observe(t));
    }

    /* ── Scroll reveal ──────────────────────────────────────── */
    const revealEls = document.querySelectorAll(
      ".feature-card, .doc-card, .code-block, .doc-section, " +
        ".examples-cta, .docs-cta, .community-cta, .coming-soon, " +
        ".window, .explore-card, .stat-card, .documentation-header, " +
        ".ex-tabs-wrap, .mini-chip, .sec-head, " +
        ".project-card, .skill-category, " +
        "section > .container > h2, section > .container > h3"
    );
    revealEls.forEach((el) => el.classList.add("reveal"));
    document
      .querySelectorAll(".features-grid, .docs-hub-grid, .explore-grid")
      .forEach((grid) =>
        Array.from(grid.children).forEach((child, i) =>
          child.classList.add(`reveal-delay-${(i % 3) + 1}`)
        )
      );
    const io = new IntersectionObserver(
      (entries) =>
        entries.forEach((en) => {
          if (en.isIntersecting) {
            en.target.classList.add("revealed");
            io.unobserve(en.target);
          }
        }),
      { threshold: 0, rootMargin: "0px 0px -40px 0px" }
    );
    revealEls.forEach((el) => io.observe(el));
    // Safety net: anything the observer misses (huge blocks, tab-hidden
    // sections re-shown later) reveals after a short delay instead of
    // staying invisible forever.
    const revealFallback = setTimeout(
      () =>
        document
          .querySelectorAll(".reveal:not(.revealed)")
          .forEach((el) => el.classList.add("revealed")),
      2500
    );

    /* ── Cursor-tracking sheen vars ─────────────────────────── */
    const sheenEls = document.querySelectorAll<HTMLElement>(
      ".btn, .feature-card"
    );
    const onMove = (e: Event) => {
      const el = e.currentTarget as HTMLElement;
      const r = el.getBoundingClientRect();
      const pe = e as PointerEvent;
      el.style.setProperty("--mx", `${((pe.clientX - r.left) / r.width) * 100}%`);
      el.style.setProperty("--my", `${((pe.clientY - r.top) / r.height) * 100}%`);
    };
    sheenEls.forEach((el) => el.addEventListener("pointermove", onMove));

    return () => {
      window.removeEventListener("scroll", onScroll);
      document.removeEventListener("click", onBtt);
      document.removeEventListener("click", onCopy);
      document.removeEventListener("click", onTab);
      clearTimeout(revealFallback);
      statIO.disconnect();
      spy?.disconnect();
      io.disconnect();
      sheenEls.forEach((el) => el.removeEventListener("pointermove", onMove));
    };
  }, [pathname]);

  return null;
}
