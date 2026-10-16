"""Verify Free Realms native .z bodies against the proven prefixed-zlib format.

Evidence: pinned Sanctuary.Core/IO/ZLib.cs and native FUN_010ebd70 /
FUN_010ebee0 / FUN_010ee080. This does not fetch files or alter the client.
"""
from pathlib import Path
import argparse
import hashlib
import json
import struct
import zlib

MAGIC = bytes.fromhex("a1b2c3d4")
MAX_BYTES = 64 * 1024 * 1024


def decode(body: bytes) -> bytes:
    if not 8 < len(body) <= MAX_BYTES or body[:4] != MAGIC:
        raise ValueError("invalid native asset envelope magic/size")
    expected = struct.unpack(">I", body[4:8])[0]
    if not 0 < expected <= MAX_BYTES:
        raise ValueError("invalid native asset decoded length")
    inflater = zlib.decompressobj(15)
    raw = inflater.decompress(body[8:], expected + 1)
    if len(raw) != expected or not inflater.eof or inflater.unused_data or inflater.unconsumed_tail:
        raise ValueError("truncated, oversized, trailing, or length-mismatched native zlib body")
    return raw


def encode(raw: bytes) -> bytes:
    if not 0 < len(raw) <= MAX_BYTES:
        raise ValueError("invalid native asset raw size")
    return MAGIC + struct.pack(">I", len(raw)) + zlib.compress(raw, 6)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("body", type=Path)
    parser.add_argument("--expected-raw", type=Path)
    parser.add_argument("--expected-crc", type=int)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    body = args.body.read_bytes()
    raw = decode(body)
    if args.expected_raw and raw != args.expected_raw.read_bytes():
        raise ValueError("decoded asset differs from expected raw file")
    if args.expected_crc is not None and zlib.crc32(raw) != args.expected_crc:
        raise ValueError("decoded asset CRC differs from manifest/request CRC")
    rejected = []
    if args.self_test:
        raw_deflater = zlib.compressobj(6, zlib.DEFLATED, -15)
        raw_deflate = raw_deflater.compress(raw) + raw_deflater.flush()
        bad_crc = bytearray(body)
        bad_crc[-1] ^= 1
        variants = {
            "bad_magic": b"BAD!" + body[4:],
            "wrong_decoded_length": MAGIC + struct.pack(">I", len(raw) + 1) + body[8:],
            "truncated": body[:-1],
            "trailing": body + b"x",
            "bad_zlib_checksum": bytes(bad_crc),
            "bare_zlib": body[8:],
            "raw_deflate": body[:8] + raw_deflate,
        }
        for name, variant in variants.items():
            try:
                decode(variant)
            except (ValueError, zlib.error):
                rejected.append(name)
            else:
                raise AssertionError(f"invalid variant accepted: {name}")
        assert decode(encode(raw)) == raw
    print(json.dumps({
        "body": str(args.body.resolve()),
        "body_length": len(body),
        "body_sha256": hashlib.sha256(body).hexdigest(),
        "declared_decoded_length": struct.unpack(">I", body[4:8])[0],
        "decoded_length": len(raw),
        "decoded_crc32": zlib.crc32(raw),
        "decoded_sha256": hashlib.sha256(raw).hexdigest(),
        "exact_stream_consumption": True,
        "regenerated_envelope_length": len(encode(raw)),
        "regenerated_envelope_matches_exactly": encode(raw) == body,
        "rejected_invalid_variants": rejected,
    }, indent=2))


if __name__ == "__main__":
    main()
