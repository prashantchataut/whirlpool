import { useState } from "react";
import { CREDIT, CREDIT_URL, FILES, RELEASE_URL, REPO, SUMS, SUMS_URL, VERSION } from "./lib/release";
import { SHOTS } from "./lib/shots";

const base = import.meta.env.BASE_URL;
const shot = (file: string) => `${base}captures/desk/${file}.jpg`;

function Nav() {
  return (
    <header className="fixed inset-x-0 top-0 z-50 border-b border-line bg-ink/70 backdrop-blur-xl">
      <div className="mx-auto flex h-14 max-w-[1180px] items-center justify-between px-5">
        <a href="#top" className="font-display text-[17px] font-bold tracking-tight">
          Whirlpool
        </a>
        <nav className="flex items-center gap-6 text-[14px] text-mute" aria-label="Sections">
          <a className="hover:text-foam" href="#desktop">Desktop</a>
          <a className="hover:text-foam" href="#wallpapers">Wallpapers</a>
          <a className="hover:text-foam" href="#specs">Specs</a>
          <a className="hover:text-foam" href="#download">Download</a>
          <a className="hover:text-foam" href={REPO}>GitHub</a>
        </nav>
      </div>
    </header>
  );
}

function Hero() {
  return (
    <section id="top" className="mx-auto max-w-[1180px] px-5 pt-32 text-center">
      <p className="eyebrow">v{VERSION} · Windows 10 and 11 · x64 and ARM64</p>
      <h1 className="mx-auto mt-5 max-w-[780px] font-display text-[clamp(2.6rem,7vw,5.2rem)] font-extrabold leading-[0.98] tracking-[-0.035em]">
        The taskbar, rebuilt as a dock.
      </h1>
      <p className="mx-auto mt-6 max-w-[560px] text-[17px] leading-relaxed text-mute">
        Dock, menu bar and island in one native Rust program. 24 MB idle. Quit it and the taskbar is back.
      </p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <a href={FILES[0].url} className="rounded-full bg-teal px-6 py-3 text-[15px] font-semibold text-ink hover:brightness-110">
          Download for x64
        </a>
        <a href={FILES[1].url} className="rounded-full border border-line px-6 py-3 text-[15px] font-medium hover:bg-white/10">
          Download for ARM64
        </a>
      </div>
    </section>
  );
}

function Desktop() {
  const [current, setCurrent] = useState(SHOTS[0].id);
  const active = SHOTS.find((s) => s.id === current) ?? SHOTS[0];
  return (
    <section id="desktop" className="mx-auto mt-14 max-w-[1280px] scroll-mt-20 px-5">
      <div className="overflow-hidden rounded-2xl border border-line bg-ink-2 shadow-[0_40px_120px_-40px_rgba(0,0,0,0.9)]">
        <img
          key={active.file}
          src={shot(active.file)}
          alt={active.caption}
          width={1920}
          height={1080}
          className="block aspect-video w-full object-cover"
        />
      </div>
      <div role="tablist" aria-label="Screens" className="mt-5 flex flex-wrap justify-center gap-2">
        {SHOTS.map((s) => (
          <button
            key={s.id}
            role="tab"
            aria-selected={s.id === current}
            onClick={() => setCurrent(s.id)}
            className={
              "rounded-full px-4 py-2 text-[13.5px] transition-colors " +
              (s.id === current ? "bg-foam text-ink" : "border border-line text-mute hover:text-foam")
            }
          >
            {s.tab}
          </button>
        ))}
      </div>
      <p className="mt-4 text-center text-[14px] text-mute">{active.caption}</p>
      <p className="mt-1 text-center text-[12.5px] text-faint">
        Captured from the running program on a GitHub-hosted Windows runner, 1920×1080, dark mode, not rendered. Wallpapers: Pexels photographs from the bundled pack. The track is a staged media session with a drawn cover. "Test Mode" bottom right is the runner's own watermark.
      </p>
    </section>
  );
}

function Specs() {
  const rows: [string, string][] = [
    ["11.9 MB", "installer, x64"],
    ["24 MB", "idle, private memory"],
    ["0.04 %", "idle CPU"],
    ["3 keys", "taken by the low-level hook: volume up, down, mute"],
    ["Win 10 1903+", "and Windows 11"],
    ["MIT", "open source, no telemetry"],
  ];
  return (
    <section id="specs" className="mx-auto mt-28 max-w-[1180px] scroll-mt-20 px-5">
      <h2 className="font-display text-[clamp(1.8rem,4vw,2.8rem)] font-extrabold tracking-[-0.03em]">Small and quiet.</h2>
      <dl className="mt-8 grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-line bg-line sm:grid-cols-3">
        {rows.map(([value, label]) => (
          <div key={label} className="bg-ink px-6 py-7">
            <dt className="font-display text-[28px] font-bold tracking-tight">{value}</dt>
            <dd className="mt-1 text-[14px] text-mute">{label}</dd>
          </div>
        ))}
      </dl>
      <p className="mt-4 text-[13px] text-faint">
        Memory and CPU from a GitHub-hosted Windows runner (STUDY.md in the repository). Not a personal desktop.
      </p>
    </section>
  );
}

function Wallpapers() {
  const groups = ["Abstract", "AMOLED", "Nature", "Minimal", "Space", "Classics"];
  return (
    <section id="wallpapers" className="mx-auto mt-28 max-w-[1180px] scroll-mt-20 px-5">
      <div className="grid items-center gap-10 lg:grid-cols-[1fr_1.35fr]">
        <div>
          <h2 className="font-display text-[clamp(1.8rem,4vw,2.8rem)] font-extrabold leading-[1.02] tracking-[-0.03em]">
            4K wallpapers, one click.
          </h2>
          <p className="mt-4 text-[16px] leading-relaxed text-mute">
            Twenty pictures in six groups: {groups.join(", ")}. Apply downloads the 3840×2160 file to{" "}
            <code className="font-mono text-[13px] text-foam">%LOCALAPPDATA%\Whirlpool\Wallpapers</code> and sets it as your
            desktop. An optional switch matches the accent to the picture.
          </p>
          <p className="mt-5 text-[15px]">
            <a className="text-teal underline underline-offset-4" href={CREDIT_URL}>
              {CREDIT}
            </a>
          </p>
          <p className="mt-3 text-[13px] text-faint">
            Every tile links to its page. Pictures belong to their authors and are cleared for personal desktop use: they
            download to your PC when you press Apply. This site and repository do not host them.
          </p>
        </div>
        <img
          src={shot("09-wallpaper-studio")}
          alt="The wallpaper studio"
          width={1920}
          height={1080}
          loading="lazy"
          className="w-full rounded-2xl border border-line"
        />
      </div>
    </section>
  );
}

function Download() {
  return (
    <section id="download" className="mx-auto mt-28 max-w-[1180px] scroll-mt-20 px-5">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <h2 className="font-display text-[clamp(1.8rem,4vw,2.8rem)] font-extrabold tracking-[-0.03em]">Download.</h2>
        <p className="text-[14px] text-mute">
          <span className="rounded-full border border-line px-3 py-1 font-mono text-[12px] text-teal">v{VERSION} pre-release</span>{" "}
          <a className="ml-2 underline underline-offset-4" href={RELEASE_URL}>Release notes</a>
        </p>
      </div>
      <div className="mt-6 overflow-x-auto rounded-2xl border border-line">
        <table className="w-full min-w-[640px] text-left text-[14px]">
          <thead className="text-[12px] uppercase tracking-wider text-faint">
            <tr>
              <th className="px-5 py-3 font-medium">File</th>
              <th className="px-5 py-3 font-medium">CPU</th>
              <th className="px-5 py-3 font-medium">SHA-256</th>
            </tr>
          </thead>
          <tbody>
            {FILES.map((f) => (
              <tr key={f.name} className="border-t border-line">
                <td className="px-5 py-3">
                  <a className="font-medium text-foam underline underline-offset-4" href={f.url}>{f.name}</a>
                  <span className="ml-2 text-mute">{f.kind}</span>
                </td>
                <td className="px-5 py-3 text-mute">{f.arch}</td>
                <td className="px-5 py-3 font-mono text-[11.5px] text-mute break-all">{SUMS[f.name] ?? "see SHA256SUMS.txt"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="mt-4 text-[13.5px] leading-relaxed text-mute">
        Unsigned for now: SmartScreen may ask once. Check a file against{" "}
        <a className="text-foam underline underline-offset-4" href={SUMS_URL}>SHA256SUMS.txt</a>. The ARM64 build is compiled in
        CI and has not run on ARM hardware. <code className="font-mono text-foam">whirlpool --restore</code> brings the taskbar back.
      </p>
    </section>
  );
}

function Footer() {
  return (
    <footer className="mx-auto mt-28 max-w-[1180px] border-t border-line px-5 py-10 text-[13.5px] text-mute">
      <p>
        <a className="underline underline-offset-4 hover:text-foam" href={CREDIT_URL}>{CREDIT}</a>
      </p>
      <p className="mt-3">
        MIT licence · <a className="underline underline-offset-4" href={REPO}>Source</a> ·{" "}
        <a className="underline underline-offset-4" href={`${REPO}/blob/main/SCORECARD.md`}>Scorecard</a> ·{" "}
        <a className="underline underline-offset-4" href={`${REPO}/blob/main/docs/PRIVACY.md`}>Privacy</a> ·{" "}
        <a className="underline underline-offset-4" href={`${REPO}/blob/main/docs/TRUST.md`}>Trust</a>
      </p>
    </footer>
  );
}

export default function App() {
  return (
    <div className="relative overflow-x-clip">
      <Nav />
      <main>
        <Hero />
        <Desktop />
        <Specs />
        <Wallpapers />
        <Download />
      </main>
      <Footer />
    </div>
  );
}
