import type { ReactNode } from "react";

export default function Window({
  id,
  title,
  children,
  className = "",
}: {
  id?: string;
  title: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`window ${className}`.trim()} id={id}>
      <div className="win-bar">
        <span className="win-dots"><i /><i /><i /></span>
        <span className="win-title">{title}</span>
        <span className="win-actions"><span /><span /><span /></span>
      </div>
      <div className="win-body">{children}</div>
    </section>
  );
}
