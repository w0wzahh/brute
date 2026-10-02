export default function Ticker({
  items,
  band,
  reverse = false,
}: {
  items: string[];
  band?: "dark" | "pink" | "yellow";
  reverse?: boolean;
}) {
  const cls = `ticker${band === "pink" ? " pink-band" : ""}${band === "yellow" ? " yellow-band" : ""}`;
  const row = items.map((t, i) => <span key={i}>{t}</span>);
  return (
    <div className={cls} aria-hidden="true">
      <div className={`ticker-track${reverse ? " ticker-rev" : ""}`}>
        {row}
        {row}
      </div>
    </div>
  );
}
