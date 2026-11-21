import { useState } from 'react'
import DitherField from './components/DitherField'
import Dock from './components/Dock'
import MenuBar from './components/MenuBar'
import Playground, { WallpaperCanvas } from './components/Playground'
import Screens from './components/Screens'
import Trust from './components/Trust'
import logo from './assets/logo.png'

const REPO = 'https://github.com/prashantchataut/whirlpool'
const DOWNLOAD = `${REPO}/releases/latest/download/Whirlpool-x64.zip`

const NAV: [string, string][] = [
  ['Customize', '#customize'],
  ['Wallpapers', '#wallpapers'],
  ['Screens', '#screens'],
  ['Trust', '#trust'],
]

function Nav() {
  return (
    <header className="fixed inset-x-0 top-0 z-50 flex justify-center px-4 pt-4">
      <nav aria-label="Primary" className="glass flex w-full max-w-[980px] items-center justify-between rounded-full py-2 pr-2 pl-4">
        <a href="#top" className="flex items-center gap-2.5 font-display text-[17px] font-semibold text-white focus-visible:outline-2 focus-visible:outline-[#5ff0e6]">
          <img src={logo} alt="" className="h-7 w-7" />
          Whirlpool
        </a>
        <ul className="hidden items-center gap-1 md:flex">
          {NAV.map(([t, h]) => (
            <li key={h}>
              <a href={h} className="rounded-full px-4 py-2 text-[14px] text-white/75 transition-colors hover:bg-white/10 hover:text-white focus-visible:outline-2 focus-visible:outline-[#5ff0e6]">{t}</a>
            </li>
          ))}
        </ul>
        <a href={DOWNLOAD} className="btn-primary !min-h-10 !px-5 !text-[14px]">Download</a>
      </nav>
    </header>
  )
}

function Hero() {
  const [replay, setReplay] = useState(0)
  return (
    <section id="top" className="relative isolate overflow-hidden">
      <div className="absolute inset-0 -z-10 opacity-90"><DitherField cell={5} center={[0.74, 0.5]} /></div>
      <div className="absolute inset-x-0 bottom-0 -z-10 h-48 bg-gradient-to-t from-[#031620] to-transparent" />
      <div className="mx-auto grid min-h-[100svh] max-w-[1240px] items-center gap-12 px-5 pt-28 pb-16 sm:px-8 lg:grid-cols-[1fr_1.12fr]">
        <div>
          <h1 className="font-display text-[clamp(2.7rem,6.6vw,5.4rem)] leading-[0.96] font-semibold tracking-[-0.025em] text-white">
            A dock that moves like water.
          </h1>
          <p className="mt-6 max-w-[520px] text-[18px] leading-relaxed text-white/75">
            Whirlpool gives Windows 10 and 11 a magnifying glass dock and a menu bar. Two small native Rust programs, no web view, and it sleeps when you do.
          </p>
          <div className="mt-9 flex flex-wrap items-center gap-3">
            <a href={DOWNLOAD} className="btn-primary">Download for Windows</a>
            <a href={REPO} className="btn-ghost">View the source</a>
          </div>
          <p className="mt-5 text-[13.5px] text-white/55">Windows 10 (1903+) and 11, 64-bit. Free and MIT licensed.</p>
        </div>

        <div className="relative">
          <div className="glass relative aspect-[5/4] overflow-hidden rounded-[30px] p-0 shadow-[0_50px_120px_-30px_rgba(0,0,0,.8)]">
            <div className="absolute inset-0 [&>canvas]:h-full [&>canvas]:w-full"><WallpaperCanvas id="abyss" w={480} h={384} /></div>
            <MenuBar className="absolute inset-x-0 top-0 z-10" />
            <div className="glass absolute top-[16%] left-[7%] z-[1] w-[52%] rounded-2xl p-4 text-white">
              <div className="text-[13px] font-semibold">Dock</div>
              <div className="mt-3 space-y-2.5 text-[12px] text-white/70">
                {[['Magnification', true], ['Genie minimize', true], ['Auto-hide', false]].map(([l, on]) => (
                  <div key={l as string} className="flex items-center justify-between">{l as string}
                    <span className={`relative h-[18px] w-8 rounded-full ${on ? 'bg-[#12a4b8]' : 'bg-white/20'}`}><span className={`absolute top-[2px] h-[14px] w-[14px] rounded-full bg-white ${on ? 'left-[16px]' : 'left-[2px]'}`} /></span>
                  </div>
                ))}
              </div>
            </div>
            <div className="absolute inset-x-0 bottom-4 z-10"><Dock size={42} magnified={70} replayKey={replay} /></div>
          </div>
          <button onClick={() => setReplay((r) => r + 1)} className="btn-ghost absolute -bottom-5 left-1/2 !min-h-9 -translate-x-1/2 !px-4 !text-[13px]">Replay the startup</button>
        </div>
      </div>
    </section>
  )
}

const FACTS: [string, string][] = [
  ['2', 'native executables'],
  ['24', 'drawn wallpapers'],
  ['3', 'dock edges'],
  ['0', 'telemetry'],
]

function Facts() {
  return (
    <section aria-label="At a glance" className="mx-auto max-w-[1240px] px-5 sm:px-8">
      <dl className="glass grid grid-cols-2 divide-white/10 rounded-[28px] sm:grid-cols-4 sm:divide-x">
        {FACTS.map(([n, l]) => (
          <div key={l} className="px-7 py-7">
            <dt className="font-mono text-[clamp(2.2rem,4vw,3.2rem)] leading-none font-medium text-[#5ff0e6] tabular-nums">{n}</dt>
            <dd className="mt-2 text-[14.5px] text-white/70">{l}</dd>
          </div>
        ))}
      </dl>
    </section>
  )
}

function Footer() {
  return (
    <footer className="mx-auto mt-10 max-w-[1240px] px-5 pb-12 sm:px-8">
      <div className="glass flex flex-col items-start justify-between gap-6 rounded-[28px] p-7 sm:flex-row sm:items-center">
        <div className="flex items-center gap-3">
          <img src={logo} alt="" className="h-10 w-10" />
          <div>
            <div className="font-display text-lg font-semibold text-white">Whirlpool</div>
            <div className="text-[13.5px] text-white/60">A dock and menu bar for Windows. MIT licensed.</div>
          </div>
        </div>
        <div className="flex flex-wrap gap-x-6 gap-y-2 text-[14px] text-white/75">
          <a className="hover:text-white" href={`${REPO}/releases`}>Releases</a>
          <a className="hover:text-white" href={REPO}>Source</a>
          <a className="hover:text-white" href={`${REPO}/blob/main/CHANGELOG.md`}>Changelog</a>
          <a className="hover:text-white" href={`${REPO}/blob/main/docs/TRUST.md`}>Trust</a>
          <a className="hover:text-white" href={`${REPO}/issues`}>Issues</a>
        </div>
      </div>
    </footer>
  )
}

export default function App() {
  return (
    <div className="relative min-h-screen overflow-x-clip">
      <a href="#customize" className="sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-[60] focus:rounded-lg focus:bg-white focus:px-4 focus:py-2 focus:text-black">Skip to content</a>
      <Nav />
      <main>
        <Hero />
        <Facts />
        <Playground />
        <Screens />
        <Trust />
      </main>
      <Footer />
    </div>
  )
}
