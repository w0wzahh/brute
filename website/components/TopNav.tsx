"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { usePathname } from "next/navigation";
import ThemeToggle from "./ThemeToggle";

const LINKS = [
  { href: "/", label: "Home" },
  { href: "/features", label: "Features" },
  { href: "/examples", label: "Examples" },
  { href: "/docs", label: "Docs" },
  { href: "/about", label: "About" },
];

export default function TopNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  const isActive = (href: string) =>
    href === "/" ? pathname === "/" : pathname.startsWith(href);

  return (
    <header className="topnav">
      <div className="container">
        <div className="topnav-inner">
          <Link href="/" className="logo-container">
            <Image
              src="/images/icon.svg"
              alt="Brute logo"
              className="logo"
              width={40}
              height={40}
              unoptimized
            />
            <span className="logo-text">Brute</span>
          </Link>

          <ul className={`topnav-links${open ? " open" : ""}`}>
            {LINKS.map(({ href, label }) => (
              <li key={href}>
                <Link
                  href={href}
                  className={isActive(href) ? "active" : ""}
                  onClick={() => setOpen(false)}
                >
                  {label}
                </Link>
              </li>
            ))}
          </ul>
          <ThemeToggle />
          <button
            className="hamburger"
            aria-label="Toggle menu"
            onClick={() => setOpen(!open)}
          >
            <i className={open ? "fas fa-times" : "fas fa-bars"} />
          </button>
        </div>
      </div>
    </header>
  );
}
