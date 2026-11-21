import Dock from './Dock'
import MenuBar from './MenuBar'
import { WallpaperCanvas } from './Playground'
import logo from '../assets/logo.png'
import { PRESETS } from '../lib/presets'

/** A Windows 11 style window frame, glass. */
function Frame({ title, children, className = '' }: { title: string; children: React.ReactNode; className?: string }) {
  return (
    <div className={`glass overflow-hidden rounded-2xl text-white ${className}`}>
      <div className="flex h-9 items-center justify-between border-b border-white/10 px-3 text-[12px] text-white/70">
        <span className="flex items-center gap-2"><img src={logo} alt="" className="h-4 w-4" />{title}</span>
        <span className="flex gap-4 font-sans text-white/50"><span>–</span><span>▢</span><span>×</span></span>
      </div>
      {children}
    </div>
  )
}

const Toggle = ({ on = true }: { on?: boolean }) => (
  <span className={`relative h-5 w-9 shrink-0 rounded-full ${on ? 'bg-[#12a4b8]' : 'bg-white/20'}`}>
    <span className={`absolute top-0.5 h-4 w-4 rounded-full bg-white transition-all ${on ? 'left-[18px]' : 'left-0.5'}`} />
  </span>
)

/** The settings window (Personalize page), drawn from the app's own palette and layout. */
export function SettingsRender({ selected = 'abyss' }: { selected?: string }) {
  const pages = ['Dock', 'Personalize', 'Windows', 'Menu bar', 'System']
  const shown = PRESETS.filter((_, i) => i < 9)
  return (
    <Frame title="Whirlpool" className="w-full !rounded-xl !bg-[#07151d]/90">
      <div className="grid grid-cols-[150px_1fr] text-[12.5px]">
        <div className="space-y-1 bg-black/25 p-3">
          <div className="mb-4 flex items-center gap-2 px-1 text-[14px] font-semibold"><img src={logo} alt="" className="h-6 w-6" />Whirlpool</div>
          {pages.map((p) => (
            <div key={p} className={`relative rounded-lg px-3 py-2 ${p === 'Personalize' ? 'bg-[#12a4b8]/25 text-white' : 'text-white/65'}`}>
              {p === 'Personalize' && <span className="absolute top-2 bottom-2 left-0 w-[3px] rounded-r bg-[#5ff0e6]" />}
              {p}
            </div>
          ))}
        </div>
        <div className="space-y-3 p-4">
          <div className="text-[19px] font-semibold">Personalize</div>
          <div className="rounded-xl bg-white/[.06] p-3.5">
            <div className="mb-2 text-[11.5px] font-semibold text-white/60">Motion</div>
            {[['Animation style', ['Relaxed', 'Smooth', 'Snappy'], 1], ['Startup animation', ['Off', 'Glide', 'Rise', 'Swift'], 2]].map(([label, opts, on]) => (
              <div key={label as string} className="flex items-center justify-between border-b border-white/5 py-2">
                <span>{label as string}</span>
                <span className="flex gap-1">
                  {(opts as string[]).map((o, i) => (
                    <span key={o} className={`rounded-md px-2 py-0.5 ${i === on ? 'bg-[#12a4b8] text-white' : 'bg-white/10 text-white/65'}`}>{o}</span>
                  ))}
                </span>
              </div>
            ))}
            <div className="flex items-center justify-between pt-2"><span>Charging animation</span><Toggle /></div>
          </div>
          <div className="rounded-xl bg-white/[.06] p-3.5">
            <div className="mb-2 flex items-center justify-between"><span className="text-[11.5px] font-semibold text-white/60">Wallpapers</span><span className="rounded-md bg-white/10 px-2 py-0.5">Shuffle</span></div>
            <div className="grid grid-cols-3 gap-2">
              {shown.map((p) => (
                <div key={p.id}>
                  <div className={`aspect-[8/5] overflow-hidden rounded-sm ${p.id === selected ? 'ring-[3px] ring-[#0e9398]' : ''}`}>
                    <div className="h-full w-full [&>canvas]:h-full [&>canvas]:w-full"><WallpaperCanvas id={p.id} w={120} h={75} /></div>
                  </div>
                  <div className="mt-1 text-[11px] text-white/80">{p.name}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </Frame>
  )
}

function Desktop({ wall, children, charging = false }: { wall: string; children?: React.ReactNode; charging?: boolean }) {
  return (
    <div className="glass relative aspect-[16/10] overflow-hidden rounded-[24px] p-0">
      <div className="absolute inset-0 [&>canvas]:h-full [&>canvas]:w-full"><WallpaperCanvas id={wall} w={480} h={300} /></div>
      <MenuBar compact charging={charging} className="absolute inset-x-0 top-0 z-10" />
      {children}
    </div>
  )
}

export default function Screens() {
  return (
    <section id="screens" className="mx-auto max-w-[1240px] scroll-mt-24 px-5 py-24 sm:px-8">
      <div className="max-w-[680px]">
        <h2 className="font-display text-[clamp(2rem,4.6vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">
          How it sits on a Windows desktop.
        </h2>
        <p className="mt-4 text-[17px] leading-relaxed text-white/72">
          These are design renders drawn from the app's own palette, labels and layout, built from the same components as the demo above. They are not captures of a running Windows session.
        </p>
      </div>
      <div className="mt-12 grid gap-6 lg:grid-cols-5">
        <div className="lg:col-span-3">
          <Desktop wall="dusk">
            <Frame title="Browser" className="absolute top-[14%] left-[8%] z-[1] h-[52%] w-[58%] !bg-white/10">
              <div className="space-y-2 p-4">
                <div className="h-3 w-2/5 rounded bg-white/30" />
                <div className="h-2 w-4/5 rounded bg-white/15" />
                <div className="h-2 w-3/5 rounded bg-white/15" />
                <div className="mt-4 grid grid-cols-3 gap-2">{[0, 1, 2].map((i) => <div key={i} className="aspect-[4/3] rounded-lg bg-white/10" />)}</div>
              </div>
            </Frame>
            <div className="absolute inset-x-0 bottom-3 z-10"><Dock size={30} magnified={46} startup="off" /></div>
          </Desktop>
        </div>
        <div className="lg:col-span-2"><SettingsRender /></div>
        <div className="lg:col-span-2">
          <Desktop wall="blue-hour" charging>
            <div className="glass absolute top-9 right-3 z-20 flex items-center gap-2 rounded-xl px-3 py-1.5 text-[11.5px] text-white"><span className="text-[#ffc857]">⚡</span>Charging, 64%</div>
            <div className="absolute inset-y-6 left-3 z-10"><Dock position="left" size={26} magnified={38} startup="off" charging /></div>
          </Desktop>
        </div>
        <div className="lg:col-span-3">
          <Desktop wall="abyss">
            <div className="absolute inset-x-0 bottom-3 z-10 grid grid-cols-3 gap-2 px-6">
              {[1, 0.55, 0.12].map((v, i) => (
                <div key={i} style={{ opacity: 0.35 + v * 0.65, transform: `translateY(${(1 - v) * 40}px)` }}><Dock size={20} magnified={20} magnify={false} startup="off" /></div>
              ))}
            </div>
          </Desktop>
        </div>
      </div>
    </section>
  )
}
