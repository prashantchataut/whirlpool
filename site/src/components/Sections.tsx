import { useState, type ReactNode } from "react";
import { Desktop, DEFAULT_BAR, DEFAULT_DOCK } from "./Desktop";
import { Dither } from "./Dither";
import {
  Arrow,
  Check,
  Copy,
  Download,
  Github,
  Shield,
} from "./Icons";
import { RELEASES_URL, REPO, SETUP_URL, VERSION, WALLS, ZIP_URL } from "../lib/theme";
import { PACK } from "../lib/pack";


/* ------------------------------------------------------------------ Nav */

export function Nav() {
  const links = [
    ["Dock", "#dock"],
    ["Customize", "#customize"],
    ["Charging", "#charging"],
    ["Wallpapers", "#wallpapers"],
    ["Trust", "#trust"],
  ];
  return (
    <header className="fixed inset-x-0 top-0 z-50 flex justify-center px-4 pt-4">
      <nav
        aria-label="Primary"
        className="glass flex w-full max-w-[880px] items-center justify-between rounded-full py-2 pl-4 pr-2"
      >
        <a href="#top" className="font-display text-[17px] font-extrabold tracking-tight">
          Whirlpool
        </a>
        <ul className="hidden items-center gap-1 md:flex">
          {links.map(([label, href]) => (
            <li key={href}>
              <a href={href} className="rounded-full px-3 py-1.5 text-[13px] text-mute transition-colors hover:text-foam">
                {label}
              </a>
            </li>
          ))}
        </ul>
        <div className="flex items-center gap-1.5">
          <a
            href={REPO}
            aria-label="Source on GitHub"
            className="grid h-9 w-9 place-items-center rounded-full text-mute transition-colors hover:text-foam"
          >
            <Github className="h-[18px] w-[18px]" />
          </a>
          <a
            href={SETUP_URL}
            className="rounded-full bg-teal px-4 py-2 text-[13px] font-semibold text-ink transition-transform hover:scale-[1.03]"
          >
            Download
          </a>
        </div>
      </nav>
    </header>
  );
}

/* ----------------------------------------------------------------- Hero */

export function Hero() {
  return (
    <section id="top" className="relative overflow-hidden pt-32 sm:pt-36">
      <div
        className="pointer-events-none absolute -right-40 top-6 h-[620px] w-[760px] max-w-full opacity-60 sm:-right-20"
        style={{ maskImage: "radial-gradient(closest-side, #000 35%, transparent 100%)", WebkitMaskImage: "radial-gradient(closest-side, #000 35%, transparent 100%)" }}
      >
        <Dither variant="whirl" cell={6} fps={16} speed={0.9} />
      </div>
      <div className="relative mx-auto max-w-[1180px] px-5">
        <p className="eyebrow mb-5">For Windows 10 and 11 · v{VERSION}</p>
        <h1 className="max-w-[820px] font-display text-[clamp(2.6rem,7.2vw,5.6rem)] font-extrabold leading-[0.98] tracking-[-0.022em]">
          The taskbar, rebuilt as a dock.
        </h1>
        <p className="mt-6 max-w-[560px] text-[17px] leading-relaxed text-mute">
          Whirlpool swaps the Windows taskbar for a glass dock and a menu bar you can restyle. It is one small program,
          needs no admin rights, sleeps at 0&nbsp;% CPU when you are not touching it, and gives the taskbar back the moment
          you quit.
        </p>
        <div className="mt-8 flex flex-wrap items-center gap-3">
          <a
            href={SETUP_URL}
            className="group inline-flex items-center gap-2.5 rounded-full bg-teal px-6 py-3.5 text-[15px] font-semibold text-ink shadow-[0_0_0_1px_rgba(255,255,255,0.2)_inset,0_18px_50px_-18px_rgba(62,230,204,0.7)] transition-transform hover:scale-[1.03]"
          >
            <Download className="h-[18px] w-[18px]" />
            Download for Windows
          </a>
          <a
            href={REPO}
            className="glass inline-flex items-center gap-2.5 rounded-full px-5 py-3.5 text-[15px] font-medium transition-colors hover:bg-white/10"
          >
            <Github className="h-[18px] w-[18px]" />
            Read the source
          </a>
        </div>
        <p className="mt-4 font-mono text-[11.5px] text-faint">
          Whirlpool-Setup.exe · about 5 MB · per-user install · MIT license
        </p>

        <div className="relative mt-14 sm:mt-16">
          <div
            className="pointer-events-none absolute -inset-x-10 -bottom-10 -top-6 -z-10 blur-3xl"
            style={{ background: "radial-gradient(60% 60% at 50% 40%, rgba(62,230,204,0.18), transparent 70%)" }}
          />
          <Desktop
            wall={WALLS[0]}
            bar={{ ...DEFAULT_BAR, style: "floating", width: 100, tint: "ocean", opacity: 52 }}
            dock={DEFAULT_DOCK}
            rounded="rounded-[22px]"
            className="shadow-[0_60px_120px_-40px_rgba(0,0,0,0.95)]"
          />
          <p className="mt-3 text-center font-mono text-[11px] text-faint">
            Design render of the app, not a capture. Move your pointer over the dock.
          </p>
        </div>
      </div>
    </section>
  );
}

/* ---------------------------------------------------------------- Stats */

export function Stats() {
  const items = [
    { k: "0 %", v: "CPU while idle", d: "The dock sleeps between events and paces animation with the display." },
    { k: "5 MB", v: "Installer", d: "Whirlpool-Setup.exe, per-user, no wizard pages you did not ask for." },
    { k: "0", v: "Admin prompts", d: "Installs and runs as you. Nothing is written to system folders." },
    { k: "1", v: "Program", d: "Settings is built in. There is no second app to install." },
  ];
  return (
    <section className="mx-auto mt-24 max-w-[1180px] px-5">
      <dl className="grid overflow-hidden rounded-2xl border border-white/10 sm:grid-cols-2 lg:grid-cols-4">
        {items.map((s, i) => (
          <div key={s.v} className={`relative p-6 ${i > 0 ? "border-t border-white/10 sm:border-t-0 lg:border-l" : ""} ${i === 2 ? "sm:border-t lg:border-t-0" : ""} ${i === 1 ? "sm:border-l" : ""} ${i === 3 ? "sm:border-l sm:border-t lg:border-t-0" : ""}`}>
            <dt className="font-mono text-[11px] uppercase tracking-[0.14em] text-faint">{s.v}</dt>
            <dd className="mt-3 font-display text-5xl font-extrabold tracking-tight">{s.k}</dd>
            <dd className="mt-3 text-[13.5px] leading-relaxed text-mute">{s.d}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

/* --------------------------------------------------------- Section frame */

export function Section({
  id,
  eyebrow,
  title,
  lead,
  children,
}: {
  id: string;
  eyebrow: string;
  title: string;
  lead?: string;
  children: ReactNode;
}) {
  return (
    <section id={id} className="mx-auto mt-32 max-w-[1180px] scroll-mt-24 px-5">
      <p className="eyebrow mb-4">{eyebrow}</p>
      <h2 className="max-w-[760px] font-display text-[clamp(2rem,4.6vw,3.6rem)] font-extrabold leading-[1] tracking-[-0.03em]">
        {title}
      </h2>
      {lead && <p className="mt-5 max-w-[620px] text-[16.5px] leading-relaxed text-mute">{lead}</p>}
      <div className="mt-12">{children}</div>
    </section>
  );
}

/* ----------------------------------------------------------- Dock demo */

export function DockDemo() {
  const bullets: [string, string][] = [
    ["Magnification", "Icons swell under the pointer along a smooth curve. Size, spacing and strength are yours."],
    ["Genie minimize", "Windows fold into their icon and come back out of it."],
    ["Live thumbnails", "Rest on an icon to see what its windows look like."],
    ["Four edges", "Bottom, top, left or right, with auto-hide when you want the whole screen."],
    ["Downloads and Recycle Bin", "Drop files on the bin. Open the Downloads stack without leaving the desktop."],
  ];
  return (
    <div className="grid items-center gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,380px)]">
      <Desktop wall={WALLS[2]} bar={{ ...DEFAULT_BAR, tint: "slate", opacity: 55 }} dock={{ ...DEFAULT_DOCK, size: 56 }} />
      <ul className="grid gap-5">
        {bullets.map(([h, p]) => (
          <li key={h} className="border-l border-teal/40 pl-4">
            <h3 className="font-display text-lg font-bold tracking-tight">{h}</h3>
            <p className="mt-1 text-[14.5px] leading-relaxed text-mute">{p}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}

/* ------------------------------------------------------------ Wallpapers */

export function Wallpapers() {
  const base = import.meta.env.BASE_URL;
  return (
    <div>
      <ul className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
        {PACK.map((w) => (
          <li key={w.id}>
            <figure className="group relative overflow-hidden rounded-2xl">
              <img
                src={`${base}pack/${w.id}.webp`}
                alt={`${w.title}, photograph by ${w.author}`}
                width={720}
                height={405}
                loading="lazy"
                decoding="async"
                className="aspect-[16/9] w-full object-cover transition-transform duration-700 group-hover:scale-105 motion-reduce:transition-none"
              />
              <figcaption className="absolute inset-x-0 bottom-0 flex flex-col items-start gap-0.5 bg-gradient-to-t from-black/80 via-black/35 to-transparent p-3 pt-10">
                <span className="font-display text-[15px] font-bold leading-tight">{w.title}</span>
                <a
                  href={`https://www.pexels.com/photo/${w.pexels}/`}
                  className="font-mono text-[10.5px] text-white/75 underline-offset-2 hover:underline focus-visible:underline"
                >
                  {w.author} · Pexels
                </a>
              </figcaption>
            </figure>
          </li>
        ))}
      </ul>
      <p className="mt-4 max-w-[640px] text-[14px] leading-relaxed text-mute">
        The pack is stored inside whirlpool.exe, so choosing one needs no download. It adds about 5.7&nbsp;MB to the
        program. Photographs are used under the Pexels License; every photographer and link is in ASSETS_LICENSES.md.
      </p>
    </div>
  );
}

/* ----------------------------------------------------------------- Trust */

function CopyLine({ text }: { text: string }) {
  const [done, setDone] = useState(false);
  return (
    <div className="flex items-center justify-between gap-3 rounded-lg bg-black/50 px-3.5 py-2.5 font-mono text-[12px] text-foam/90">
      <code className="min-w-0 overflow-x-auto whitespace-nowrap">{text}</code>
      <button
        aria-label="Copy command"
        onClick={() => {
          void navigator.clipboard?.writeText(text);
          setDone(true);
          window.setTimeout(() => setDone(false), 1400);
        }}
        className="shrink-0 text-mute transition-colors hover:text-teal"
      >
        {done ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
      </button>
    </div>
  );
}

export function Trust() {
  const [taskbar, setTaskbar] = useState(false);
  return (
    <div className="grid gap-5 lg:grid-cols-2">
      <div className="glass-strong rounded-2xl p-6 sm:p-8">
        <div className="flex items-center gap-3">
          <Shield className="h-6 w-6 text-teal" />
          <h3 className="font-display text-2xl font-bold tracking-tight">If Windows warns you</h3>
        </div>
        <p className="mt-4 text-[14.5px] leading-relaxed text-mute">
          A new, unsigned program has no reputation yet, so SmartScreen shows a blue box. That is about how many people have
          downloaded the file, not about what is inside it. The code is public, and every release is built by GitHub Actions,
          not on a personal machine.
        </p>
        <ol className="mt-6 grid gap-4 text-[14.5px]">
          {[
            ["Run Whirlpool-Setup.exe.", "If the blue box appears, choose More info."],
            ["Choose Run anyway.", "Setup is per-user. It never asks for administrator rights."],
            ["Check the file if you like.", "Compare the SHA-256 below with SHA256SUMS.txt on the release page."],
          ].map(([a, b], i) => (
            <li key={a} className="flex gap-4">
              <span className="grid h-7 w-7 shrink-0 place-items-center rounded-full border border-teal/50 font-mono text-[12px] text-teal">
                {i + 1}
              </span>
              <span>
                <strong className="font-semibold">{a}</strong> <span className="text-mute">{b}</span>
              </span>
            </li>
          ))}
        </ol>
        <div className="mt-6 grid gap-2">
          <CopyLine text="Get-FileHash .\Whirlpool-Setup.exe -Algorithm SHA256" />
          <CopyLine text={`gh attestation verify Whirlpool-Setup.exe --repo prashantchataut/whirlpool`} />
        </div>
        <p className="mt-4 text-[12.5px] leading-relaxed text-faint">
          Signing removes the warning entirely. The release workflow signs automatically once a certificate is configured;
          see docs/TRUST.md for the current state.
        </p>
      </div>

      <div className="glass-strong flex flex-col rounded-2xl p-6 sm:p-8">
        <h3 className="font-display text-2xl font-bold tracking-tight">Your taskbar is only tucked away</h3>
        <p className="mt-4 text-[14.5px] leading-relaxed text-mute">
          Whirlpool hides the Windows taskbar while it runs and restores it when it stops. A small guardian process puts it
          back even if the dock crashes or is killed from Task Manager.
        </p>
        <div className="relative mt-6 flex-1">
          <Desktop
            wall={WALLS[3]}
            bar={{ ...DEFAULT_BAR, opacity: 40 }}
            dock={DEFAULT_DOCK}
            showDock={!taskbar}
            rounded="rounded-xl"
          >
            <div
              className="absolute inset-x-0 bottom-0 z-30 flex h-[3.4cqw] items-center gap-[1.4cqw] bg-[#1b2428]/95 px-[1.4cqw] backdrop-blur transition-transform duration-500"
              style={{ transform: taskbar ? "translateY(0)" : "translateY(105%)" }}
              aria-hidden={!taskbar}
            >
              <span className="grid h-[2.1cqw] w-[2.1cqw] place-items-center rounded-[0.3cqw] bg-[#3b9be8]" />
              {Array.from({ length: 6 }).map((_, i) => (
                <span key={i} className="h-[2cqw] w-[2cqw] rounded-[0.4cqw] bg-white/15" />
              ))}
              <span className="ml-auto font-mono text-[1.1cqw] text-white/70">9:41</span>
            </div>
          </Desktop>
          <button
            role="switch"
            aria-checked={taskbar}
            onClick={() => setTaskbar((v) => !v)}
            className="glass absolute bottom-4 left-4 z-40 flex items-center gap-3 rounded-full py-1.5 pl-4 pr-2 text-[12.5px]"
          >
            {taskbar ? "Windows taskbar" : "Whirlpool running"}
            <span className={`relative h-6 w-11 rounded-full transition-colors ${taskbar ? "bg-white/20" : "bg-teal"}`}>
              <span className={`absolute top-[3px] h-[18px] w-[18px] rounded-full bg-white transition-[left] ${taskbar ? "left-[3px]" : "left-[23px]"}`} />
            </span>
          </button>
        </div>
        <ul className="mt-6 grid gap-2 text-[13.5px] text-mute">
          <li>Quit from the dock menu and the taskbar returns at once.</li>
          <li>Uninstall from Windows Settings and nothing is left behind.</li>
          <li>No account, no telemetry code in the repository.</li>
        </ul>
      </div>
    </div>
  );
}

/* ---------------------------------------------------------------- Install */

export function Install() {
  return (
    <section id="install" className="mx-auto mt-32 max-w-[1180px] scroll-mt-24 px-5">
      <div className="glass-strong relative overflow-hidden rounded-3xl p-8 sm:p-14">
        <div className="pointer-events-none absolute inset-y-0 right-0 w-1/2 opacity-60 [mask-image:linear-gradient(90deg,transparent,#000)]">
          <Dither variant="rise" cell={6} fps={12} />
        </div>
        <div className="relative max-w-[560px]">
          <p className="eyebrow mb-4">v{VERSION}</p>
          <h2 className="font-display text-[clamp(2rem,5vw,3.8rem)] font-extrabold leading-[1] tracking-[-0.03em]">
            Try it for an afternoon.
          </h2>
          <p className="mt-5 text-[16px] leading-relaxed text-mute">
            If it is not for you, quit it from the dock menu and Windows looks exactly as before.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <a
              href={SETUP_URL}
              className="inline-flex items-center gap-2.5 rounded-full bg-teal px-6 py-3.5 text-[15px] font-semibold text-ink transition-transform hover:scale-[1.03]"
            >
              <Download className="h-[18px] w-[18px]" /> Whirlpool-Setup.exe
            </a>
            <a
              href={ZIP_URL}
              className="glass inline-flex items-center gap-2 rounded-full px-5 py-3.5 text-[15px] font-medium hover:bg-white/10"
            >
              Portable .zip
            </a>
            <a
              href={RELEASES_URL}
              className="inline-flex items-center gap-2 px-3 py-3.5 text-[15px] text-mute transition-colors hover:text-foam"
            >
              All releases <Arrow className="h-4 w-4" />
            </a>
          </div>
          <div className="mt-8 grid gap-2">
            <CopyLine text="irm https://github.com/prashantchataut/whirlpool/releases/latest/download/install.ps1 | iex" />
            <CopyLine text="git clone https://github.com/prashantchataut/whirlpool && cd whirlpool && cargo run --release -p whirlpool" />
          </div>
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------- Changelog */

export function WhatsNew() {
  const rows: [string, string][] = [
    ["Update reminders", "When a newer release exists, Windows shows one notification. It repeats after three days, three times at most per version, and never downloads anything on its own. One switch turns it off."],
    ["Updates that install, and roll back", "Settings downloads the release for your CPU, checks its SHA-256 against the published sums, installs and restarts. Rolling back to the previous version was run end to end on a clean Windows runner."],
    ["System overlay", "CPU, memory, GPU, network, battery and the desktop frame rate behind one hotkey. It is a separate process, so nothing runs while it is hidden."],
    ["Wallpaper rotation", "Every five minutes to a day, in order or shuffled, from a folder or the bundled photographs, with a different folder for dark mode and a different picture per display."],
    ["Quick settings that follow your wallpaper", "The panel takes its colours from the picture and the Windows theme, with real Wi-Fi and Bluetooth switches, volume, Cast, Night light and Battery saver."],
    ["Bottom bar above the dock", "With the dock on the bottom edge, the menu bar now stands clear of it instead of underneath."],
    ["ARM64 installer", "Built in CI for Windows on ARM. The x64 installer also runs there through emulation."],
  ];
  const next: [string, string][] = [
    ["Dynamic island presets", "Compact, Classic, Wide and Expressive change the live island, closed and open. Seen in four runner captures; not yet on a personal desktop."],
    ["Motion for dragged windows", "A soft shadow behind the window you drag, a preview of where it will snap, and a spring settle on release. Windows does not let a program tilt another program's window, so there is no tilt. Seen in runner captures."],
    ["Launchpad-style Start grid", "The dock's Start button and Ctrl+Alt+Space open a grid of your apps with search and arrow-key selection. Seen in runner captures; icons fill in a moment after it opens."],
    ["Volume, Wi-Fi, Bluetooth and brightness pop-ups", "Not started."],
    ["Display scaling pass at 100, 150 and 200 percent", "Not started."],
    ["Verified on ARM hardware", "The ARM64 build has not been run on an ARM device by the maintainer yet."],
    ["Signed installers", "Needs a code-signing certificate. Until then each release publishes SHA-256 sums and build provenance."],
  ];
  return (
    <div className="grid gap-5 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
      <ol className="glass divide-y divide-white/[0.07] rounded-2xl">
        {rows.map(([h, p], i) => (
          <li key={h} className="flex gap-5 p-5 sm:p-6">
            <span className="font-mono text-[11px] text-teal">{String(i + 1).padStart(2, "0")}</span>
            <div>
              <h3 className="font-display text-lg font-bold tracking-tight">{h}</h3>
              <p className="mt-1 text-[14.5px] leading-relaxed text-mute">{p}</p>
            </div>
          </li>
        ))}
      </ol>
      <div className="self-start rounded-2xl border border-white/10 p-6">
        <h3 className="eyebrow">Roadmap</h3>
        <ul className="mt-4 grid gap-4">
          {next.map(([h, p]) => (
            <li key={h}>
              <h4 className="text-[14.5px] font-semibold text-foam">{h}</h4>
              <p className="mt-1 text-[13.5px] leading-relaxed text-mute">{p}</p>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

/* ----------------------------------------------------------------- Footer */

export function Footer() {
  return (
    <footer className="mx-auto mt-32 max-w-[1180px] px-5 pb-14">
      <div className="flex flex-col justify-between gap-8 border-t border-white/10 pt-10 sm:flex-row">
        <div>
          <div className="font-display text-xl font-extrabold tracking-tight">
            Whirlpool
          </div>
          <p className="mt-3 max-w-[340px] text-[13px] leading-relaxed text-faint">
            Made by Prashant Chataut. MIT licensed. Not affiliated with Microsoft. Photos by Nala Berting, Mete Kaan Özdilek and
            Quang Nguyen Vinh via Pexels.
          </p>
        </div>
        <nav aria-label="Footer" className="grid grid-cols-2 gap-x-12 gap-y-2 text-[13.5px] text-mute">
          <a className="hover:text-foam" href={REPO}>Source</a>
          <a className="hover:text-foam" href={`${REPO}/releases`}>Releases</a>
          <a className="hover:text-foam" href={`${REPO}/blob/main/CHANGELOG.md`}>Changelog</a>
          <a className="hover:text-foam" href={`${REPO}/blob/main/docs/TRUST.md`}>Trust</a>
          <a className="hover:text-foam" href={`${REPO}/issues`}>Report a bug</a>
          <a className="hover:text-foam" href={`${REPO}/blob/main/LICENSE`}>License</a>
        </nav>
      </div>
    </footer>
  );
}
