"""Builds game_treasurewar.gfx, the Treasure War movie, from game_treasurewar.as.

    python -I build.py <java> <ffdec-cli.jar> <out.gfx>

The movie is one frame of script and no library: this writes an empty one-frame SWF 8 and has
JPEXS FFDec (26.3.0 was used) compile the script into its frame. The client's Scaleform loads a
plain SWF under a .gfx name. The built file is a client asset: it ships in the launcher's fill
pack, never in this repository.
"""
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
WIDTH, HEIGHT, FPS = 800, 600, 30


def tag(code, body=b""):
    if len(body) < 63:
        return struct.pack("<H", code << 6 | len(body)) + body
    return struct.pack("<HI", code << 6 | 63, len(body)) + body


def rect(width, height):
    """RECT 0..width, 0..height in twips, 15 bits a field."""
    bits = f"{15:05b}" + "".join(f"{value:015b}" for value in (0, width * 20, 0, height * 20))
    bits += "0" * (-len(bits) % 8)
    return int(bits, 2).to_bytes(len(bits) // 8, "big")


def empty_swf():
    body = rect(WIDTH, HEIGHT) + struct.pack("<BBH", 0, FPS, 1)
    body += tag(69, b"\x00\x00\x00\x00")  # FileAttributes: ActionScript 1/2
    body += tag(9, b"\x1f\x3a\x2b")  # SetBackgroundColor
    body += tag(12, b"\x00")  # DoAction: end
    body += tag(1)  # ShowFrame
    body += tag(0)  # End
    return b"FWS\x08" + struct.pack("<I", 8 + len(body)) + body


def main(java, ffdec, out):
    with tempfile.TemporaryDirectory() as work:
        work = pathlib.Path(work)
        base = work / "base.swf"
        base.write_bytes(empty_swf())
        frame = work / "scripts" / "frame_1"
        frame.mkdir(parents=True)
        shutil.copy(HERE / "game_treasurewar.as", frame / "DoAction.as")
        built = work / "built.swf"
        run = subprocess.run(
            [java, "-jar", ffdec, "-importScript", str(base), str(built), str(work / "scripts")],
            capture_output=True,
            text=True,
        )
        print(run.stdout[-2000:], run.stderr[-2000:])
        if run.returncode != 0 or not built.is_file():
            sys.exit("FFDec did not build the movie")
        if built.stat().st_size <= base.stat().st_size + 1000:
            sys.exit("the script was not compiled into the movie")
        shutil.copy(built, out)
        print(f"wrote {out}: {built.stat().st_size} bytes")


if __name__ == "__main__":
    main(*sys.argv[1:4])
