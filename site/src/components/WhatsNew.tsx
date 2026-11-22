import charging from '../assets/charging.png'
import menubar from '../assets/menubar-styles.png'
import wallpapers from '../assets/wallpapers-new.jpg'

const REPO = 'https://github.com/prashantchataut/whirlpool'
const SETUP = `${REPO}/releases/latest/download/Whirlpool-Setup.exe`

function Shot({ src, alt, caption }: { src: string; alt: string; caption: string }) {
  return (
    <figure className="glass overflow-hidden rounded-[26px] p-2">
      <img src={src} alt={alt} loading="lazy" className="w-full rounded-[20px]" />
      <figcaption className="px-3 pt-3 pb-2 font-mono text-[12px] tracking-wide text-white/50 uppercase">{caption}</figcaption>
    </figure>
  )
}

function Row({ k, title, children, shot, flip }: { k: string; title: string; children: React.ReactNode; shot: React.ReactNode; flip?: boolean }) {
  return (
    <div className="grid items-center gap-10 lg:grid-cols-2">
      <div className={flip ? 'lg:order-2' : ''}>
        <div className="font-mono text-[12.5px] tracking-[0.18em] text-[#5ff0e6] uppercase">{k}</div>
        <h3 className="mt-3 font-display text-[clamp(1.7rem,3vw,2.5rem)] leading-tight font-semibold text-white">{title}</h3>
        <div className="mt-4 max-w-[500px] space-y-3 text-[16.5px] leading-relaxed text-white/72">{children}</div>
      </div>
      <div className={flip ? 'lg:order-1' : ''}>{shot}</div>
    </div>
  )
}

export default function WhatsNew() {
  return (
    <section id="new" className="mx-auto max-w-[1240px] scroll-mt-24 px-5 py-24 sm:px-8">
      <div className="max-w-[640px]">
        <div className="font-mono text-[12.5px] tracking-[0.18em] text-[#5ff0e6] uppercase">Whirlpool 2.0</div>
        <h2 className="mt-3 font-display text-[clamp(2rem,4.4vw,3.4rem)] leading-[1.02] font-semibold tracking-[-0.02em] text-white">One program. Installs like one.</h2>
        <p className="mt-4 text-[17px] leading-relaxed text-white/70">Settings now lives inside whirlpool.exe, so there is nothing second to download. A proper Setup puts it in Apps and Features, and the Start menu.</p>
      </div>

      <div className="mt-14 space-y-24">
        <Row k="Install" title="Setup, Start menu, uninstall" shot={
          <div className="glass rounded-[26px] p-7">
            <ol className="space-y-4 font-mono text-[14px] text-white/80">
              {['Download Whirlpool-Setup.exe', 'Run it: no administrator rights', 'Pick “start when I sign in”', 'Whirlpool and Whirlpool Settings appear in Start'].map((s, i) => (
                <li key={s} className="flex gap-4"><span className="text-[#5ff0e6] tabular-nums">0{i + 1}</span>{s}</li>
              ))}
            </ol>
            <a href={SETUP} className="btn-primary mt-7 inline-flex">Download Setup</a>
          </div>
        }>
          <p>Right-click the dock for Settings, or start Whirlpool again while it runs and Settings opens. It is the same executable, started with <code className="rounded bg-white/10 px-1.5 py-0.5 text-[14px]">--settings</code>, and it exits the moment you close it, so it costs no memory afterwards.</p>
          <p>Prefer no installer? The portable zip and a checksum-verified one-line script are on the release page.</p>
        </Row>

        <Row k="Charging" title="A charging animation made for it" flip shot={<Shot src={charging} alt="A glass medallion with a bolt and a battery ring, rings of water spreading from it" caption="Design render · Surge style · 82%" />}>
          <p>Plug in and a glass medallion drops under the menu bar. Its ring fills to your real battery level, a bolt pulses, and rings of water spread out. Pick Surge or the calmer Halo, or turn it off.</p>
          <p>The compositor runs every keyframe. Nothing is drawn per frame, and the window is gone in under four seconds.</p>
        </Row>

        <Row k="Menu bar" title="Docked, or floating where you want it" shot={<Shot src={menubar} alt="The menu bar docked edge to edge, and floating as a rounded bar inset from the screen edges" caption="Design render · docked and floating" />}>
          <p>Keep the bar edge to edge, or float it as a rounded pill. Choose compact, regular or large, then set the side margin, the top gap and the corner radius. Windows keep clear of it either way.</p>
        </Row>

        <Row k="Wallpapers" title="54 of them, all drawn at your exact resolution" flip shot={<Shot src={wallpapers} alt="Thirty new wallpapers: aurora, topographic maps, a synthwave grid and the dithered vortex" caption="Rendered by the app's own renderer" />}>
          <p>Six new families join the originals: aurora, contour maps, a synthwave grid and the dithered vortex in six palettes. Nothing is downloaded and nothing is scaled. Each is drawn for your screen.</p>
        </Row>
      </div>
    </section>
  )
}
