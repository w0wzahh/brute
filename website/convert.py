#!/usr/bin/env python3
"""Migration: convert legacy static pages' content to JSX in app/ dirs.
Docs pages (guide/reference/api) are wrapped in DocsShell with an
extracted per-page TOC for the sidebar."""
import re, html, json, pathlib

ROOT = pathlib.Path(__file__).parent
SRC = ROOT.parent / 'website-legacy'

_TAGS = (
    'a|abbr|address|area|article|aside|audio|b|base|bdi|bdo|blockquote|br|'
    'button|canvas|caption|cite|code|col|colgroup|data|datalist|dd|del|'
    'details|dfn|dialog|div|dl|dt|em|embed|fieldset|figcaption|figure|footer|'
    'form|h[1-6]|head|header|hgroup|hr|html|i|iframe|img|input|ins|kbd|label|'
    'legend|li|link|main|map|mark|menu|meta|meter|nav|noscript|object|ol|'
    'optgroup|option|output|p|picture|pre|progress|q|rp|rt|ruby|s|samp|script|'
    'search|section|select|slot|small|source|span|strong|style|sub|summary|'
    'sup|table|tbody|td|template|textarea|tfoot|th|thead|time|title|tr|track|'
    'u|ul|var|video|wbr|svg|path|circle|rect|line|polyline|polygon|g|defs|'
    'lineargradient|radialgradient|stop|text|tspan|ellipse|use|symbol'
)
TAG_RE = re.compile(
    r'</?(?:' + _TAGS + r')\b(?:\s[^>]*)?\s*/?>')

def jsxify(h: str) -> str:
    h = re.sub(r'<!--.*?-->', '', h, flags=re.S)

    # <code>/<pre> inner content is literal text. JSX collapses newlines in
    # text nodes, so emit it as a JS string expression {"...\n..."} — keeps
    # newlines and never produces live elements (Result<int>, <html>, etc).
    # Stashed with \x01 sentinels so the { } survive the escaping phase.
    exprs = []
    def code_escape(m):
        inner = html.unescape(m.group(2)).strip('\n')
        exprs.append('{' + json.dumps(inner) + '}')
        return m.group(1) + f'\x01{len(exprs)-1}\x01' + m.group(3)
    h = re.sub(r'(<code\b[^>]*>)(.*?)(</code>)', code_escape, h, flags=re.S)

    # protect real tags, then escape text-level specials
    tags = []
    def stash(m):
        tags.append(m.group(0))
        return f'\x00{len(tags)-1}\x00'
    h = TAG_RE.sub(stash, h)
    h = re.sub(r'&(?!\w+;|#\d+;|#x[\da-fA-F]+;)', '&amp;', h)
    h = h.replace('<', '&lt;').replace('>', '&gt;')
    h = h.replace('{', '&#123;').replace('}', '&#125;')
    h = re.sub(r'\x00(\d+)\x00', lambda m: tags[int(m.group(1))], h)
    h = re.sub(r'\x01(\d+)\x01', lambda m: exprs[int(m.group(1))], h)

    h = re.sub(r'\bclass=', 'className=', h)
    h = re.sub(r'\bfor=', 'htmlFor=', h)
    def style(m):
        decls = [d.strip() for d in m.group(1).split(';') if d.strip()]
        parts = []
        for d in decls:
            k, v = d.split(':', 1)
            k = re.sub(r'-([a-z])', lambda mm: mm.group(1).upper(), k.strip())
            parts.append(f'{k}: {v.strip()!r}')
        return 'style={{' + ', '.join(parts) + '}}'
    h = re.sub(r'style="([^"]*)"', style, h)
    # self-close void elements
    h = re.sub(r'<(br|hr|img|input|meta|link|source|wbr|col|embed|track|area|base)\b([^>]*?)\s*/?>',
               lambda m: f'<{m.group(1)}{m.group(2)} />', h)
    # internal .html links -> clean routes
    route = {
        'index': '/', 'features': '/features', 'examples': '/examples',
        'about': '/about', 'guide': '/docs/guide',
        'reference': '/docs/reference', 'api': '/docs/api',
    }
    def link(m):
        return f'href="{route[m.group(1)]}"'
    h = re.sub(r'href="(index|features|guide|examples|reference|api|about)\.html"', link, h)
    # <pre><code class="language-x">{"..."}</code></pre> -> server-highlighted component
    h = re.sub(
        r'<pre>\s*<code className="language-([a-zA-Z0-9]+)">(\{.*?\})</code>\s*</pre>',
        r'<HiCode lang="\1" code=\2 />', h)
    h = re.sub(r'\n{3,}', '\n\n', h)
    return h.strip()

def hicode_import(jsx: str) -> str:
    return 'import HiCode from "@/components/HiCode";\n' if '<HiCode' in jsx else ''

def main_region(src: str) -> str:
    start = src.index('</header>') + len('</header>')
    end = src.index('<!-- Footer Ticker -->')
    return src[start:end]

def title_of(src: str) -> str:
    m = re.search(r'<title>(.*?)</title>', src)
    return html.unescape(m.group(1)).strip() if m else 'Brute'

PAGE_TMPL = '''import type {{ Metadata }} from "next";
{hicode}
export const metadata: Metadata = {{ title: {title!r} }};

export default function Page() {{
  return (
    <>
{jsx}
    </>
  );
}}
'''

def convert_page(src_file: str, out_dir: str):
    src = (SRC / src_file).read_text(encoding='utf-8')
    region = main_region(src)
    # legacy .doc-nav aside is dropped entirely — about is hand-maintained
    region = re.sub(r'<aside class="doc-nav">.*?</aside>', '', region, flags=re.S)
    region = region.replace('<div class="docs-shell">', '<div>')
    region = region.replace('<div class="doc-content">', '<div>')
    jsx = jsxify(region)
    jsx = re.sub(r'^', '      ', jsx, flags=re.M)
    out = ROOT / 'app' / out_dir
    out.mkdir(parents=True, exist_ok=True)
    (out / 'page.tsx').write_text(
        PAGE_TMPL.format(title=title_of(src), hicode=hicode_import(jsx), jsx=jsx),
        encoding='utf-8')
    print(f'{src_file} -> app/{out_dir}/page.tsx')

DOC_TMPL = '''import type {{ Metadata }} from "next";
import DocsShell from "@/components/DocsShell";
{hicode}

export const metadata: Metadata = {{ title: {title!r} }};

const TOC = {toc};

export default function Page() {{
  return (
    <DocsShell toc={{TOC}}>
      <h1 className="doc-title">{heading}</h1>
      <p className="doc-sub">{sub}</p>
{jsx}
    </DocsShell>
  );
}}
'''

def convert_doc(src_file: str, out_dir: str):
    src = (SRC / src_file).read_text(encoding='utf-8')
    region = main_region(src)
    # heading + subtitle from .documentation-header
    h1 = re.search(r'<h1>(.*?)</h1>', region)
    sub = re.search(r'<h1>.*?</h1>\s*<p[^>]*>(.*?)</p>', region, flags=re.S)
    # TOC from .doc-nav anchor list
    nav = re.search(r'<aside class="doc-nav">(.*?)</aside>', region, flags=re.S)
    toc = []
    if nav:
        toc = [{'id': m.group(1), 'title': html.unescape(m.group(2)).strip()}
               for m in re.finditer(r'<a[^>]*href="#([^"]+)"[^>]*>(.*?)</a>', nav.group(1))]
    # content = inside .doc-content
    dc = re.search(r'<div class="doc-content">(.*)</div>\s*</div>\s*</div>\s*</section>', region, flags=re.S)
    if not dc:
        dc = re.search(r'<div class="doc-content">(.*)', region, flags=re.S)
    jsx = jsxify(dc.group(1))
    jsx = re.sub(r'^', '      ', jsx, flags=re.M)
    out = ROOT / 'app' / out_dir
    out.mkdir(parents=True, exist_ok=True)
    heading = jsxify(h1.group(1)) if h1 else 'Docs'
    subtitle = jsxify(sub.group(1).strip()) if sub else ''
    (out / 'page.tsx').write_text(
        DOC_TMPL.format(title=title_of(src), toc=json.dumps(toc, indent=2),
                        heading=heading, sub=subtitle,
                        hicode=hicode_import(jsx), jsx=jsx),
        encoding='utf-8')
    print(f'{src_file} -> app/{out_dir}/page.tsx (toc: {len(toc)} items)')

def convert_examples():
    src = (SRC / "examples.html").read_text(encoding='utf-8')
    region = main_region(src)
    hero_m = re.search(r'<section class="hero">.*?</section>', region, flags=re.S)
    hero = jsxify(hero_m.group(0)) if hero_m else ''
    sections = re.findall(
        r'<section[^>]*class="code-section"[^>]*>.*?</section>', region, flags=re.S)
    parts = [jsxify(s) for s in sections]
    body = (re.sub(r'^', '      ', hero, flags=re.M)
            + '\n      <ExampleTabs>\n'
            + '\n'.join(re.sub(r'^', '        ', p, flags=re.M) for p in parts)
            + '\n      </ExampleTabs>')
    out = ROOT / 'app' / '(marketing)' / 'examples'
    out.mkdir(parents=True, exist_ok=True)
    (out / 'page.tsx').write_text(
        'import type { Metadata } from "next";\n'
        'import ExampleTabs from "@/components/ExampleTabs";\n'
        + hicode_import(body) + '\n'
        'export const metadata: Metadata = { title: "Examples" };\n\n'
        'export default function Page() {\n  return (\n    <>\n'
        + body
        + '\n    </>\n  );\n}\n', encoding='utf-8')
    print(f'examples.html -> app/(marketing)/examples/page.tsx ({len(parts)} sections)')

if __name__ == '__main__':
    for f, d in [('guide.html', 'docs/guide'),
                 ('reference.html', 'docs/reference'),
                 ('api.html', 'docs/api')]:
        convert_doc(f, d)
    # about.html is intentionally skipped — app/(marketing)/about/page.tsx
    # is hand-maintained now.
    for f, d in [('features.html', '(marketing)/features')]:
        convert_page(f, d)
    convert_examples()
