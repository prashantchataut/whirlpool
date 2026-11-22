import { useState } from 'react'

type Style = 'surge' | 'halo' | 'edge' | 'tide'

const STYLES: { id: Style; name: string; note: string }[] = [
  { id: 'surge', name: 'Surge', note: 'A small medallion drops in under the menu bar. Two thin rings spread out.' },
  { id: 'halo', name: 'Halo', note: 'A slim glass capsule with a bolt, the level and a thin meter.' },
  { id: 'edge', name: 'Edge glow', note: 'The edges of the screen glow like liquid glass, coloured by the level.' },
  { id: 'tide', name: 'Tide', note: 'A line of light runs along the top edge up to the battery level.' },
]

function tone(p: number) {
  return p <= 20 ? '255,128,30' : p < 60 ? '255,204,46' : '72,230,122'
}

function Stage({ style, percent, run }: { style: Style; percent: number; run: number }) {
  const c = tone(percent)
  return (
    <div key={run} className="wp-stage relative aspect-[16/10] overflow-hidden rounded-2xl border border-white/15 bg-[#041a24]">
      <div className="absolute inset-x-0 top-0 h-5 bg-white/5" />
      {style === 'surge' && (
        <>
          {[0, 1].map((i) => (
            <span key={i} className="wp-ring absolute top-[8%] left-1/2 h-24 w-24 -translate-x-1/2 rounded-full border border-[#8ff5ee]/60" style={{ animationDelay: `${0.3 + i * 0.55}s` }} />
          ))}
          <div className="wp-drop absolute top-[9%] left-1/2 flex h-16 w-16 -translate-x-1/2 flex-col items-center justify-center rounded-full border border-white/20 bg-[#0a2a35] text-[#bffbf5] shadow-[0_0_30px_rgba(95,240,230,.3)]">
            <svg viewBox="0 0 24 24" className="h-5 w-5 fill-current"><path d="M13 2 4 14h6l-1 8 9-12h-6z" /></svg>
            <span className="font-mono text-[10px]">{percent}%</span>
          </div>
        </>
      )}
      {style === 'halo' && (
        <div className="wp-drop absolute top-[10%] left-1/2 flex h-10 w-40 -translate-x-1/2 items-center gap-3 rounded-full border border-white/20 bg-[#0a2430]/90 px-3 text-white">
          <svg viewBox="0 0 24 24" className="h-4 w-4 shrink-0" style={{ fill: `rgb(${c})` }}><path d="M13 2 4 14h6l-1 8 9-12h-6z" /></svg>
          <span className="font-mono text-[12px]">{percent}%</span>
          <span className="relative ml-auto h-1 w-14 rounded-full bg-white/15"><span className="absolute inset-y-0 left-0 rounded-full" style={{ width: `${percent}%`, background: `rgb(${c})` }} /></span>
        </div>
      )}
      {style === 'edge' && (
        <div className="wp-edge absolute inset-0" style={{ boxShadow: `inset 0 0 70px 6px rgba(${c},.55), inset 0 0 14px 1px rgba(${c},.9), inset 0 0 0 1px rgba(255,255,255,.35)`, borderRadius: 16 }} />
      )}
      {style === 'tide' && (
        <div className="absolute inset-x-0 top-0">
          <span className="wp-tide absolute top-0 left-0 block h-[3px] origin-left" style={{ width: `${percent}%`, background: `linear-gradient(90deg, rgba(${c},0), rgba(${c},1) 70%, #fff)`, boxShadow: `0 4px 18px 1px rgba(${c},.7)` }} />
        </div>
      )}
      <style>{`
        @keyframes wp-drop { 0%{opacity:0;transform:translate(-50%,-16px) scale(.9)} 20%,80%{opacity:1;transform:translate(-50%,0) scale(1)} 100%{opacity:0;transform:translate(-50%,-14px) scale(.96)} }
        @keyframes wp-ring { 0%{opacity:0;transform:translateX(-50%) scale(.6)} 10%{opacity:.5} 100%{opacity:0;transform:translateX(-50%) scale(2.6)} }
        @keyframes wp-edge { 0%{opacity:0} 22%{opacity:1} 45%{opacity:.7} 70%{opacity:1} 100%{opacity:0} }
        @keyframes wp-tide { 0%{transform:scaleX(0);opacity:0} 10%{opacity:1} 45%{transform:scaleX(1)} 85%{opacity:1} 100%{transform:scaleX(1);opacity:0} }
        .wp-drop{animation:wp-drop 3.4s cubic-bezier(.2,.9,.3,1) both}
        .wp-ring{animation:wp-ring 2s cubic-bezier(.2,.9,.3,1) both;opacity:0}
        .wp-edge{animation:wp-edge 3.6s ease-in-out both}
        .wp-tide{animation:wp-tide 3.4s cubic-bezier(.2,.9,.3,1) both}
        @media (prefers-reduced-motion: reduce){.wp-drop,.wp-ring,.wp-edge,.wp-tide{animation-duration:.01s}}
      `}</style>
    </div>
  )
}

export default function Charging() {
  const [style, setStyle] = useState<Style>('edge')
  const [percent, setPercent] = useState(72)
  const [run, setRun] = useState(0)
  const current = STYLES.find((s) => s.id === style)!
  return (
    <section id="charging" className="mx-auto mt-28 max-w-[1240px] px-5 sm:px-8">
      <h2 className="font-display text-[clamp(2rem,4.4vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">Plug in. The screen answers.</h2>
      <p className="mt-4 max-w-[560px] text-[17px] leading-relaxed text-white/70">Four charging animations, or none. Green above 60%, yellow in the middle, orange at 20% and below. They draw once and are gone in about three seconds.</p>
      <div className="glass mt-9 grid gap-8 rounded-[28px] p-5 sm:p-8 lg:grid-cols-[1.4fr_1fr]">
        <Stage style={style} percent={percent} run={run} />
        <div className="flex flex-col justify-between gap-6">
          <div role="radiogroup" aria-label="Charging animation" className="grid grid-cols-2 gap-2">
            {STYLES.map((s) => (
              <button key={s.id} role="radio" aria-checked={style === s.id} onClick={() => { setStyle(s.id); setRun((r) => r + 1) }} className={`min-h-11 rounded-xl border px-3 text-[14px] font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[#5ff0e6] ${style === s.id ? 'border-[#5ff0e6] bg-[#5ff0e6]/15 text-white' : 'border-white/15 text-white/70 hover:bg-white/10'}`}>{s.name}</button>
            ))}
          </div>
          <p className="text-[15px] leading-relaxed text-white/70">{current.note}</p>
          <label className="block text-[13px] text-white/60">Battery level <span className="font-mono text-white">{percent}%</span>
            <input type="range" min={5} max={100} value={percent} onChange={(e) => { setPercent(+e.target.value) }} onPointerUp={() => setRun((r) => r + 1)} onKeyUp={() => setRun((r) => r + 1)} className="mt-2 block w-full accent-[#5ff0e6]" />
          </label>
          <button onClick={() => setRun((r) => r + 1)} className="btn-ghost !min-h-10 self-start !text-[14px]">Play again</button>
        </div>
      </div>
      <p className="mt-3 text-[12.5px] text-white/45">A web preview of the motion. The real animations are drawn by Windows' compositor.</p>
    </section>
  )
}
