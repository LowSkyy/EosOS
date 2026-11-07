"""Genera l'icona sorgente di EosOS (assets/icon-source.png, 1024x1024).
Disegno originale: un sole che sorge (Eos = alba) su fondo scuro. Nessun asset di terzi.
Poi: npm run tauri icon assets/icon-source.png"""
from PIL import Image, ImageDraw, ImageFilter

S = 1024
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

# Fondo: quadrato arrotondato con gradiente verticale blu notte -> viola.
bg = Image.new("RGBA", (S, S))
px = bg.load()
top, bot = (22, 24, 46), (70, 44, 96)
for y in range(S):
    t = y / (S - 1)
    c = tuple(int(top[i] + (bot[i] - top[i]) * t) for i in range(3)) + (255,)
    for x in range(S):
        px[x, y] = c
mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle((0, 0, S - 1, S - 1), radius=230, fill=255)
img.paste(bg, (0, 0), mask)

# Alone del sole.
glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
cx, cy, r = S // 2, 640, 250
ImageDraw.Draw(glow).ellipse((cx - r - 90, cy - r - 90, cx + r + 90, cy + r + 90), fill=(255, 170, 90, 120))
glow = glow.filter(ImageFilter.GaussianBlur(60))
img = Image.alpha_composite(img, glow)

# Sole (semidisco sopra l'orizzonte) con gradiente arancio -> giallo.
sun = Image.new("RGBA", (S, S), (0, 0, 0, 0))
sp = sun.load()
for y in range(cy - r, cy + 1):
    t = (y - (cy - r)) / r
    col = (255, int(214 - 70 * t), int(120 - 50 * t), 255)
    for x in range(cx - r, cx + r + 1):
        if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
            sp[x, y] = col
img = Image.alpha_composite(img, sun)

# Orizzonte: tre linee che si accorciano.
d = ImageDraw.Draw(img)
for i, (w, y) in enumerate([(560, 690), (400, 750), (240, 810)]):
    d.rounded_rectangle((cx - w // 2, y, cx + w // 2, y + 28), radius=14, fill=(245, 245, 250, 255 - i * 45))

# Riapplica la maschera per mantenere gli angoli trasparenti.
out = Image.new("RGBA", (S, S), (0, 0, 0, 0))
out.paste(img, (0, 0), mask)
out.save("assets/icon-source.png")
print("ok")
