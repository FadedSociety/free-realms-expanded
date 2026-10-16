"""Search the described client images by words in their file name, description or tags.

Index: lab/reports/image-descriptions/index.jsonl (one described image per line:
number, file, size, description, kind, tags, text, sheet). Contact sheets with the
pictures themselves are in lab/reports/image-sheets/<sheet>.png.

Usage: image_search.py <words...> [--kind KIND] [--limit N] [--masks]
All words must match (case-insensitive). Mask/silhouette variants are hidden
unless --masks is given.
"""
import argparse
import json
from pathlib import Path

INDEX = Path(__file__).resolve().parents[2] / 'lab/reports/image-descriptions/index.jsonl'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('words', nargs='+')
    ap.add_argument('--kind')
    ap.add_argument('--limit', type=int, default=25)
    ap.add_argument('--masks', action='store_true')
    args = ap.parse_args()
    words = [w.lower() for w in args.words]
    hits = 0
    for line in INDEX.open(encoding='utf-8'):
        row = json.loads(line)
        if not args.masks and row.get('kind') == 'mask/silhouette':
            continue
        if args.kind and row.get('kind') != args.kind:
            continue
        haystack = ' '.join([row.get('file', ''), row.get('description', ''),
                             ' '.join(row.get('tags', [])), row.get('text', '')]).lower()
        if all(w in haystack for w in words):
            print(f"{row['file']}  [{row.get('kind')}]  sheet {row['sheet']} #{row['number']}\n    {row.get('description')}")
            hits += 1
            if hits >= args.limit:
                break
    print(f"-- {hits} shown")


if __name__ == '__main__':
    main()
