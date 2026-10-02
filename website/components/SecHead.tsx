import type { ReactNode } from "react";

export default function SecHead({
  no,
  children,
}: {
  no: string;
  children: ReactNode;
}) {
  return (
    <div className="sec-head">
      <span className="sec-no">{no}</span>
      <h2 style={{ margin: 0 }}>{children}</h2>
    </div>
  );
}
