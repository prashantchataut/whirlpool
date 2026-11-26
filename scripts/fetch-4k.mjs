// Fetches 4K pictures and previews from 4kwallpapers.com with a real Edge window (Playwright) into the
// studio's cache, for the capture runner only. The site puts a bot check in front of plain HTTP clients, so
// the program's own download is refused on a hosted runner; Edge passes it or does not, and the log says which.
// Usage: node scripts/fetch-4k.mjs OUT_DIR [--previews | --previews-only] [--all | ID...]
import { chromium } from 'playwright-core';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [out, ...rest] = process.argv.slice(2);
const previews = rest.includes('--previews') || rest.includes('--previews-only');
const only = rest.includes('--previews-only');
const catalog = readFileSync(new URL('../crates/whirlpool-core/src/studio.rs', import.meta.url), 'utf8');
const pages = new Map([...catalog.matchAll(/entry\(\s*(\d+),\s*Category::\w+,\s*"[^"]*",\s*"([^"]+)"/g)].map((m) => [Number(m[1]), m[2]]));
const ids = rest.includes('--all') ? [...pages.keys()] : rest.filter((a) => /^\d+$/.test(a)).map(Number);
mkdirSync(join(out, 'previews'), { recursive: true });

const browser = await chromium.launch({ channel: 'msedge', headless: false });
const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, locale: 'en-US' });
const page = await ctx.newPage();
const log = (m) => console.log(`${new Date().toISOString().slice(11, 19)} ${m}`);
let ok = 0;

async function cleared() {
  for (let i = 0; i < 15; i++) {
    if (!/just a moment|attention required/i.test(await page.title())) return true;
    await page.waitForTimeout(3000);
  }
  return false;
}

for (const id of ids) {
  const path = pages.get(id);
  if (!path) { log(`id ${id}: not in the catalogue`); continue; }
  const slug = path.split('/').pop().replace(/\.html$/, '').replace(new RegExp(`-${id}$`), '');
  try {
    await page.goto(`https://4kwallpapers.com/${path}`, { waitUntil: 'domcontentloaded', timeout: 60000 });
    if (!(await cleared())) { log(`id ${id}: bot check not passed`); continue; }
    if (previews) {
      const r = await ctx.request.get(`https://4kwallpapers.com/images/walls/thumbs_2t/${id}.jpg`, { headers: { Referer: page.url() } });
      const body = await r.body();
      if (r.status() === 200 && body[0] === 0xff) { writeFileSync(join(out, 'previews', `${id}.jpg`), body); log(`id ${id}: preview ${body.length} bytes`); }
      else log(`id ${id}: preview status ${r.status()}`);
    }
    if (only) continue;
    const link = await page.evaluate(() => [...document.querySelectorAll('a')].map((a) => a.href).find((h) => h.includes('3840x2160')));
    if (!link) { log(`id ${id}: no 3840x2160 link on the page`); continue; }
    const r = await ctx.request.get(link, { headers: { Referer: page.url() }, timeout: 120000 });
    const body = await r.body();
    if (r.status() === 200 && body[0] === 0xff && body[1] === 0xd8) {
      writeFileSync(join(out, `${slug}-3840x2160-${id}.jpg`), body);
      ok++; log(`id ${id}: 4K ${(body.length / 1048576).toFixed(1)} MB -> ${slug}-3840x2160-${id}.jpg`);
    } else log(`id ${id}: download status ${r.status()} (${body.length} bytes, not a picture)`);
  } catch (e) { log(`id ${id}: ${String(e).slice(0, 160)}`); }
}
await browser.close();
log(`fetched ${ok} of ${ids.length}`);
process.exit(0);
