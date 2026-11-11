"""Write the world spawns' floor heights into the content data (F6, op-P-npc-life).

  python -I scripts/npc_floor.py [--terrain <dir>] [--check]

The server sets a body onto its floor when it loads the world (`src/npc_life.rs`,
`Terrain::floor_at`), but only when it runs with `--terrain`, and the terrain bake is derived
from client files, so the public release cannot carry it. This script applies the same rule
offline, from the lab bake (`lab/terrain`: `zone_0.hgt` and `zone_0.obs`), to the rows at rest:
the regional NPC rows the world draws (`npcs/*.json`), the extras bodies (`npc_spawns_extra.json`),
the field enemy homes (`field_enemies.json`) and the quest pick-ups with a body
(`quest_nodes.json`). The load-time snap then has nothing left to move.

The rule, as `src/npc_life.rs` and `src/terrain.rs` have it:
- The floor is the higher of the height grid when it is at most 2 m above the feet (the
  client's own ground cast for a moved NPC, `010ba6e0`, starts at Y+2) and the highest baked
  collision top at most 0.5 m above them.
- A body more than 0.1 m off its floor moves onto it; the height written is the fixpoint (the
  floor of the floor), so a second run, and the load-time snap, change nothing.
- Not moved: props (`npc_poses.json` `props`), rows with `keep_height` evidence, a zone's own
  placed object (`terrain_object_id`), rows under the ground with no baked floor (caves), rows
  the world does not draw, a house lot's NPCs, and capture-matched rows.
- A regional row drawn through its extras body follows that body, so the two stay together.

`--check` changes nothing and exits 1 when a row would move. Prints the rows changed per file
and region.
"""
import argparse
import array
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / "prototype" / "osfr-tools" / "data" / "content"
LIFT_M = 2.0  # terrain.rs FLOOR_CAST_LIFT_M
STEP_UP_M = 0.5  # terrain.rs FLOOR_STEP_UP_M
DEPTH_M = 2000.0  # terrain.rs FLOOR_CAST_DEPTH_M
SETTLED_M = 0.1  # npc_life.rs SETTLED_M
MAX_SLOPE_DEGREES = 50.0  # obstacles.rs
EPS = 1e-6  # obstacles.rs
VIDEO_DRAW_CONFIDENCE = ("high", "medium")  # world.rs
QUEST_NODE_GUID = 0x4E4F_4400_0000_0000 | 3 << 32  # harvest_nodes.rs node_guid(QUEST_NODE_TAG, id)


class Grid:
    """`zone_<id>.hgt` (`terrain.rs` HeightGrid): decimetre heights, bilinear lookup."""

    def __init__(self, path):
        data = path.read_bytes()
        if data[:8] != b"OSFRHGT1":
            raise SystemExit(f"{path}: not an OSFRHGT1 grid")
        self.min_x, self.min_z, self.cell = array.array("f", data[8:20])
        self.cols, self.rows = array.array("I", data[20:28])
        self.heights = array.array("h", data[28:])

    def node(self, col, row):
        if col < 0 or row < 0 or col >= self.cols or row >= self.rows:
            return None
        value = self.heights[row * self.cols + col]
        return None if value == -32768 else value / 10.0

    def height_at(self, x, z):
        fx, fz = (x - self.min_x) / self.cell, (z - self.min_z) / self.cell
        col, row = math.floor(fx), math.floor(fz)
        tx, tz = fx - col, fz - row
        corners = [(self.node(col, row), (1 - tx) * (1 - tz)), (self.node(col + 1, row), tx * (1 - tz)),
                   (self.node(col, row + 1), (1 - tx) * tz), (self.node(col + 1, row + 1), tx * tz)]
        if all(h is not None for h, _ in corners):
            return sum(h * w for h, w in corners)
        present = [h for h, _ in corners if h is not None]
        return sum(present) / len(present) if present else None


class Obstacles:
    """`zone_<id>.obs` (`obstacles.rs` OSFROBS1): world triangles in an x/z bucket grid."""

    def __init__(self, path):
        data = path.read_bytes()
        if data[:8] != b"OSFROBS1":
            raise SystemExit(f"{path}: not an OSFROBS1 obstacle file")
        self.cell, self.min_x, self.min_z = array.array("f", data[12:24])
        self.cols, self.rows, triangles, items = array.array("I", data[24:40])
        at = 40
        self.vertices = array.array("f", data[at:at + triangles * 36])
        at += triangles * 36
        buckets = self.cols * self.rows + 1
        self.starts = array.array("I", data[at:at + buckets * 4])
        at += buckets * 4
        self.items = array.array("I", data[at:at + items * 4])

    def surface_below(self, x, z, from_y, step_down):
        col, row = math.floor((x - self.min_x) / self.cell), math.floor((z - self.min_z) / self.cell)
        if col < 0 or row < 0 or col >= self.cols or row >= self.rows:
            return None
        bucket = row * self.cols + col
        best = None
        for index in self.items[self.starts[bucket]:self.starts[bucket + 1]]:
            v = self.vertices[index * 9:index * 9 + 9]
            y = triangle_surface((v[0:3], v[3:6], v[6:9]), x, z)
            if y is not None and from_y - step_down <= y <= from_y and (best is None or y > best):
                best = y
        return best


def triangle_surface(triangle, x, z):
    """`obstacles.rs` triangle_surface: the walkable y of a triangle over (x, z)."""
    a, b, c = triangle
    ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
    vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
    nx, ny, nz = uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx
    length_sq = nx * nx + ny * ny + nz * nz
    limit = math.cos(math.radians(MAX_SLOPE_DEGREES))
    if length_sq <= 1e-12 or ny * ny < length_sq * limit * limit:
        return None
    denominator = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2])
    if abs(denominator) <= 1e-12:
        return None
    l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denominator
    l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denominator
    l3 = 1.0 - l1 - l2
    if l1 < -EPS or l2 < -EPS or l3 < -EPS:
        return None
    return l1 * a[1] + l2 * b[1] + l3 * c[1]


class Floor:
    def __init__(self, terrain):
        self.grid = Grid(terrain / "zone_0.hgt")
        self.obstacles = Obstacles(terrain / "zone_0.obs")

    def floor_at(self, x, y, z):
        """`Terrain::floor_at`."""
        ground = self.grid.height_at(x, z)
        if ground is not None and ground > y + LIFT_M:
            ground = None
        top = self.obstacles.surface_below(x, z, y + STEP_UP_M, DEPTH_M)
        present = [v for v in (ground, top) if v is not None]
        return max(present) if present else None

    def settled(self, x, y, z):
        """The height to write, or None when the row stays: off by more than SETTLED_M, then
        followed to the fixpoint so `floor_at` of the written height is itself."""
        floor = self.floor_at(x, y, z)
        if floor is None or abs(floor - y) <= SETTLED_M:
            return None
        for _ in range(16):
            again = self.floor_at(x, floor, z)
            if again is None or abs(again - floor) <= 1e-4:
                break
            floor = again
        return round(floor, 4)


def read(path):
    raw = path.read_bytes()
    return json.loads(raw), b"\r\n" in raw


def write(path, document, crlf):
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    if crlf:
        text = text.replace("\n", "\r\n")
    path.write_bytes(text.encode("utf-8"))


class Regions:
    def __init__(self, content):
        self.regions = []
        for region in json.loads((content / "regions.json").read_text(encoding="utf-8"))["regions"]:
            self.regions.append((region["name"], region["shapes"]))

    @staticmethod
    def inside(shape, x, z):
        if shape["shape"] == "sphere":
            return (x - shape["x"]) ** 2 + (z - shape["z"]) ** 2 <= shape["r"] ** 2
        return shape["x1"] <= x <= shape["x2"] and shape["z1"] <= z <= shape["z2"]

    def name(self, x, z):
        best = None
        for name, shapes in self.regions:
            for shape in shapes:
                if self.inside(shape, x, z):
                    area = math.pi * shape["r"] ** 2 if shape["shape"] == "sphere" else \
                        (shape["x2"] - shape["x1"]) * (shape["z2"] - shape["z1"])
                    if best is None or area < best[0]:
                        best = (area, name)
        return best[1] if best else "zone 0"


def regional_drawn(row):
    """`world.rs` RegionalNpc::drawn."""
    body = (row.get("model_id") or 0) > 0 and (row.get("name_string_id") or 0) > 0
    placement = row.get("placement")
    if placement == "legacy_data":
        return True
    if placement == "video_index":
        return row.get("placement_confidence") in VIDEO_DRAW_CONFIDENCE and body
    return body


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--terrain", type=Path, default=ROOT.parent / "lab" / "terrain")
    parser.add_argument("--content", type=Path, default=CONTENT)
    parser.add_argument("--check", action="store_true", help="change nothing; exit 1 if a row would move")
    args = parser.parse_args()
    floor = Floor(args.terrain)
    content = args.content
    poses, _ = read(content / "npc_poses.json")
    props = {m for row in poses["props"] for m in row["model_ids"]}
    kept = {row["guid"] for row in poses["spawns"] if row.get("keep_height")}
    regions = Regions(content)
    changed = defaultdict(Counter)

    def settle(guid, model, position):
        """The new height of a body row, or None (and the reason it stays is the rule's)."""
        if model in props or guid in kept or not all(math.isfinite(v) for v in position):
            return None
        return floor.settled(*position)

    def move(file, position, y):
        changed[file][regions.name(position[0], position[2])] += 1
        position[1] = y

    # Extras bodies first: a regional row drawn through one follows it.
    extras_path = content / "npc_spawns_extra.json"
    extras, crlf = read(extras_path)
    moved_bodies = {}
    for row in extras["records"]:
        if row.get("capture_match") is not None or row.get("terrain_object_id") or row.get("content_key") == 0:
            continue
        before = list(row["position"])
        y = settle(row["spawn_guid"], row.get("model_id") or 0, row["position"])
        if y is not None:
            move(extras_path.name, row["position"], y)
            if row.get("content_key"):
                moved_bodies[row["content_key"]] = (before, list(row["position"]))
    if changed[extras_path.name] and not args.check:
        write(extras_path, extras, crlf)
    bound = {row["content_key"] for row in extras["records"] if row.get("content_key")}

    for path in sorted((content / "npcs").glob("*.json")):
        document, crlf = read(path)
        name = f"npcs/{path.name}"
        for row in document["records"]:
            position = row.get("position")
            if not position or row.get("house_zone_id") is not None or row.get("capture_match"):
                continue
            body = moved_bodies.get(row["npc_id"])
            if body and all(abs(a - b) <= 1e-3 for a, b in zip(body[0], position)):
                move(name, position, body[1][1])
                continue
            if row.get("placement") != "legacy_data" and row["npc_id"] in bound:
                continue  # the extras body draws it
            if not regional_drawn(row):
                continue
            y = settle(row["npc_id"], row.get("model_id") or 0, position)
            if y is not None:
                move(name, position, y)
        if changed[name] and not args.check:
            write(path, document, crlf)

    path = content / "field_enemies.json"
    document, crlf = read(path)
    for row in document["enemies"]:
        y = settle(row["guid"], row.get("model_id") or 0, row["position"])
        if y is not None:
            move(path.name, row["position"], y)
    if changed[path.name] and not args.check:
        write(path, document, crlf)

    path = content / "quest_nodes.json"
    document, crlf = read(path)
    types = {t["key"]: t for t in document["node_types"]}
    pools = {p["key"]: types[p["node_type"]] for p in document["pools"]}
    for row in document["spawns"]:
        node_type = pools[row["pool_key"]]
        offset = node_type.get("placement_y_offset", 0.0)
        x, y0, z = row["position"]
        drawn = [x, y0 + offset, z]
        y = settle(QUEST_NODE_GUID | row["id"], node_type["model_id"], drawn)
        if y is not None:
            move(path.name, row["position"], round(y - offset, 4))
    if changed[path.name] and not args.check:
        write(path, document, crlf)

    total = sum(sum(c.values()) for c in changed.values())
    for file, counts in sorted(changed.items()):
        if counts:
            print(f"{file}: {sum(counts.values())} ({', '.join(f'{k} {v}' for k, v in sorted(counts.items()))})")
    print(f"{'would move' if args.check else 'moved'} {total} rows onto the floor")
    return 1 if args.check and total else 0


if __name__ == "__main__":
    sys.exit(main())
