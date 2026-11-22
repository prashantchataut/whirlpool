import { useEffect, useRef, useState } from "react";
import { Desktop, DEFAULT_BAR, DEFAULT_DOCK } from "./Desktop";
import { CHARGE_STYLES, WALLS, levelColor, type ChargeStyle } from "../lib/theme";
import { Play } from "./Icons";

export function Charging() {
  const [style, setStyle] = useState<ChargeStyle>("edge");
  const [percent, setPercent] = useState(64);
  const [key, setKey] = useState(0);
  const frame = useRef<HTMLDivElement>(null);
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const el = frame.current;
    if (!el) return;
    const io = new IntersectionObserver(([e]) => setInView(e.isIntersecting), { threshold: 0.35 });
    io.observe(el);
    return () => io.disconnect();
  }, []);

  // Replay on a calm loop while the preview is on screen.
  useEffect(() => {
    if (!inView) return;
    const id = window.setInterval(() => setKey((k) => k + 1), 6200);
    return () => window.clearInterval(id);
  }, [inView]);

  const color = levelColor(percent);
  const current = CHARGE_STYLES.find((c) => c.id === style) ?? CHARGE_STYLES[0];

  return (
    <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,380px)_minmax(0,1fr)]">
      <div>
        <ul className="grid gap-1.5" aria-label="Charging animations">
          {CHARGE_STYLES.map((c, i) => (
            <li key={c.id}>
              <button
                onClick={() => {
                  setStyle(c.id);
                  setKey((k) => k + 1);
                }}
                aria-pressed={style === c.id}
                className={`group flex w-full items-baseline gap-4 rounded-xl border px-4 py-3 text-left transition-colors ${
                  style === c.id
                    ? "glass border-teal/40"
                    : "border-transparent hover:border-white/10 hover:bg-white/[0.03]"
                }`}
              >
                <span className="font-mono text-[11px] text-faint">{String(i + 1).padStart(2, "0")}</span>
                <span className="min-w-0">
                  <span className="flex items-center gap-2 font-display text-lg font-bold tracking-tight">
                    {c.name}
                    {c.isNew && (
                      <span className="rounded bg-teal px-1.5 py-px font-mono text-[9px] font-medium tracking-wider text-ink">
                        NEW
                      </span>
                    )}
                  </span>
                  {style === c.id && <span className="mt-1 block text-[13.5px] leading-relaxed text-mute">{c.blurb}</span>}
                </span>
              </button>
            </li>
          ))}
        </ul>

        <div className="glass mt-5 rounded-xl p-4">
          <div className="mb-3 flex items-center justify-between">
            <label htmlFor="battery" className="text-[13px] font-medium">
              Battery level
            </label>
            <span className="flex items-center gap-2 font-mono text-[12px] tabular-nums">
              <span className="h-2.5 w-2.5 rounded-full" style={{ background: color, boxShadow: `0 0 10px ${color}` }} />
              {percent}%
            </span>
          </div>
          <input
            id="battery"
            type="range"
            min={5}
            max={100}
            value={percent}
            onChange={(e) => {
              setPercent(Number(e.target.value));
              setKey((k) => k + 1);
            }}
            className="wp-range w-full"
            style={{ ["--fill" as string]: `${((percent - 5) / 95) * 100}%` }}
          />
          <div className="mt-2 flex justify-between font-mono text-[10.5px] text-faint">
            <span>orange ≤ 20</span>
            <span>yellow 21–59</span>
            <span>green ≥ 60</span>
          </div>
          <button
            onClick={() => setKey((k) => k + 1)}
            className="mt-4 inline-flex items-center gap-2 rounded-lg bg-white/10 px-3.5 py-2 text-[13px] font-medium transition-colors hover:bg-white/15"
          >
            <Play className="h-3.5 w-3.5" /> Replay
          </button>
        </div>
      </div>

      <div ref={frame}>
        <Desktop
          wall={WALLS[1]}
          bar={{ ...DEFAULT_BAR, tint: "graphite", opacity: 60 }}
          dock={DEFAULT_DOCK}
          percent={percent}
          charging
          fx={style}
          fxKey={key}
          className="shadow-[0_50px_100px_-40px_rgba(0,0,0,0.95)]"
        />
        <p className="mt-3 font-mono text-[11px] leading-relaxed text-faint">
          {current.name}: motion preview drawn by this page with the app's timings and colour thresholds. In the app the same
          scene is drawn by the Windows compositor.
        </p>
      </div>
    </div>
  );
}
