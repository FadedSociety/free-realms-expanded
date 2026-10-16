"""Build the locale reverse map: hash key -> numeric Global.Text id.

The client locale keys entries by JenkinsLookup2("Global.Text.<id>"); the build
guides cite those hashes, but the server sends numeric ids. This scans ids
0..N (default 6,000,000) in parallel and keeps the ones present in the locale.

Usage: locale_id_map.py <en_us_data.dat> <out.json> [--max N]
"""
import argparse
import importlib.util
import json
import multiprocessing as mp
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "quest_text_backfill", Path(__file__).with_name("quest_text_backfill.py"))
qb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(qb)

_KEYS = None


def _init(keys):
    global _KEYS
    _KEYS = keys


def _scan(bounds):
    lo, hi = bounds
    h = qb.jenkins_lookup2
    return [(key, n) for n in range(lo, hi) if (key := h(f"Global.Text.{n}")) in _KEYS]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("locale", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument("--max", type=int, default=6_000_000)
    args = ap.parse_args()
    keys = frozenset(qb.load_locale(args.locale))
    step = 100_000
    chunks = [(lo, min(lo + step, args.max)) for lo in range(0, args.max, step)]
    found = {}
    with mp.Pool(initializer=_init, initargs=(keys,)) as pool:
        for part in pool.imap_unordered(_scan, chunks):
            for key, n in part:
                found.setdefault(key, []).append(n)
    args.out.write_text(json.dumps({"max_id": args.max, "keys_in_locale": len(keys),
                                    "mapped": len(found),
                                    "hash_to_ids": {str(k): v for k, v in sorted(found.items())}}),
                        encoding="utf-8")
    print(f"locale keys {len(keys)}, mapped {len(found)} up to id {args.max}")


if __name__ == "__main__":
    main()
