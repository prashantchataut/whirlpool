import type { ReactNode } from "react";
import { Dock } from "./Dock";
import { Mark } from "./Mark";

/* ------------------------------------------------------------------ Menu bar */

const I = { fill: "none", stroke: "currentColor", strokeWidth: 1.7, strokeLinecap: "round", strokeLinejoin: "round" } as const;

function Status() {
  return (
    <div className="flex items-center gap-3.5 text-white/90">
      <span className="font-mono text-[10px] tracking-wide text-white/70">EN</span>
      <svg viewBox="0 0 24 24" className="h-[14px] w-[14px]">
        <path {...I} d="M2 9a15 15 0 0 1 20 0M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0" />
        <circle cx="12" cy="19.2" r="1" fill="currentColor" />
      </svg>
      <svg viewBox="0 0 24 24" className="h-[14px] w-[14px]">
        <path {...I} d="M4 10v4h3.5L12 18V6L7.5 10zM16 9a4 4 0 0 1 0 6" />
      </svg>
      <span className="flex items-center gap-1.5">
        <svg viewBox="0 0 26 14" className="h-[11px] w-[20px]">
          <rect x="1" y="1.5" width="21" height="11" rx="3" fill="none" stroke="currentColor" strokeWidth="1.4" />
          <rect x="3" y="3.5" width="14" height="7" rx="1.6" fill="#7aeede" />
          <rect x="23" y="5" width="2" height="4" rx="1" fill="currentColor" />
        </svg>
        <span className="figures text-[11px]">82%</span>
      </span>
      <span className="figures text-[11.5px] font-medium">Tue 14:08</span>
    </div>
  );
}

function Island({ style }: { style: "notch" | "bubble" }) {
  const body = (
    <div className="flex items-center gap-2.5 text-[11px] text-white">
      <span className="relative grid h-[14px] w-[14px] place-items-center">
        <span className="absolute inset-0 rounded-full bg-foam/25" />
        <span className="h-[6px] w-[6px] rounded-full bg-foam" />
      </span>
      <span className="text-white/65">Claude</span>
      <span className="figures font-medium">38%</span>
      <span className="h-3 w-px bg-white/20" />
      <span className="text-white/65">Codex</span>
      <span className="figures font-medium">12%</span>
    </div>
  );
  return style === "notch" ? (
    <div className="absolute left-1/2 top-0 -translate-x-1/2 rounded-b-[16px] bg-[#020b0f] px-5 pb-[7px] pt-[5px]">{body}</div>
  ) : (
    <div className="absolute right-[190px] top-[5px] rounded-full bg-[#020b0f] px-3.5 py-[4px]">{body}</div>
  );
}

export function MenuBar({ island = "notch" }: { island?: "notch" | "bubble" }) {
  return (
    <div className="glass-bar relative z-10 flex h-[30px] items-center justify-between px-4 text-white">
      <div className="flex items-center gap-4 text-[12px]">
        <Mark className="h-[15px] w-[15px] text-foam" stroke="#7aeede" strokeWidth={5} />
        <span className="font-semibold">Terminal</span>
        <span className="text-white/70">Shell</span>
        <span className="text-white/70">Edit</span>
        <span className="text-white/70">View</span>
      </div>
      <Island style={island} />
      <Status />
    </div>
  );
}

/* ------------------------------------------------------------------ Desktop */

function MiniWindow({ title, tone, active }: { title: string; tone: "term" | "web"; active?: boolean }) {
  return (
    <div className={`w-[150px] shrink-0 rounded-[10px] p-1.5 ${active ? "bg-white/14 ring-1 ring-foam/60" : "bg-white/6"}`}>
      <div className={`h-[78px] overflow-hidden rounded-[6px] ${tone === "term" ? "bg-[#06171d]" : "bg-[#e9f3f4]"}`}>
        <div className={`h-[9px] ${tone === "term" ? "bg-[#12303a]" : "bg-[#cfe1e4]"}`} />
        {tone === "term" ? (
          <div className="space-y-[5px] p-2 font-mono text-[6px] leading-none text-foam/80">
            <div>$ cargo build --release</div>
            <div className="text-white/50">Compiling whirlpool-core</div>
            <div className="text-white/50">Compiling whirlpool-shell</div>
            <div className="text-white/50">Compiling whirlpool</div>
            <div>Finished in 41.2s</div>
          </div>
        ) : (
          <div className="space-y-[5px] p-2">
            <div className="h-[6px] w-3/4 rounded bg-[#0b3a48]/70" />
            <div className="h-[4px] w-full rounded bg-[#0b3a48]/25" />
            <div className="h-[4px] w-5/6 rounded bg-[#0b3a48]/25" />
            <div className="h-[22px] w-full rounded bg-[#0ea5a8]/30" />
          </div>
        )}
      </div>
      <div className="mt-1.5 truncate px-0.5 text-center text-[9.5px] text-white/85">{title}</div>
    </div>
  );
}

/** What hovering an app with several windows shows: live, titled previews. */
export function WindowPreviews({ className = "" }: { className?: string }) {
  return (
    <div className={`glass-dock rounded-[18px] p-2.5 ${className}`}>
      <div className="flex gap-2">
        <MiniWindow title="whirlpool — design notes" tone="web" active />
        <MiniWindow title="cargo build" tone="term" />
      </div>
    </div>
  );
}

export function Desktop({
  children,
  island = "notch",
  className = "",
  bar = true,
}: {
  children?: ReactNode;
  island?: "notch" | "bubble";
  className?: string;
  bar?: boolean;
}) {
  return (
    <div className={`wallpaper relative overflow-hidden ${className}`}>
      {/* Ripples: concentric arcs, drawn once, very low contrast. */}
      <svg className="pointer-events-none absolute -right-[18%] -top-[30%] h-[150%] w-[75%] opacity-[.16]" viewBox="0 0 200 200" fill="none">
        {[18, 30, 44, 60, 78, 98].map((r, i) => (
          <circle key={r} cx="100" cy="100" r={r} stroke="#c9fbf3" strokeWidth={0.5 + i * 0.05} strokeDasharray={`${60 + i * 24} ${20 + i * 9}`} />
        ))}
      </svg>
      {bar && <MenuBar island={island} />}
      {children}
    </div>
  );
}

/* ------------------------------------------------------------------ Settings window */

const P = {
  window: "#071419",
  sidebar: "#0a1b22",
  card: "#0e252d",
  border: "#1b3a45",
  text: "#e6f4f2",
  muted: "#84a8ad",
  track: "#2a4b56",
  accent: "#0ea5a8",
  foam: "#7aeede",
};

function Switch({ on }: { on: boolean }) {
  return (
    <span className="relative inline-block h-6 w-[42px] rounded-full" style={{ background: on ? P.accent : P.track }}>
      <span className="absolute top-[3px] h-[18px] w-[18px] rounded-full bg-white" style={{ left: on ? 21 : 3 }} />
    </span>
  );
}

function Row({ label, hint, children, last }: { label: string; hint?: string; children: ReactNode; last?: boolean }) {
  return (
    <div className="flex items-center justify-between gap-4 py-[9px]" style={{ borderBottom: last ? "none" : `1px solid ${P.border}` }}>
      <div className="min-w-0">
        <div className="text-[13.5px]" style={{ color: P.text }}>
          {label}
        </div>
        {hint && (
          <div className="mt-px text-[11.5px] leading-snug" style={{ color: P.muted }}>
            {hint}
          </div>
        )}
      </div>
      {children}
    </div>
  );
}

function Slider({ value }: { value: number }) {
  return (
    <span className="relative inline-block h-[18px] w-[190px]">
      <span className="absolute left-0 right-0 top-[7px] h-[4px] rounded-full" style={{ background: P.track }} />
      <span className="absolute left-0 top-[7px] h-[4px] rounded-full" style={{ background: P.accent, width: `${value}%` }} />
      <span
        className="absolute top-0 h-[18px] w-[18px] rounded-full bg-white shadow"
        style={{ left: `calc(${value}% - 9px)`, boxShadow: "0 0 0 1px rgba(0,0,0,.2)" }}
      />
    </span>
  );
}

function PositionChoice({ label, at, selected }: { label: string; at: "left" | "bottom" | "right"; selected?: boolean }) {
  return (
    <div className="w-[118px]">
      <div
        className="relative h-[66px] overflow-hidden rounded-[10px]"
        style={{
          background: "linear-gradient(165deg,#1f8fa0,#0c3b55 70%)",
          boxShadow: `0 0 0 ${selected ? 2.5 : 1}px ${selected ? P.accent : P.border} inset`,
        }}
      >
        <div
          className="absolute rounded-[5px] bg-white/30"
          style={
            at === "bottom"
              ? { left: "22%", right: "22%", bottom: 6, height: 15 }
              : at === "left"
                ? { left: 6, top: "18%", bottom: "18%", width: 15 }
                : { right: 6, top: "18%", bottom: "18%", width: 15 }
          }
        />
      </div>
      <div className="mt-1.5 text-center text-[12px]" style={{ color: selected ? P.text : P.muted }}>
        {label}
      </div>
    </div>
  );
}

export function SettingsWindow({ className = "" }: { className?: string }) {
  const nav = ["Dock", "Windows", "Menu bar", "System"];
  return (
    <div
      className={`overflow-hidden rounded-[10px] text-left ${className}`}
      style={{ background: P.window, boxShadow: "0 40px 80px -24px rgba(0,8,12,.9), 0 0 0 1px rgba(122,238,222,.14)" }}
    >
      {/* Native title bar */}
      <div className="flex h-[32px] items-center justify-between pl-3 text-[12px]" style={{ background: "#061015", color: P.muted }}>
        <span>Whirlpool</span>
        <span className="flex h-full">
          {["—", "▢", "✕"].map((g) => (
            <span key={g} className="grid w-[46px] place-items-center text-[11px]">
              {g}
            </span>
          ))}
        </span>
      </div>
      <div className="flex" style={{ height: 470 }}>
        <aside className="flex w-[190px] shrink-0 flex-col px-3 py-4" style={{ background: P.sidebar }}>
          <div className="mb-4 flex items-center gap-2">
            <span
              className="grid h-8 w-8 place-items-center rounded-[10px]"
              style={{ background: "#04222c", boxShadow: `inset 0 0 0 1px ${P.accent}b3` }}
            >
              <Mark className="h-[26px] w-[26px]" stroke={P.foam} strokeWidth={5.5} />
            </span>
            <span className="text-[18px] font-semibold" style={{ color: P.text }}>
              Whirlpool
            </span>
          </div>
          {nav.map((n, i) => (
            <div
              key={n}
              className="relative mb-0.5 flex h-[36px] items-center rounded-[9px] pl-3.5 text-[14px]"
              style={{ background: i === 0 ? `${P.accent}33` : "transparent", color: i === 0 ? P.foam : P.text }}
            >
              {i === 0 && <span className="absolute left-0 h-[18px] w-[3px] rounded-full" style={{ background: P.foam }} />}
              {n}
              {n === "System" && <span className="absolute right-3.5 h-[9px] w-[9px] rounded-full" style={{ background: "#f2644a" }} />}
            </div>
          ))}
          <div className="mt-auto text-[11px]" style={{ color: P.muted }}>
            v0.1.0 · native Rust
          </div>
        </aside>
        <main className="min-w-0 flex-1 overflow-hidden px-6 py-4">
          <div className="text-[24px] font-semibold leading-none" style={{ color: P.text }}>
            Dock
          </div>
          <svg viewBox="0 0 120 10" className="mb-3 mt-2 h-[10px] w-[120px]">
            <defs>
              <linearGradient id="wl" x1="0" x2="1">
                <stop offset="0" stopColor={P.accent} />
                <stop offset="1" stopColor={P.accent} stopOpacity=".12" />
              </linearGradient>
            </defs>
            <path d={Array.from({ length: 61 }, (_, i) => `${i ? "L" : "M"}${i * 2} ${5 + Math.sin((i / 60) * Math.PI * 6) * 2.2}`).join(" ")} stroke="url(#wl)" strokeWidth="1.6" fill="none" />
          </svg>
          <div className="mb-1 text-[11px] font-semibold tracking-[.06em]" style={{ color: P.muted }}>
            SIZE
          </div>
          <div className="rounded-[12px] px-4" style={{ background: P.card, boxShadow: `inset 0 0 0 1px ${P.border}` }}>
            <Row label="Icon size" hint="Small to large.">
              <Slider value={46} />
            </Row>
            <Row label="Magnification" hint="Icons grow as the pointer passes over them.">
              <Switch on />
            </Row>
            <Row label="Magnified size" last>
              <Slider value={72} />
            </Row>
          </div>
          <div className="mb-1 mt-4 text-[11px] font-semibold tracking-[.06em]" style={{ color: P.muted }}>
            POSITION ON SCREEN
          </div>
          <div className="rounded-[12px] px-4 pb-2.5 pt-3" style={{ background: P.card, boxShadow: `inset 0 0 0 1px ${P.border}` }}>
            <div className="flex gap-3.5">
              <PositionChoice label="Left" at="left" />
              <PositionChoice label="Bottom" at="bottom" selected />
              <PositionChoice label="Right" at="right" />
            </div>
          </div>
          <div className="mt-3 rounded-[12px] px-4" style={{ background: P.card, boxShadow: `inset 0 0 0 1px ${P.border}` }}>
            <Row label="Automatically hide" hint="Rest the pointer at the Dock's edge to reveal it." last>
              <Switch on={false} />
            </Row>
          </div>
        </main>
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ Composed screens */

export function DockScreen({ className = "" }: { className?: string }) {
  return (
    <Desktop className={`aspect-[16/10] ${className}`}>
      <div className="absolute bottom-[132px] left-[calc(50%-150px)] z-10 -translate-x-1/2">
        <WindowPreviews />
        <div className="mx-auto h-2 w-2 rotate-45 bg-[rgba(4,26,36,.6)]" style={{ marginTop: -4 }} />
      </div>
      <div className="absolute bottom-3 left-1/2 z-10 -translate-x-1/2">
        <Dock focus={1} showTail />
      </div>
    </Desktop>
  );
}
