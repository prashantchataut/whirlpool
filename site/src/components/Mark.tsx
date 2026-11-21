import { useMemo } from "react";

/** Same geometry as `ui::vortex` in the Rust settings app: two arms winding into a point. */
function arm(radius: number, offset: number, steps = 64, turns = 1.55) {
  const pts: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const r = radius * Math.pow(1 - t, 0.85);
    const a = offset + t * turns * Math.PI * 2;
    pts.push(`${(50 + Math.cos(a) * r).toFixed(2)} ${(50 + Math.sin(a) * r).toFixed(2)}`);
  }
  return "M" + pts.join(" L");
}

export function Mark({
  className,
  stroke = "currentColor",
  strokeWidth = 3,
}: {
  className?: string;
  stroke?: string;
  strokeWidth?: number;
}) {
  const [a, b] = useMemo(() => [arm(44, 0), arm(44, Math.PI)], []);
  return (
    <svg viewBox="0 0 100 100" className={className} fill="none" aria-hidden="true">
      <path d={a} stroke={stroke} strokeWidth={strokeWidth} strokeLinecap="round" />
      <path d={b} stroke={stroke} strokeWidth={strokeWidth * 0.66} strokeLinecap="round" />
      <circle cx="50" cy="50" r="3.2" fill={stroke} />
    </svg>
  );
}

/** The app icon as shipped: abyss squircle, foam vortex. */
export function AppIcon({ size = 40 }: { size?: number }) {
  return (
    <span
      className="inline-grid place-items-center rounded-[28%] text-foam"
      style={{
        width: size,
        height: size,
        background: "radial-gradient(120% 120% at 20% 10%, #0b5d6b, #03222c 70%)",
        boxShadow: "inset 0 0 0 1px rgba(122,238,222,.28), inset 0 1px 0 rgba(255,255,255,.18)",
      }}
    >
      <Mark className="h-[72%] w-[72%]" strokeWidth={3.4} />
    </span>
  );
}
