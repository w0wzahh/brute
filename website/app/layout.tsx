import type { Metadata } from "next";
import SiteEffects from "@/components/SiteEffects";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Brute Programming Language",
    template: "%s — Brute",
  },
  description:
    "Brute — a compiled, statically-typed language: Rust's safety, Python's readability, C's speed.",
  icons: { icon: "/images/icon.svg" },
};

const themeInit = `
try {
  var t = localStorage.getItem("brute-theme") ||
    (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  document.documentElement.setAttribute("data-theme", t);
} catch (e) {}
`;

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Lexend+Mega:wght@400;500;600;700;800;900&family=Public+Sans:wght@400;500;600;700;800&family=Fira+Code:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
        <link
          rel="stylesheet"
          href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css"
        />
        <link
          rel="stylesheet"
          href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css"
        />
        <script dangerouslySetInnerHTML={{ __html: themeInit }} />
      </head>
      <body>
        {children}
        <SiteEffects />
      </body>
    </html>
  );
}
