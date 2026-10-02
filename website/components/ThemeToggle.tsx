"use client";

import { useEffect, useState } from "react";

export default function ThemeToggle() {
  const [dark, setDark] = useState(false);

  useEffect(() => {
    setDark(document.documentElement.getAttribute("data-theme") === "dark");
  }, []);

  const toggle = () => {
    const next = dark ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    localStorage.setItem("brute-theme", next);
    setDark(!dark);
  };

  return (
    <button className="theme-toggle" aria-label="Toggle theme" onClick={toggle}>
      <i className={dark ? "fas fa-sun" : "fas fa-moon"} />
    </button>
  );
}
