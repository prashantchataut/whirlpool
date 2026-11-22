"""Design renders for the 2.0 screens (README and site). Not captures of a running Windows session.
Wallpapers come from the real renderer (`cargo run -p whirlpool-core --example wallpaper_sheet`)."""
import math, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter

sheet = Image.open(sys.argv[1]).convert("RGB")  # wallpaper_sheet output as PPM/PNG, 6 columns of 320x200 tiles
OUT = sys.argv[2]
def tile(i, size):
    cx, cy = (i % 6) * 320, (i // 6) * 200
    return sheet.crop((cx, cy, cx + 320, cy + 200)).resize(size, Image.LANCZOS)
def font(sz, bold=False):
    for p in ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",):
        try: return ImageFont.truetype(p, sz)
        except OSError: pass
    return ImageFont.load_default(sz)

# ---- medallion, same maths as apps/dock/src/charge.rs
GLYPH = {'0':"###|# #|# #|# #|###",'1':" # |## | # | # |###",'2':"###|  #|###|#  |###",'3':"###|  #|###|  #|###",'4':"# #|# #|###|  #|  #",'5':"###|#  |###|  #|###",'6':"###|#  |###|# #|###",'7':"###|  #|  #|  #|  #",'8':"###|# #|###|# #|###",'9':"###|# #|###|  #|###",'%':"# #|  #| # |#  |# #"}
BOLT = [(0.08,-0.52),(-0.24,0.06),(-0.04,0.06),(-0.12,0.44),(0.24,-0.12),(0.04,-0.12)]
def medal(px, pct, ss=3):
    acc = np.zeros((px, px, 4), np.float32)
    text = f"{pct}%"
    for sy in range(ss):
        for sx in range(ss):
            xs = ((np.arange(px)[None, :] + (sx + .5) / ss) / px) * 2 - 1
            ys = ((np.arange(px)[:, None] + (sy + .5) / ss) / px) * 2 - 1
            u = np.broadcast_to(xs, (px, px)); v = np.broadcast_to(ys, (px, px))
            r = np.sqrt(u*u + v*v); out = np.zeros((px, px, 4), np.float32)
            def paint(mask, col, a):
                col = np.asarray(col, np.float32)
                a = np.where(mask, a, 0.0)[..., None]
                out[..., :3] = col * a + out[..., :3] * (1 - a)
                out[..., 3:] = a + out[..., 3:] * (1 - a)
            light = np.clip(.5 - .35*(u+v), 0, 1)
            c0 = np.array([.02,.10,.14]); c1 = np.array([.07,.26,.33])
            body = c0 + (c1 - c0) * (light*.8)[..., None]
            gl = np.clip(1 - np.sqrt((u+.15)**2 + (v+.6)**2)/.62, 0, 1)
            gl = gl*gl*(3-2*gl)*.16
            body = body + (1 - body) * gl[..., None]
            m = r < .8
            out[..., :3] = np.where(m[..., None], body*.95, 0); out[..., 3] = np.where(m, .95, 0)
            paint((r >= .8) & (r < .83), [1,1,1], .10)
            ring = (r >= .86) & (r < .97)
            paint(ring, [1,1,1], .14)
            turn = (np.arctan2(u, -v) % (2*math.pi)) / (2*math.pi); f = pct/100
            t = np.clip(turn / max(f, .01), 0, 1)
            lo = np.array([.09,.63,.77]); mid = np.array([.47,.94,.94]); hi = np.array([.86,1,.98])
            col = np.where((t < .6)[..., None], lo + (mid-lo)*(t/.6)[..., None], mid + (hi-mid)*((t-.6)/.4)[..., None])
            paint(ring & (turn <= f), col, 1.0)
            # bolt
            vv = v + .14; inside = np.zeros((px, px), bool); prev = BOLT[-1]
            for p in BOLT:
                cond = ((p[1] > vv) != (prev[1] > vv))
                with np.errstate(divide='ignore', invalid='ignore'):
                    xint = (prev[0]-p[0]) * (vv-p[1]) / (prev[1]-p[1]) + p[0]
                inside ^= cond & (u < xint); prev = p
            paint(inside & m, [.9,1,.98], 1.0)
            cell = .05; cols = len(text)*4 - 1; x0 = -cols*cell/2; y0 = .38
            gx = np.floor((u-x0)/cell).astype(int); gy = np.floor((v-y0)/cell).astype(int)
            ink = np.zeros((px, px), bool)
            for gi, ch in enumerate(text):
                rows = GLYPH[ch].split('|')
                for ry in range(5):
                    for rx in range(3):
                        if rows[ry][rx] == '#': ink |= (gx == gi*4+rx) & (gy == ry)
            paint(ink & m, [.88,.98,.97], 1.0)
            acc += out
    acc /= ss*ss
    a = acc[..., 3:4]; rgb = np.where(a > 0, acc[..., :3] / np.maximum(a, 1e-6), 0)
    return Image.fromarray((np.dstack([rgb, a]).clip(0,1)*255).astype(np.uint8), "RGBA")

def glass_bar(img, box, radius, text_left, text_right, alpha=150):
    x0, y0, x1, y1 = box
    region = img.crop(box).filter(ImageFilter.GaussianBlur(10)).convert("RGBA")
    tint = Image.new("RGBA", region.size, (6, 28, 36, alpha)); region = Image.alpha_composite(region, tint)
    mask = Image.new("L", region.size, 0); ImageDraw.Draw(mask).rounded_rectangle((0, 0, region.size[0]-1, region.size[1]-1), radius, fill=255)
    base = img.convert("RGBA"); base.paste(region, (x0, y0), mask)
    d = ImageDraw.Draw(base); h = y1 - y0; f = font(int(h*.45)); fb = font(int(h*.45), True)
    d.text((x0+18, y0+h/2), text_left, font=fb, fill=(235,250,250), anchor="lm")
    d.text((x1-18, y0+h/2), text_right, font=f, fill=(235,250,250), anchor="rm")
    d.rounded_rectangle((x0, y0, x1-1, y1-1), radius, outline=(255,255,255,40), width=1)
    return base.convert("RGB")

W, H = 1600, 1000
# 1. charging
bg = tile(0, (W, H)); bg = glass_bar(bg, (0, 0, W, 34), 0, "Whirlpool      File   Edit   View", "Wi-Fi    82%    Mon 9:41")
over = Image.new("RGBA", (W, H), (0, 0, 0, 0)); cx, cy = W//2, 190
yy, xx = np.mgrid[0:H, 0:W]; rr = np.sqrt((xx-cx)**2 + (yy-cy)**2)
glow = np.exp(-(rr/150)**2 / .5) * .5
layer = np.zeros((H, W, 4), np.float32); layer[..., 0] = .24; layer[..., 1] = .86; layer[..., 2] = .90; layer[..., 3] = glow
for rad, a in ((235, .55), (330, .35), (430, .18)):
    ring = np.exp(-((rr - rad)/5.0)**2) * a; wash = .05*np.clip(1-rr/rad, 0, 1)*(rr < rad)
    al = np.clip(ring + wash, 0, 1)
    layer[..., 0] = layer[..., 0]*(1-al) + .59*al; layer[..., 1] = layer[..., 1]*(1-al) + .94*al; layer[..., 2] = layer[..., 2]*(1-al) + .92*al
    layer[..., 3] = np.clip(layer[..., 3] + al, 0, 1)
over = Image.fromarray((layer.clip(0,1)*255).astype(np.uint8), "RGBA")
canvas = Image.alpha_composite(bg.convert("RGBA"), over)
m = medal(300, 82); canvas.alpha_composite(m, (cx-150, cy-150))
canvas.convert("RGB").save(f"{OUT}/charging.png", optimize=True)

# 2. menu bar: docked vs floating
def panel(floating):
    im = tile(18, (W, 500)) if False else tile(19, (W, 500))
    if floating: im = glass_bar(im, (60, 22, W-60, 22+44), 18, "Whirlpool      File   Edit   View", "Bluetooth    Wi-Fi    82%    Mon 9:41")
    else: im = glass_bar(im, (0, 0, W, 40), 0, "Whirlpool      File   Edit   View", "Bluetooth    Wi-Fi    82%    Mon 9:41")
    d = ImageDraw.Draw(im); d.text((24, 470), "Floating: side margin 60, gap 22, radius 18, large" if floating else "Docked: edge to edge, regular", font=font(20, True), fill=(255,255,255), anchor="ls")
    return im
both = Image.new("RGB", (W, 1010)); both.paste(panel(False), (0, 0)); both.paste(panel(True), (0, 510))
both.save(f"{OUT}/menubar-styles.png", optimize=True)

# 3. the 30 new wallpapers, from the real renderer
new = sheet.crop((0, 800, 1920, 1600)).resize((1440, 600), Image.LANCZOS)
new.save(f"{OUT}/wallpapers-new.jpg", quality=88, optimize=True)
print("ok")
