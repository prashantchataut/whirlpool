import { useState, type ReactNode } from "react";
import { Dock } from "./Dock";
import { AppIcon, Mark } from "./Mark";
import { Desktop, DockScreen, SettingsWindow } from "./Screens";
import { INSTALL_LINE, LATEST_EXE, REPO_URL, formatSize, useLatestRelease, useScrollProgress } from "../lib/hooks";

/* ------------------------------------------------------------------ chrome */

export function Header() {
  const links = [
    ["Screens", "#screens"],
    ["Behavior", "#behavior"],
    ["Release", "#release"],
    ["Specs", "#specs"],
  ];
  return (
    <header className="fixed left-1/2 top-4 z-50 w-[min(94vw,760px)] -translate-x-1/2">
      <nav className="flex items-center justify-between rounded-full bg-abyss/88 backdrop-blur-xl py-2 pl-2.5 pr-2 shadow-[0_10px_30px_-10px_rgba(0,10,16,.6)] ring-1 ring-white/10">
        <a href="#top" className="flex items-center gap-2.5 pr-3 text-[15px] font-medium text-white">
          <AppIcon size={30} />
          Whirlpool
        </a>
        <div className="hidden items-center gap-1 md:flex">
          {links.map(([l, h]) => (
            <a key={h} href={h} className="rounded-full px-3.5 py-1.5 text-[13.5px] text-white/70 transition-colors hover:bg-white/10 hover:text-white">
              {l}
            </a>
          ))}
        </div>
        <a
          href={LATEST_EXE}
          className="rounded-full bg-foam px-4 py-2 text-[13.5px] font-semibold text-abyss transition-transform hover:-translate-y-px active:translate-y-0"
        >
          Download
        </a>
      </nav>
    </header>
  );
}

/** Depth replaces the scrollbar: how far down the water column you are. */
export function DepthGauge() {
  const { progress } = useScrollProgress();
  const m = Math.round(progress * 42);
  return (
    <div className="pointer-events-none fixed right-5 top-1/2 z-40 hidden -translate-y-1/2 lg:block" aria-hidden="true">
      <div className="flex flex-col items-center gap-2 rounded-full bg-abyss/85 backdrop-blur-xl px-2.5 py-3 ring-1 ring-white/10">
        <div className="relative h-[200px] w-3">
          {Array.from({ length: 15 }, (_, i) => (
            <span key={i} className="absolute left-1/2 h-px -translate-x-1/2 bg-white/35" style={{ top: `${(i / 14) * 100}%`, width: i % 7 === 0 ? 12 : 6 }} />
          ))}
          <span
            className="absolute left-1/2 h-[9px] w-[9px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-foam shadow-[0_0_10px_rgba(122,238,222,.9)]"
            style={{ top: `${progress * 100}%` }}
          />
        </div>
        <span className="figures font-mono text-[10px] text-white/80">{m} m</span>
      </div>
    </div>
  );
}

/** One big slow vortex behind the dark sections; it turns with the scroll. */
export function BackdropVortex() {
  const { y } = useScrollProgress();
  return (
    <div className="pointer-events-none sticky top-0 z-0 h-0" aria-hidden="true">
      <div
        className="absolute -right-[38vh] -top-[6vh] h-[130vh] w-[130vh] text-foam opacity-[0.055]"
        style={{ transform: `rotate(${y * 0.045}deg)` }}
      >
        <Mark className="h-full w-full" strokeWidth={0.9} />
      </div>
    </div>
  );
}

function Eyebrow({ n, label, depth }: { n: string; label: string; depth: string }) {
  return (
    <div className="mb-6 flex items-center gap-3 font-mono text-[11.5px] uppercase tracking-[.16em] text-foam/80">
      <span className="figures">{n}</span>
      <span className="h-px w-8 bg-foam/40" />
      <span>{label}</span>
      <span className="figures text-white/40">· {depth}</span>
    </div>
  );
}

/* ------------------------------------------------------------------ hero */

export function Hero() {
  return (
    <section id="top" className="relative overflow-hidden bg-shallows text-abyss">
      {/* Caustics: light through moving water, faked with two blurred conic washes. */}
      <div
        className="pointer-events-none absolute inset-0 opacity-80"
        style={{
          background:
            "radial-gradient(60% 50% at 78% 8%, rgba(14,165,168,.22), transparent 70%), radial-gradient(50% 40% at 10% 30%, rgba(122,238,222,.35), transparent 70%)",
        }}
      />
      <div className="pointer-events-none absolute -right-[12vw] top-[4vh] h-[78vw] max-h-[900px] w-[78vw] max-w-[900px] opacity-[0.07] spin-slow" aria-hidden="true">
        <Mark className="h-full w-full text-water" strokeWidth={0.8} />
      </div>

      <div className="relative mx-auto max-w-[1280px] px-6 pb-16 pt-36 md:px-10 md:pt-44">
        <p className="mb-7 font-mono text-[12px] uppercase tracking-[.16em] text-water/80">
          Dock &amp; menu bar for Windows 10 / 11 <span className="mx-2 opacity-40">/</span> 100% Rust
        </p>
        <h1 className="font-display text-[clamp(5.2rem,17vw,15rem)] leading-[0.82] tracking-[-0.035em] text-abyss">
          Whirl<em className="pr-[0.04em] text-current/100 italic text-[#0b6f78]">pool</em>
        </h1>
        <div className="mt-16 grid gap-10 md:mt-20 md:grid-cols-12">
          <p className="max-w-[34ch] text-[clamp(1.25rem,2.1vw,1.7rem)] leading-[1.28] text-water md:col-span-6">
            The dock Windows never shipped. Smooth magnification, live window previews, and a menu bar that earns its pixels, in two small executables.
          </p>
          <div className="flex flex-wrap items-start gap-3 md:col-span-6 md:justify-end md:pt-1">
            <a
              href={LATEST_EXE}
              className="group inline-flex items-center gap-3 rounded-full bg-abyss px-6 py-4 text-[15px] font-semibold text-shallows shadow-[0_18px_30px_-14px_rgba(3,20,27,.8)] transition-transform hover:-translate-y-0.5"
            >
              Download whirlpool.exe
              <span className="grid h-6 w-6 place-items-center rounded-full bg-foam text-abyss transition-transform group-hover:translate-y-0.5">↓</span>
            </a>
            <a href={REPO_URL} className="inline-flex items-center rounded-full px-6 py-4 text-[15px] font-medium text-abyss ring-1 ring-abyss/25 transition-colors hover:bg-abyss/5">
              Read the source
            </a>
          </div>
        </div>

        {/* A real, interactive dock on the page. */}
        <div className="relative mt-16 md:mt-20">
          <Desktop className="aspect-[16/10] rounded-[20px] shadow-[0_50px_90px_-30px_rgba(3,20,27,.75)] ring-1 ring-abyss/20 md:aspect-[16/8.6]">
            <div className="absolute inset-x-0 bottom-4 z-10 flex justify-center">
              <div className="max-w-full origin-bottom scale-[.62] sm:scale-[.8] md:scale-100">
                <Dock hint />
              </div>
            </div>
          </Desktop>
          <p className="mt-4 font-mono text-[11.5px] text-water/70">Fig. 0 — this dock is live. Same spring, same falloff curve as the native one.</p>
        </div>
      </div>
      <svg viewBox="0 0 1440 120" preserveAspectRatio="none" className="relative block h-[70px] w-full md:h-[110px]" aria-hidden="true">
        <path d="M0 62C150 14 300 108 480 64S820 8 1020 58 1330 104 1440 44V120H0Z" fill="#0a4656" />
      </svg>
    </section>
  );
}

/* ------------------------------------------------------------------ screens */

function Figure({ n, caption, children, className = "" }: { n: string; caption: string; children: ReactNode; className?: string }) {
  return (
    <figure className={`reveal ${className}`}>
      {children}
      <figcaption className="mt-4 flex flex-wrap items-baseline gap-x-3 font-mono text-[11.5px] text-mist">
        <span className="text-foam">Fig. {n}</span>
        <span>{caption}</span>
        <span className="rounded-full border hairline px-2 py-0.5 text-[10px] uppercase tracking-wider text-white/45">design render</span>
      </figcaption>
    </figure>
  );
}

export function Screens() {
  return (
    <section id="screens" className="relative z-10 mx-auto max-w-[1280px] px-6 pb-28 pt-16 md:px-10 md:pt-24">
      <Eyebrow n="01" label="Screens" depth="8 m" />
      <h2 className="reveal max-w-[16ch] font-display text-[clamp(2.8rem,6.4vw,5.6rem)] leading-[0.95] tracking-[-0.02em]">
        Three surfaces, <em className="text-foam">one tide.</em>
      </h2>
      <p className="reveal mt-6 max-w-[52ch] text-[17px] leading-relaxed text-mist">
        The dock, the menu bar and the settings window share one palette, one set of springs and one rule: nothing moves unless you did.
      </p>

      <div className="mt-14 space-y-20 md:mt-20">
        <Figure n="1" caption="Dock — hover an app with several windows for titled, live previews.">
          <DockScreen className="rounded-[18px] ring-1 ring-white/10" />
        </Figure>

        <div className="grid items-center gap-10 lg:grid-cols-12">
          <Figure n="2" caption="Settings — native egui window, edits apply to the running dock live." className="lg:col-span-8">
            <SettingsWindow />
          </Figure>
          <div className="reveal lg:col-span-4">
            <h3 className="font-display text-[2.2rem] leading-none">No web view.</h3>
            <p className="mt-4 text-[15.5px] leading-relaxed text-mist">
              Settings is a plain Rust window. It writes <code className="font-mono text-[13px] text-foam">config.json</code> and pings the dock to reload, so there is no second runtime idling behind a tray icon.
            </p>
            <ul className="mt-6 space-y-3 border-t hairline pt-5 font-mono text-[12.5px] text-white/75">
              <li className="flex justify-between"><span>Light + dark</span><span className="text-mist">follows Windows</span></li>
              <li className="flex justify-between"><span>Languages</span><span className="text-mist">EN · ES · FR</span></li>
              <li className="flex justify-between"><span>Previews</span><span className="text-mist">over your wallpaper</span></li>
            </ul>
          </div>
        </div>

        <Figure n="3" caption="Menu bar — the island holds AI plan usage or now-playing. Notch and bubble styles.">
          <div className="-mx-6 overflow-x-auto px-6 md:mx-0 md:px-0">
            <div className="grid min-w-[760px] gap-4">
              <Desktop island="notch" className="h-[84px] rounded-[16px] ring-1 ring-white/10" />
              <Desktop island="bubble" className="h-[84px] rounded-[16px] ring-1 ring-white/10" />
            </div>
          </div>
        </Figure>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ behavior */

const BEHAVIOR: { t: string; d: string; tag: string }[] = [
  { t: "Magnification that follows the pointer", d: "Icons swell under the cursor on a damped spring. No overshoot, no wobble, same curve at 60 and 240 Hz.", tag: "spring" },
  { t: "Genie minimize", d: "Windows fold into their dock icon in three speeds. Or keep minimized windows as live thumbnails on the right.", tag: "3 speeds" },
  { t: "Live window previews", d: "Hover an app with several windows for titled previews. Scroll for more, click to switch, Esc to dismiss.", tag: "per window" },
  { t: "Groups, pins and drag-and-drop", d: "Drop one app onto another to group it; click to fan it out. Drop files on apps, Downloads or the Recycle Bin.", tag: "reorderable" },
  { t: "A menu bar with an island", d: "Clock, network, volume, battery, input language and tray, plus plan usage for your AI tools and what’s playing.", tag: "optional" },
  { t: "Auto-hide on any edge", d: "Left, bottom or right. Rest the pointer at the edge on any display and it returns.", tag: "multi-monitor" },
  { t: "A watchdog for the taskbar", d: "If the dock ever exits unexpectedly, a guard process restores the Windows taskbar. whirlpool.exe --restore does it by hand.", tag: "fail-safe" },
];

export function Behavior() {
  return (
    <section id="behavior" className="relative z-10 border-t hairline">
      <div className="mx-auto grid max-w-[1280px] gap-12 px-6 py-24 md:px-10 lg:grid-cols-12 lg:py-32">
        <div className="lg:col-span-5">
          <div className="lg:sticky lg:top-32">
            <Eyebrow n="02" label="Behavior" depth="20 m" />
            <h2 className="reveal font-display text-[clamp(2.6rem,5.4vw,4.8rem)] leading-[0.96] tracking-[-0.02em]">
              Calm on the surface. <em className="text-foam">Busy underneath.</em>
            </h2>
            <p className="reveal mt-6 max-w-[40ch] text-[16.5px] leading-relaxed text-mist">
              Everything animates on the compositor, and nothing runs while it is not needed. Idle is the default state, not an optimisation.
            </p>
          </div>
        </div>
        <ol className="lg:col-span-7">
          {BEHAVIOR.map((b, i) => (
            <li key={b.t} className="reveal group grid grid-cols-[3.2rem_1fr_auto] items-baseline gap-x-4 border-t hairline py-7 first:border-t-0 first:pt-0">
              <span className="figures font-mono text-[12px] text-foam/70">{String(i + 1).padStart(2, "0")}</span>
              <div>
                <h3 className="text-[20px] font-medium leading-snug text-white">{b.t}</h3>
                <p className="mt-2 max-w-[56ch] text-[15px] leading-relaxed text-mist">{b.d}</p>
              </div>
              <span className="hidden whitespace-nowrap rounded-full border hairline px-2.5 py-1 font-mono text-[10.5px] uppercase tracking-wider text-white/50 sm:block">{b.tag}</span>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ release */

function Copy({ text }: { text: string }) {
  const [done, setDone] = useState(false);
  return (
    <button
      onClick={() => {
        navigator.clipboard?.writeText(text);
        setDone(true);
        setTimeout(() => setDone(false), 1600);
      }}
      className="shrink-0 rounded-full bg-white/10 px-3.5 py-1.5 font-mono text-[11.5px] text-white transition-colors hover:bg-white/20"
    >
      {done ? "copied" : "copy"}
    </button>
  );
}

const ASSET_NOTE: Record<string, string> = {
  "whirlpool.exe": "The dock and menu bar",
  "whirlpool-settings.exe": "Native settings window",
  "Whirlpool-x64.zip": "Both programs, zipped",
  "install.ps1": "One-line per-user installer",
  "SHA256SUMS.txt": "Checksums for every file",
};

export function Release() {
  const { release, state } = useLatestRelease();
  const names = Object.keys(ASSET_NOTE);
  const rows = names.map((name) => ({ name, asset: release?.assets.find((a) => a.name === name) }));
  return (
    <section id="release" className="relative z-10 border-t hairline">
      <div className="mx-auto grid max-w-[1280px] gap-14 px-6 py-24 md:px-10 lg:grid-cols-12 lg:py-32">
        <div className="lg:col-span-5">
          <Eyebrow n="03" label="Release menu" depth="30 m" />
          <h2 className="reveal font-display text-[clamp(2.6rem,5.4vw,4.8rem)] leading-[0.96] tracking-[-0.02em]">
            Pick one. <em className="text-foam">Run it.</em>
          </h2>
          <p className="reveal mt-6 max-w-[42ch] text-[16.5px] leading-relaxed text-mist">
            Windows 10 (1903+) or 11, 64-bit. Installs for your user only, no administrator rights. The binaries are unsigned, so SmartScreen may ask: <span className="text-white">More info → Run anyway</span>.
          </p>
          <div className="reveal mt-8 rounded-2xl border hairline bg-black/20 p-4">
            <div className="mb-2 font-mono text-[10.5px] uppercase tracking-[.16em] text-mist">PowerShell · one line</div>
            <div className="flex items-center justify-between gap-3">
              <code className="min-w-0 overflow-x-auto whitespace-nowrap font-mono text-[12.5px] text-foam">{INSTALL_LINE}</code>
              <Copy text={INSTALL_LINE} />
            </div>
          </div>
        </div>

        {/* The download list, drawn like the dock's own context menu. */}
        <div className="reveal lg:col-span-7">
          <div className="glass-dock overflow-hidden rounded-[20px] p-2 text-[14.5px]">
            <div className="flex items-center justify-between px-4 pb-3 pt-3">
              <div className="flex items-center gap-3">
                <AppIcon size={34} />
                <div>
                  <div className="font-medium text-white">Whirlpool {release ? release.tag : ""}</div>
                  <div className="font-mono text-[11px] text-mist">
                    {state === "loading" && "checking GitHub…"}
                    {state === "none" && "release list unavailable · see GitHub"}
                    {release && `published ${new Date(release.published).toISOString().slice(0, 10)}`}
                  </div>
                </div>
              </div>
              {release && (
                <a href={release.page} className="font-mono text-[11.5px] text-foam underline-offset-4 hover:underline">
                  notes ↗
                </a>
              )}
            </div>
            <div className="mx-3 h-px bg-white/10" />
            <ul className="p-1.5">
              {rows.map(({ name, asset }, i) => (
                <li key={name}>
                  <a
                    href={asset?.url ?? (name === "whirlpool.exe" ? LATEST_EXE : `${REPO_URL}/releases`)}
                    className={`flex items-center justify-between gap-4 rounded-[12px] px-3.5 py-3 transition-colors ${
                      i === 0 ? "bg-foam/15 text-foam hover:bg-foam/25" : "text-white/90 hover:bg-white/8"
                    }`}
                  >
                    <span className="min-w-0">
                      <span className="block truncate font-mono text-[13.5px]">{name}</span>
                      <span className={`block text-[12.5px] ${i === 0 ? "text-foam/70" : "text-mist"}`}>{ASSET_NOTE[name]}</span>
                    </span>
                    <span className="figures shrink-0 font-mono text-[12px] text-mist">{asset ? formatSize(asset.size) : "—"}</span>
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ specs */

export function Specs() {
  const { release } = useLatestRelease();
  const size = (n: string) => {
    const a = release?.assets.find((x) => x.name === n);
    return a ? formatSize(a.size) : "~5 MB";
  };
  const rows: [string, string][] = [
    ["Language", "Rust, edition 2024. No web view, no managed runtime."],
    ["Programs", `whirlpool.exe (${size("whirlpool.exe")}) · whirlpool-settings.exe (${size("whirlpool-settings.exe")})`],
    ["Dock rendering", "Direct2D + DirectComposition + Windows.UI.Composition"],
    ["Settings UI", "egui on OpenGL (glow), one native window"],
    ["Platform", "Windows 10 1903+ and Windows 11, x64"],
    ["Glass", "Backdrop blur on Windows 11; solid tint on Windows 10"],
    ["Config", "%APPDATA%\\Whirlpool\\config.json"],
    ["Languages", "English · Español · Français"],
    ["Network", "Off by default. Optional: update check, AI plan usage"],
    ["Telemetry", "None"],
    ["License", "MIT (derived from Muelle, also MIT)"],
  ];
  const layers = [
    { depth: "0 m", name: "Programs", items: ["whirlpool.exe", "whirlpool-settings.exe"], tone: "from-foam/25" },
    { depth: "−10 m", name: "Crates", items: ["whirlpool-core · layout, springs, genie", "whirlpool-shell · Win32 integration", "whirlpool-config · serde + JSON"], tone: "from-current/25" },
    { depth: "−20 m", name: "Platform", items: ["Direct2D", "DirectComposition", "WinRT Composition", "egui · glow"], tone: "from-water/60" },
  ];
  return (
    <section id="specs" className="relative z-10 border-t hairline">
      <div className="mx-auto max-w-[1280px] px-6 py-24 md:px-10 lg:py-32">
        <Eyebrow n="04" label="Specifications" depth="40 m" />
        <h2 className="reveal max-w-[18ch] font-display text-[clamp(2.6rem,5.4vw,4.8rem)] leading-[0.96] tracking-[-0.02em]">
          Everything it is made of, <em className="text-foam">in one table.</em>
        </h2>
        <div className="mt-14 grid gap-14 lg:grid-cols-12">
          <dl className="reveal lg:col-span-7">
            {rows.map(([k, v]) => (
              <div key={k} className="grid grid-cols-[8.5rem_1fr] gap-4 border-t hairline py-3.5 last:border-b sm:grid-cols-[11rem_1fr]">
                <dt className="font-mono text-[11.5px] uppercase tracking-[.12em] text-mist">{k}</dt>
                <dd className="figures text-[15px] leading-snug text-white/90">{v}</dd>
              </div>
            ))}
          </dl>
          <div className="reveal lg:col-span-5">
            <div className="mb-3 font-mono text-[10.5px] uppercase tracking-[.16em] text-mist">Cutaway</div>
            <div className="space-y-2.5">
              {layers.map((l) => (
                <div key={l.name} className={`rounded-2xl border hairline bg-gradient-to-b ${l.tone} to-transparent p-4`}>
                  <div className="mb-3 flex items-baseline justify-between">
                    <span className="text-[14px] font-medium">{l.name}</span>
                    <span className="figures font-mono text-[11px] text-white/45">{l.depth}</span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {l.items.map((it) => (
                      <span key={it} className="rounded-full bg-black/25 px-3 py-1 font-mono text-[11.5px] text-white/80 ring-1 ring-white/10">
                        {it}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
            <p className="mt-4 text-[13.5px] leading-relaxed text-mist">
              Layout and animation live in a platform-independent crate, so the maths is unit-tested without a desktop.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ footer */

export function Footer() {
  return (
    <footer className="relative z-10 overflow-hidden border-t hairline">
      <div className="mx-auto max-w-[1280px] px-6 pb-10 pt-20 md:px-10">
        <div className="flex flex-col gap-10 md:flex-row md:items-end md:justify-between">
          <div>
            <div className="font-display text-[clamp(3.6rem,13vw,11rem)] leading-[0.8] tracking-[-0.03em] text-white/[0.92]">
              Whirl<em className="text-foam">pool</em>
            </div>
          </div>
          <div className="max-w-[38ch] text-[14px] leading-relaxed text-mist">
            Whirlpool is a fork of{" "}
            <a className="text-white underline decoration-white/30 underline-offset-4 hover:decoration-foam" href="https://github.com/crafter-station/Muelle">
              Muelle
            </a>{" "}
            by Crafter Station, released under the same MIT license. Original work © Muelle contributors.
          </div>
        </div>
        <div className="mt-14 flex flex-wrap items-center justify-between gap-4 border-t hairline pt-6 font-mono text-[11.5px] text-white/45">
          <span>MIT · no telemetry · native Rust</span>
          <span className="flex gap-6">
            <a className="hover:text-foam" href={REPO_URL}>GitHub</a>
            <a className="hover:text-foam" href={`${REPO_URL}/releases`}>Releases</a>
            <a className="hover:text-foam" href={`${REPO_URL}/blob/main/docs/README.md`}>Docs</a>
          </span>
        </div>
      </div>
    </footer>
  );
}
