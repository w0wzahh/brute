# Brute Website

The Brute programming language website — **Next.js 15 + TypeScript** (App Router),
exported as a fully static site (`output: "export"`) so it can be hosted on
Netlify or any static host.

## Stack

- **Next.js 15** (App Router) + **React 19** + **TypeScript**
- Plain CSS design system in `app/globals.css` (neobrutalist: black borders,
  hard offset shadows, flat saturated colors)
- Prism for syntax highlighting
- Font Awesome icons via CDN

## Structure

```
app/            # routes — page.tsx per page
  guide/  examples/  features/  reference/  api/  about/
  globals.css   # the design system
  layout.tsx    # shared shell (navbar, footer, fonts, theme init)
components/     # Navbar, Footer, Ticker, Window, SecHead, DocShell,
              # ExampleTabs, ThemeToggle, SiteEffects (all client behaviors)
public/images/  # static assets
netlify.toml    # builds `out/` via `npm run build`
```

## Development

```bash
npm install
npm run dev        # dev server on :3000
npm run build      # static export -> out/
```

On Windows, if the folder path contains `&`, use
`node node_modules/next/dist/bin/next dev` — npm's .cmd shims break on `&`.
