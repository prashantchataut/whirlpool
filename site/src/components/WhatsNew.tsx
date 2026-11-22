const ITEMS: [string, string][] = [
  ['Four charging animations', 'Surge, Halo, Edge glow and Tide. The edge glow follows the battery: green, yellow or orange. The crab is off by default and never appears for the charger.'],
  ['A menu bar that is yours', 'Docked or floating, with its own colour, width, alignment, radius, size and an optional edge highlight.'],
  ['66 wallpapers', 'Tidal waves, halftone spheres, ripples and silk join the dithered vortex, drawn at the exact size of your screen.'],
  ['More motion', 'Five animation feels and six startup animations, chosen from drop-downs that fit.'],
  ['One program, one installer', 'Settings lives inside whirlpool.exe. If it ever cannot open, it now tells you why and where the log is.'],
  ['Smaller and lighter', 'Native TLS replaces a bundled crypto library, and the build is optimised for size. Nothing runs while the dock is idle.'],
]

import sheet from '../assets/wallpapers-3.jpg'

export default function WhatsNew() {
  return (
    <section id="new" className="mx-auto mt-28 max-w-[1240px] px-5 sm:px-8">
      <h2 className="font-display text-[clamp(2rem,4.4vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">New in 3.0</h2>
      <ul className="mt-9 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {ITEMS.map(([t, d]) => (
          <li key={t} className="glass rounded-[24px] p-6">
            <h3 className="font-display text-[19px] font-semibold text-white">{t}</h3>
            <p className="mt-2 text-[14.5px] leading-relaxed text-white/70">{d}</p>
          </li>
        ))}
      </ul>
      <figure className="glass mt-4 overflow-hidden rounded-[24px] p-2">
        <img src={sheet} alt="Twelve new wallpapers: tidal waves, halftone spheres, ripples and silk, rendered by the app" className="w-full rounded-[18px]" loading="lazy" />
        <figcaption className="px-3 py-2 text-[12.5px] text-white/50">The twelve new wallpapers, rendered by the app's own engine.</figcaption>
      </figure>
    </section>
  )
}
