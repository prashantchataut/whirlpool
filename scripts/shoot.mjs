// Screenshots of the built site for the README: `node scripts/shoot.mjs <url> <outDir>`.
import { chromium } from 'playwright'
import { mkdirSync } from 'node:fs'
const [url = 'http://localhost:4173/whirlpool/', out = 'docs/previews'] = process.argv.slice(2)
mkdirSync(out, { recursive: true })
const browser = await chromium.launch()
const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 })
const errors = []
page.on('pageerror', (e) => errors.push(String(e)))
await page.goto(url, { waitUntil: 'networkidle' })
await page.waitForTimeout(2200)
await page.screenshot({ path: `${out}/site-hero.png` })
await page.hover('[aria-label="Browser"] >> nth=0')
await page.waitForTimeout(600)
await page.screenshot({ path: `${out}/site-hero-hover.png`, clip: { x: 640, y: 120, width: 800, height: 700 } })
for (const id of ['customize', 'wallpapers', 'screens', 'trust']) {
  const el = page.locator(`#${id}`)
  await el.scrollIntoViewIfNeeded()
  await page.waitForTimeout(500)
  await el.screenshot({ path: `${out}/site-${id}.png` })
}
await page.screenshot({ path: `${out}/site-full.png`, fullPage: true })
const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2 })
await phone.goto(url, { waitUntil: 'networkidle' })
await phone.waitForTimeout(1800)
await phone.screenshot({ path: `${out}/site-mobile.png` })
const overflow = await phone.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)
console.log('console errors:', errors, 'mobile horizontal overflow px:', overflow)
await browser.close()
