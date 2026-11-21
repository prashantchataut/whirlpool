import { useState } from 'react'

const REPO = 'https://github.com/prashantchataut/whirlpool'

function Command({ label, text }: { label: string; text: string }) {
  const [done, setDone] = useState(false)
  return (
    <div className="glass overflow-hidden rounded-2xl">
      <div className="flex items-center justify-between border-b border-white/10 px-4 py-2 text-[12.5px] text-white/60">
        {label}
        <button
          className="rounded-md px-2.5 py-1 font-medium text-[#5ff0e6] hover:bg-white/10 focus-visible:outline-2 focus-visible:outline-[#5ff0e6]"
          onClick={() => navigator.clipboard?.writeText(text).then(() => { setDone(true); window.setTimeout(() => setDone(false), 1600) })}
        >
          {done ? 'Copied' : 'Copy'}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 font-mono text-[12.5px] leading-6 text-white/90" tabIndex={0}>{text}</pre>
    </div>
  )
}

const FACTS: [string, string][] = [
  ['Built in public', 'Every release is built by GitHub Actions from a tagged commit. Nobody uploads a binary from a laptop.'],
  ['Provenance attested', 'Each file carries a signed build attestation proving which workflow and commit made it.'],
  ['Checksums', 'SHA-256 sums ship with every release, and the installer refuses a download that does not match.'],
  ['No admin rights', 'The manifest asks for the invoking user only. Whirlpool never needs elevation.'],
  ['No telemetry', 'The only network calls are the optional update check and AI-plan usage, both switchable.'],
  ['A way back', 'If the dock exits, a watchdog returns the Windows taskbar. whirlpool.exe --restore does it by hand.'],
]

export default function Trust() {
  return (
    <section id="trust" className="mx-auto max-w-[1240px] scroll-mt-24 px-5 py-24 sm:px-8">
      <div className="grid gap-12 lg:grid-cols-[1fr_1.1fr]">
        <div>
          <h2 className="font-display text-[clamp(2rem,4.6vw,3.4rem)] leading-[1.02] font-semibold tracking-tight text-white">
            Windows may call it unknown. You can check it.
          </h2>
          <p className="mt-5 max-w-[560px] text-[17px] leading-relaxed text-white/72">
            SmartScreen judges a new download by its signature and by how many people already run it. A young open-source app has neither yet.
            The warning means unknown, not malicious, so we made Whirlpool easy to verify and we say plainly what is still unsigned.
          </p>
          <ul className="mt-8 grid gap-x-8 gap-y-5 sm:grid-cols-2">
            {FACTS.map(([t, d]) => (
              <li key={t}>
                <div className="flex items-center gap-2 font-medium text-white"><span aria-hidden className="h-1.5 w-1.5 rounded-full bg-[#5ff0e6] shadow-[0_0_8px_#5ff0e6]" />{t}</div>
                <p className="mt-1.5 text-[14.5px] leading-relaxed text-white/65">{d}</p>
              </li>
            ))}
          </ul>
        </div>
        <div className="grid content-start gap-4">
          <Command label="Install, with the checksum checked for you" text={`irm ${REPO.replace('github.com', 'raw.githubusercontent.com')}/main/scripts/install.ps1 | iex`} />
          <Command label="Compare the hash with SHA256SUMS.txt" text="Get-FileHash .\\whirlpool.exe -Algorithm SHA256" />
          <Command label="Verify where it was built (GitHub CLI)" text={`gh attestation verify .\\whirlpool.exe --repo prashantchataut/whirlpool`} />
          <p className="text-[14px] leading-relaxed text-white/60">
            Still warned? Choose <em>More info, then Run anyway</em>; Windows remembers it. Code signing is wired into the release workflow and switches on when a certificate is added; read{' '}
            <a className="text-[#5ff0e6] underline underline-offset-4" href={`${REPO}/blob/main/docs/TRUST.md`}>the full trust notes</a>.
          </p>
        </div>
      </div>
    </section>
  )
}
