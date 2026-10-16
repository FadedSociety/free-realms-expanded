"""Build labelled contact sheets of the client's icons and UI pictures.

Selects images from an unpacked client (fl_pack output), keeps one copy of each
icon (the largest of its size variants, e.g. _32/_64/_128), decodes DDS/TGA/PNG
with Pillow, and tiles them into numbered, labelled sheets for visual
description. Writes sheets plus index.json (sheet -> cells -> source file).

Usage: image_sheets.py <unpacked dir> <out dir>
"""
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

EXTS = {'.dds', '.png', '.tga'}
SIZE_SUFFIX = re.compile(r'_(16|24|32|40|48|64|96|128|256|512)(?=$|_)', re.I)
UI_NAME = re.compile(r'^(ui|hud|loading|emblem|portrait|enc|minigame|notification)|_notification|loading', re.I)
LAYOUTS = {"icon": (4, 4, 192), "ui": (2, 2, 384)}
LABEL_H = 30


def checker(size):
    tile = Image.new('RGB', (size, size), (150, 150, 150))
    draw = ImageDraw.Draw(tile)
    step = 16
    for y in range(0, size, step):
        for x in range(0, size, step):
            if (x // step + y // step) % 2:
                draw.rectangle([x, y, x + step - 1, y + step - 1], fill=(185, 185, 185))
    return tile


def select(root):
    icons, ui = {}, []
    for path in root.rglob('*'):
        if path.suffix.lower() not in EXTS or not path.is_file():
            continue
        stem = path.stem
        if stem.lower().startswith('icon'):
            base = SIZE_SUFFIX.sub('', stem).lower()
            best = icons.get(base)
            if best is None or path.stat().st_size > best.stat().st_size:
                icons[base] = path
        elif UI_NAME.search(stem):
            ui.append(path)
    return sorted(icons.values(), key=lambda p: p.name.lower()), sorted(ui, key=lambda p: p.name.lower())


def build(kind, paths, out, font, start_number):
    cols, rows, cell = LAYOUTS[kind]
    per = cols * rows
    sheets, failed = [], []
    background = checker(cell)
    for s in range(0, len(paths), per):
        batch = paths[s:s + per]
        sheet = Image.new('RGB', (cols * cell, rows * (cell + LABEL_H)), (40, 40, 48))
        draw = ImageDraw.Draw(sheet)
        cells = []
        for i, path in enumerate(batch):
            x, y = (i % cols) * cell, (i // cols) * (cell + LABEL_H)
            number = start_number + s + i
            try:
                with Image.open(path) as image:
                    image.load()
                    size = image.size
                    rgba = image.convert('RGBA')
                rgba.thumbnail((cell - 8, cell - 8), Image.LANCZOS)
                tile = background.copy()
                tile.paste(rgba, ((cell - rgba.width) // 2, (cell - rgba.height) // 2), rgba)
                sheet.paste(tile, (x, y))
            except Exception as error:  # keep going; record undecodable files
                failed.append({"file": str(path), "error": str(error)})
                size = None
                draw.rectangle([x, y, x + cell - 1, y + cell - 1], fill=(90, 30, 30))
            label = f"{number}: {path.stem}"
            while draw.textlength(label, font=font) > cell - 6 and len(label) > 8:
                label = label[:-2]
            draw.text((x + 3, y + cell + 6), label, fill=(255, 255, 255), font=font)
            cells.append({"number": number, "file": path.name, "path": str(path), "size": size})
        name = f"{kind}_{s // per + 1:04d}.png"
        sheet.save(out / name, optimize=True)
        sheets.append({"sheet": name, "kind": kind, "cells": cells})
    return sheets, failed


def main():
    root, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    font = ImageFont.load_default(size=13)
    icons, ui = select(root)
    sheets_ui, failed_ui = build('ui', ui, out, font, 1)
    sheets_icon, failed_icon = build('icon', icons, out, font, len(ui) + 1)
    index = {"sheets": sheets_ui + sheets_icon, "failed": failed_ui + failed_icon,
             "counts": {"ui": len(ui), "icons": len(icons)}}
    (out / 'index.json').write_text(json.dumps(index, indent=1), encoding='utf-8')
    print(json.dumps({"ui_images": len(ui), "icons": len(icons),
                      "ui_sheets": len(sheets_ui), "icon_sheets": len(sheets_icon),
                      "failed": len(index['failed'])}))


if __name__ == '__main__':
    main()
