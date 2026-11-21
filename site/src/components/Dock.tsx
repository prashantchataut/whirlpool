import { useEffect, useRef, type ReactNode } from "react";
import { Mark } from "./Mark";

type Glyph = { id: string; label: string; from: string; to: string; running?: boolean; glyph: ReactNode };

const stroke = { fill: "none", stroke: "currentColor", strokeWidth: 1.8, strokeLinecap: "round", strokeLinejoin: "round" } as const;

export const APPS: Glyph[] = [
  {
    id: "files",
    label: "Files",
    from: "#f6d58a",
    to: "#e0a43c",
    running: true,
    glyph: <path {...stroke} d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />,
  },
  {
    id: "browser",
    label: "Browser",
    from: "#6fe0f2",
    to: "#1189b8",
    running: true,
    glyph: (
      <>
        <circle {...stroke} cx="12" cy="12" r="9" />
        <path {...stroke} d="m15.5 8.5-2 5-5 2 2-5z" />
      </>
    ),
  },
  {
    id: "terminal",
    label: "Terminal",
    from: "#244650",
    to: "#07171d",
    running: true,
    glyph: (
      <>
        <polyline {...stroke} points="5 8 10 12 5 16" />
        <line {...stroke} x1="12.5" y1="17" x2="19" y2="17" />
      </>
    ),
  },
  {
    id: "music",
    label: "Music",
    from: "#ff9a7e",
    to: "#e5483a",
    glyph: (
      <>
        <path {...stroke} d="M9 17V6l10-2v11" />
        <circle {...stroke} cx="7" cy="17" r="2.2" />
        <circle {...stroke} cx="17" cy="15" r="2.2" />
      </>
    ),
  },
  {
    id: "mail",
    label: "Mail",
    from: "#9cc7ff",
    to: "#3a6fd8",
    glyph: (
      <>
        <rect {...stroke} x="3" y="6" width="18" height="13" rx="2.4" />
        <polyline {...stroke} points="3.5 7.5 12 13.5 20.5 7.5" />
      </>
    ),
  },
  {
    id: "photos",
    label: "Photos",
    from: "#a6f0cf",
    to: "#16a37f",
    glyph: (
      <>
        <rect {...stroke} x="3" y="5" width="18" height="14" rx="2.4" />
        <circle {...stroke} cx="9" cy="10.5" r="1.6" />
        <path {...stroke} d="m4 17 5-4.5 4 3.5 3-2.5 4 3.5" />
      </>
    ),
  },
  {
    id: "whirlpool",
    label: "Whirlpool Settings",
    from: "#0b5d6b",
    to: "#03222c",
    glyph: null,
  },
];

const TAIL: Glyph[] = [
  {
    id: "downloads",
    label: "Downloads",
    from: "#2d6f86",
    to: "#124457",
    glyph: (
      <>
        <path {...stroke} d="M12 4v11" />
        <path {...stroke} d="m7 11 5 5 5-5" />
        <path {...stroke} d="M5 20h14" />
      </>
    ),
  },
  {
    id: "bin",
    label: "Recycle Bin",
    from: "#7ea6b0",
    to: "#3d6874",
    glyph: (
      <>
        <path {...stroke} d="M5 7h14" />
        <path {...stroke} d="M9 7V4.5h6V7" />
        <path {...stroke} d="m7 7 1 13h8l1-13" />
      </>
    ),
  },
];

function Tile({ app, size }: { app: Glyph; size: number }) {
  return (
    <div
      className="relative grid place-items-center text-white"
      style={{
        width: size,
        height: size,
        borderRadius: size * 0.225,
        background: `linear-gradient(155deg, ${app.from}, ${app.to})`,
        boxShadow: "inset 0 1px 0 rgba(255,255,255,.35), inset 0 0 0 1px rgba(255,255,255,.1), 0 4px 10px -2px rgba(0,10,16,.55)",
      }}
    >
      {app.id === "whirlpool" ? (
        <Mark className="text-foam" stroke="#7aeede" strokeWidth={3.6} />
      ) : (
        <svg viewBox="0 0 24 24" style={{ width: size * 0.54, height: size * 0.54 }}>
          {app.glyph}
        </svg>
      )}
    </div>
  );
}

const BASE = 52;
const PEAK = 0.78; // extra scale at the pointer
const REACH = 96; // px over which magnification falls off

/**
 * `focus` pins the magnification to an icon index (used by the static design renders);
 * leave it out for the interactive dock, which follows the pointer with a critically damped spring.
 */
export function Dock({
  focus,
  scale = 1,
  showTail = true,
  hint = false,
}: {
  focus?: number;
  scale?: number;
  showTail?: boolean;
  hint?: boolean;
}) {
  const items = showTail ? [...APPS, "sep" as const, ...TAIL] : APPS;
  const refs = useRef<(HTMLDivElement | null)[]>([]);
  const wrap = useRef<HTMLDivElement>(null);
  const state = useRef({ x: null as number | null, scales: [] as number[], vel: [] as number[] });

  useEffect(() => {
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let raf = 0;
    let last = performance.now();
    const s = state.current;
    const el = wrap.current;
    if (!el) return;
    const nodes = () => refs.current.filter(Boolean) as HTMLDivElement[];
    // Where each icon sits at rest, measured from the dock's centre. The glass grows
    // symmetrically, so these stay valid while it widens.
    const rest = nodes().map((n) => n.offsetLeft + n.offsetWidth / 2 - el.offsetWidth / 2);
    const padding = 12;

    const target = (i: number, cx: number) => {
      if (focus !== undefined) {
        const d = Math.abs(i - focus);
        return 1 + PEAK * Math.exp(-((d / 1.25) ** 2));
      }
      if (s.x === null) return 1;
      return 1 + PEAK * Math.exp(-(((s.x - cx) / REACH) ** 2));
    };

    const tick = (now: number) => {
      const dt = Math.min(0.032, (now - last) / 1000);
      last = now;
      const els = nodes();
      const box = el.getBoundingClientRect();
      const mid = box.left + box.width / 2;
      let moving = false;
      els.forEach((_, i) => {
        const want = target(i, mid + rest[i]);
        const cur = s.scales[i] ?? 1;
        const v = s.vel[i] ?? 0;
        if (reduce) {
          s.scales[i] = want;
        } else {
          // Critically damped spring: attached to the pointer, no wobble.
          const k = 520;
          const c = 2 * Math.sqrt(k);
          const nv = v + (k * (want - cur) - c * v) * dt;
          s.vel[i] = nv;
          s.scales[i] = cur + nv * dt;
          if (Math.abs(want - s.scales[i]) > 0.002 || Math.abs(nv) > 0.002) moving = true;
        }
      });
      // Neighbours make room: each icon is pushed out by the growth of those between it and the edge.
      const extra = els.map((_, i) => ((s.scales[i] ?? 1) - 1) * BASE);
      const total = extra.reduce((a, b) => a + b, 0);
      let before = 0;
      els.forEach((n, i) => {
        const shift = before + extra[i] / 2 - total / 2;
        before += extra[i];
        const sc = s.scales[i] ?? 1;
        n.style.transform = `translateX(${shift.toFixed(2)}px) translateY(${(-(sc - 1) * 5).toFixed(2)}px) scale(${sc.toFixed(4)})`;
      });
      el.style.paddingInline = `${padding + total / 2}px`;
      if (moving || s.x !== null) raf = requestAnimationFrame(tick);
      else raf = 0;
    };
    const kick = () => {
      if (!raf) {
        last = performance.now();
        raf = requestAnimationFrame(tick);
      }
    };
    const move = (e: PointerEvent) => {
      s.x = e.clientX;
      kick();
    };
    const leave = () => {
      s.x = null;
      kick();
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerleave", leave);
    kick();
    return () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerleave", leave);
      cancelAnimationFrame(raf);
    };
  }, [focus]);

  return (
    <div style={{ transform: `scale(${scale})`, transformOrigin: "bottom center" }} className="inline-block">
      <div ref={wrap} className="glass-dock relative inline-flex items-end gap-[10px] rounded-[26px] pb-[10px] pt-[10px]" style={{ paddingInline: 12 }} data-dock>
        {items.map((app, i) =>
          app === "sep" ? (
            <div key="sep" className="mx-[2px] mb-[6px] h-[40px] w-px self-end bg-[rgba(150,230,236,.28)]" />
          ) : (
            <div
              key={app.id}
              ref={(n) => {
                refs.current[i] = n;
              }}
              className="group relative origin-bottom will-change-transform"
              style={{ width: BASE, height: BASE }}
              title={app.label}
            >
              <Tile app={app} size={BASE} />
              {app.running && (
                <span className="absolute -bottom-[7px] left-1/2 h-[4px] w-[4px] -translate-x-1/2 rounded-full bg-foam shadow-[0_0_6px_rgba(122,238,222,.9)]" />
              )}
            </div>
          ),
        )}
      </div>
      {hint && (
        <p className="mt-3 text-center font-mono text-[11px] tracking-wide text-white/55">move your pointer across the dock</p>
      )}
    </div>
  );
}
