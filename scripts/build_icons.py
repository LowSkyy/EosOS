"""Genera le icone .ico di cartelle e file (src-tauri/iconpack) dai disegni di src/components/iconart.tsx.

Uso: python scripts/build_icons.py   (serve Chrome per rendere gli SVG e Pillow per tagliare e salvare)
"""
import json, os, subprocess, sys, tempfile
from PIL import Image

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
out_dir = os.path.join(root, "src-tauri", "iconpack")
os.makedirs(out_dir, exist_ok=True)
tmp = tempfile.mkdtemp()
sheet = os.path.join(tmp, "sheet.html")
png = os.path.join(tmp, "sheet.png")
env = dict(os.environ, EOSOS_ICON_SHEET=sheet)
subprocess.run(["npx", "vitest", "run", "src/components/iconsheet.test.tsx"], cwd=root, env=env, shell=True, check=True, stdout=subprocess.DEVNULL)
chrome = next(p for p in [r"C:\Program Files\Google\Chrome\Application\chrome.exe", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"] if os.path.exists(p))
subprocess.run([chrome, "--headless", "--disable-gpu", "--hide-scrollbars", "--default-background-color=00000000", f"--screenshot={png}", "--window-size=3072,2048", "file:///" + sheet.replace("\\", "/")], check=True, stderr=subprocess.DEVNULL)
img = Image.open(png).convert("RGBA")
# L'ordine delle celle e' quello della pagina: prima tutte le chiare, poi tutte le scure, nell'ordine di FILE_ART.
html = open(sheet, encoding="utf-8").read()
import re
names = re.findall(r'data-n="([^"]+)"', html)
for i, name in enumerate(names):
    col, row = i % 6, i // 6
    cell = img.crop((col * 512, row * 512, col * 512 + 512, row * 512 + 512))
    cell.save(os.path.join(out_dir, f"{name}.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
print(f"{len(names)} icone in {out_dir}")
