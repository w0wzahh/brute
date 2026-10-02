import Prism from "prismjs";
import "prismjs/components/prism-rust";
import "prismjs/components/prism-bash";

/**
 * Server-rendered syntax highlighting — Prism.highlight runs at render time
 * and the resulting HTML is deterministic, so there's no DOM mutation after
 * hydration (unlike Prism.highlightAll, which caused hydration mismatches).
 */
export default function HiCode({
  lang,
  code,
}: {
  lang: string;
  code: string;
}) {
  const grammar = Prism.languages[lang] ?? Prism.languages.plain;
  const highlighted = Prism.highlight(code, grammar, lang);
  return (
    <pre className={`language-${lang}`}>
      <code
        className={`language-${lang}`}
        dangerouslySetInnerHTML={{ __html: highlighted }}
      />
    </pre>
  );
}
