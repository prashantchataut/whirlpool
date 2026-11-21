import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import Dock, { type AnimationStyle, type DockPosition, type StartupAnimation } from './Dock'
import MenuBar from './MenuBar'
import { PRESETS } from '../lib/presets'
import { render } from '../lib/wallpaper'

export function WallpaperCanvas({ id, w = 320, h = 200, className = '' }: { id: string; w?: number; h?: number; className?: string }) {
  const ref = useRef<HTMLCanvasElement>(null)
  useEffect(() => {
    const preset = PRESETS.find((p) => p.id === id) ?? PRESETS[0]
    const c = ref.current
    const ctx = c?.getContext('2d')
    if (!c || !ctx) return
    c.width = w
    c.height = h
    ctx.putImageData(render(preset, w, h), 0, 0)
  }, [id, w, h])
  return <canvas ref={ref} aria-hidden className={className} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
}

function Seg<T extends string | boolean | null>({ value, options, onChange, label }: { value: T; options: [T, string][]; onChange: (v: T) => void; label: string }) {
  return (
    <div role="radiogroup" aria-label={label} className="flex rounded-xl bg-black/25 p-1 ring-1 ring-white/10">
      {options.map(([v, text]) => (
        <button
          key={String(v)}
          role="radio"
          aria-checked={value === v}
          onClick={() => onChange(v)}
          className={`min-h-9 flex-1 rounded-lg px-3 text-[13px] font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[#5ff0e6] ${
            value === v ? 'bg-white/90 text-[#04202c]' : 'text-white/75 hover:bg-white/10 hover:text-white'
          }`}
        >
          {text}
        </button>
      ))}
    </div>
  )
}

const Row = ({ label, children, value }: { label: string; children: ReactNode; value?: string }) => (
  <div className="grid gap-2">
    <div className="flex items-baseline justify-between">
      <span className="text-[13.5px] font-medium text-white/90">{label}</span>
      {value && <span className="font-mono text-[12px] text-[#5ff0e6]">{value}</span>}
    </div>
    {children}
  </div>
)

const Range = ({ min, max, value, onChange, label }: { min: number; max: number; value: number; onChange: (n: number) => void; label: string }) => (
  <input type="range" aria-label={label} min={min} max={max} value={value} onChange={(e) => onChange(+e.target.value)} className="range" />
)

export default function Playground() {
  const [position, setPosition] = useState<DockPosition>('bottom')
  const [size, setSize] = useState(48)
  const [magnified, setMagnified] = useState(77)
  const [magnify, setMagnify] = useState(true)
  const [style, setStyle] = useState<AnimationStyle>('smooth')
  const [startup, setStartup] = useState<StartupAnimation>('rise')
  const [replay, setReplay] = useState(0)
  const [opacity, setOpacity] = useState(50)
  const [showDate, setShowDate] = useState(true)
  const [hour24, setHour24] = useState<boolean | null>(null)
  const [charging, setCharging] = useState(false)
  const [wall, setWall] = useState('abyss')

  useEffect(() => {
    if (!charging) return
    const id = window.setTimeout(() => setCharging(false), 4200)
    return () => window.clearTimeout(id)
  }, [charging])

  const config = useMemo(
    () => ({
      dockPosition: position,
      iconSize: size,
      magnification: magnify,
      magnifiedSize: magnified,
      animationStyle: style,
      startupAnimation: startup,
      chargingAnimation: true,
      menuBarOpacity: opacity,
      menuBarDate: showDate,
      menuBar24h: hour24,
      wallpaper: wall,
    }),
    [position, size, magnify, magnified, style, startup, opacity, showDate, hour24, wall],
  )
  const k = 0.58 // the stage is smaller than a real screen

  return (
    <section id="customize" className="relative mx-auto max-w-[1240px] px-5 py-24 sm:px-8">
      <div className="max-w-[640px]">
        <h2 className="font-display text-[clamp(2rem,4.6vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">
          Every spring, edge and pixel is a setting.
        </h2>
        <p className="mt-4 text-[17px] leading-relaxed text-white/72">
          This is the real settings model, running in your browser. Change something and watch the dock and the{' '}
          <code className="font-mono text-[#5ff0e6]">config.json</code> the app would write.
        </p>
      </div>

      <div className="mt-12 grid gap-6 lg:grid-cols-[360px_1fr]">
        <div className="glass grid content-start gap-5 rounded-[28px] p-6">
          <Row label="Dock position">
            <Seg label="Dock position" value={position} onChange={(v) => { setPosition(v); setReplay((r) => r + 1) }} options={[['left', 'Left'], ['bottom', 'Bottom'], ['right', 'Right']]} />
          </Row>
          <Row label="Icon size" value={`${size} px`}>
            <Range label="Icon size" min={32} max={72} value={size} onChange={(n) => { setSize(n); setMagnified((m) => Math.max(m, n)) }} />
          </Row>
          <Row label="Magnification" value={magnify ? `${magnified} px` : 'off'}>
            <div className="flex items-center gap-3">
              <button role="switch" aria-checked={magnify} aria-label="Magnification" onClick={() => setMagnify(!magnify)} className={`switch ${magnify ? 'on' : ''}`}><span /></button>
              <Range label="Magnified size" min={Math.max(size, 48)} max={110} value={magnified} onChange={setMagnified} />
            </div>
          </Row>
          <Row label="Animation style">
            <Seg label="Animation style" value={style} onChange={setStyle} options={[['relaxed', 'Relaxed'], ['smooth', 'Smooth'], ['snappy', 'Snappy']]} />
          </Row>
          <Row label="Startup animation">
            <Seg label="Startup animation" value={startup} onChange={(v) => { setStartup(v); setReplay((r) => r + 1) }} options={[['off', 'Off'], ['glide', 'Glide'], ['rise', 'Rise'], ['swift', 'Swift']]} />
          </Row>
          <div className="flex gap-3">
            <button onClick={() => setReplay((r) => r + 1)} className="btn-ghost flex-1">Replay startup</button>
            <button onClick={() => setCharging(true)} className="btn-ghost flex-1">Plug in charger</button>
          </div>
          <hr className="border-white/10" />
          <Row label="Menu bar tint" value={`${opacity}%`}>
            <Range label="Menu bar tint strength" min={10} max={100} value={opacity} onChange={setOpacity} />
          </Row>
          <Row label="Clock">
            <Seg label="Clock format" value={hour24} onChange={setHour24} options={[[null, 'Windows'], [true, '24-hour'], [false, '12-hour']]} />
          </Row>
          <label className="flex min-h-9 cursor-pointer items-center justify-between text-[13.5px] font-medium text-white/90">
            Show the date
            <button role="switch" aria-checked={showDate} aria-label="Show the date" onClick={() => setShowDate(!showDate)} className={`switch ${showDate ? 'on' : ''}`}><span /></button>
          </label>
        </div>

        <div className="grid gap-6">
          <div className="glass relative aspect-[16/10] overflow-hidden rounded-[28px] p-0">
            <div className="absolute inset-0 [&>canvas]:h-full [&>canvas]:w-full">
              <WallpaperCanvas id={wall} w={480} h={300} />
            </div>
            <MenuBar opacity={opacity} showDate={showDate} hour24={hour24} charging={charging} compact className="absolute inset-x-0 top-0 z-10" />
            <div
              className={`absolute z-10 ${position === 'bottom' ? 'inset-x-0 bottom-3' : position === 'left' ? 'inset-y-8 left-3' : 'inset-y-8 right-3'}`}
            >
              <Dock position={position} size={Math.round(size * k)} magnified={Math.round(magnified * k)} magnify={magnify} style={style} startup={startup} replayKey={replay} charging={charging} />
            </div>
            {charging && (
              <div className="glass toast absolute top-10 right-3 z-20 flex items-center gap-2 rounded-2xl px-3.5 py-2 text-[12.5px] text-white">
                <span className="text-[#ffc857]">⚡</span> Charging, 64%
              </div>
            )}
          </div>

          <div className="glass overflow-hidden rounded-[28px]">
            <div className="flex items-center justify-between border-b border-white/10 px-5 py-3 text-[12.5px] text-white/60">
              <span className="font-mono">%APPDATA%\Whirlpool\config.json</span>
              <span className="flex items-center gap-2"><span className="h-1.5 w-1.5 rounded-full bg-[#5ff0e6] shadow-[0_0_8px_#5ff0e6]" />live</span>
            </div>
            <pre className="max-h-[260px] overflow-auto p-5 font-mono text-[12.5px] leading-6 text-white/85" tabIndex={0} aria-label="Live configuration">
              {Object.entries(config).map(([key, value]) => (
                <div key={key}>
                  <span className="text-white/45">  "{key}": </span>
                  <span className={typeof value === 'string' ? 'text-[#9ff0e6]' : typeof value === 'number' ? 'text-[#ffc857]' : 'text-[#ff9ec2]'}>{JSON.stringify(value)}</span>,
                </div>
              ))}
            </pre>
          </div>
        </div>
      </div>
      <p className="sr-only">Wallpaper in the preview: {wall}. <button onClick={() => setWall('abyss')}>Reset</button></p>
      <WallpaperPicker current={wall} onPick={setWall} />
    </section>
  )
}

function WallpaperPicker({ current, onPick }: { current: string; onPick: (id: string) => void }) {
  return (
    <div id="wallpapers" className="mt-24 scroll-mt-24">
      <div className="flex flex-wrap items-end justify-between gap-6">
        <div className="max-w-[620px]">
          <h2 className="font-display text-[clamp(2rem,4.6vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">
            {PRESETS.length} wallpapers, drawn for your screen.
          </h2>
          <p className="mt-4 text-[17px] leading-relaxed text-white/72">
            Nothing to download: each one is a few numbers, rendered sharp at your exact resolution and dithered so gradients never band.
            Pick one and the stage above changes. In the app, one click sets your desktop.
          </p>
        </div>
        <button
          className="btn-ghost"
          onClick={() => {
            const others = PRESETS.filter((p) => p.id !== current)
            onPick(others[Math.floor(Math.random() * others.length)].id)
          }}
        >
          Surprise me
        </button>
      </div>
      <div className="mt-8 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
        {PRESETS.map((p) => (
          <button
            key={p.id}
            onClick={() => onPick(p.id)}
            aria-pressed={current === p.id}
            className={`group glass overflow-hidden rounded-2xl p-1.5 text-left transition-transform duration-200 hover:-translate-y-0.5 focus-visible:outline-2 focus-visible:outline-[#5ff0e6] ${
              current === p.id ? 'ring-2 ring-[#5ff0e6]' : ''
            }`}
          >
            <div className="aspect-[8/5] overflow-hidden rounded-xl">
              <div className="h-full w-full transition-transform duration-300 group-hover:scale-[1.03] [&>canvas]:h-full [&>canvas]:w-full">
                <WallpaperCanvas id={p.id} w={192} h={120} />
              </div>
            </div>
            <div className="px-1.5 pt-2 pb-1 text-[13px] font-medium text-white/90">{p.name}</div>
          </button>
        ))}
      </div>
    </div>
  )
}
