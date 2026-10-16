"""Validate supplied assets using the pinned community FR ADR Toolkit readers."""
import csv
import hashlib
import json
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
from PIL import Image

lab = Path(__file__).resolve().parents[2] / "lab"
source = lab / "references/fr-adr-toolkit"
sys.path.insert(0, str(source))
from fr_formats import AdrFile, DmaFile, DmeFile, DskFile

root = lab / "extracted/client-assets"
with (root / "catalog.tsv").open(encoding="utf-8", newline="") as stream:
    rows = {row["name"].lower(): row for row in csv.DictReader(stream, delimiter="\t")}
output = lab / "extracted/community-asset-check"
output.mkdir()  # Refuse to overwrite an earlier check.

def asset(name):
    return root / rows[name.lower()]["extracted_relative_path"]

actor = AdrFile.load(asset("chatdy.adr"))
material = DmaFile.load(asset(actor.material_file))
skeleton = DskFile.load(asset(actor.skeleton_file))
mesh = DmeFile.load(asset(actor.mesh_file))
texture = np.asarray(Image.open(asset(material.textures[0])).convert("RGB")) / 255.0
all_vertices, polygons, colors = [], [], []
for chunk in mesh.meshes:
    vertices = np.asarray([v.position.as_tuple() for v in chunk.vertices])
    uv = np.asarray([v.uv for v in chunk.vertices])
    indices = np.asarray(chunk.indices, dtype=np.int64).reshape(-1, 3)
    assert np.isfinite(vertices).all() and indices.min() >= 0 and indices.max() < len(vertices)
    # The game uses Y up; the plot uses Z up. This is a static bind-pose preview.
    vertices = vertices[:, [0, 2, 1]]
    all_vertices.append(vertices)
    polygons.extend(vertices[indices])
    face_uv = uv[indices].mean(axis=1) % 1.0
    x = np.minimum((face_uv[:, 0] * texture.shape[1]).astype(int), texture.shape[1] - 1)
    y = np.minimum(((1.0 - face_uv[:, 1]) * texture.shape[0]).astype(int), texture.shape[0] - 1)
    colors.extend(texture[y, x])

vertices = np.concatenate(all_vertices)
figure = plt.figure(figsize=(7, 7), facecolor="#18202a")
axes = figure.add_subplot(111, projection="3d", facecolor="#18202a")
axes.add_collection3d(Poly3DCollection(polygons, facecolors=colors, edgecolors="none"))
low, high = vertices.min(axis=0), vertices.max(axis=0)
center, radius = (low + high) / 2.0, max(high - low) * 0.55
axes.set_xlim(center[0] - radius, center[0] + radius)
axes.set_ylim(center[1] - radius, center[1] + radius)
axes.set_zlim(center[2] - radius, center[2] + radius)
axes.set_box_aspect((1, 1, 1))
axes.view_init(elev=14, azim=45)
axes.set_axis_off()
axes.set_title("Chatdy — community reader, static pose", color="white", pad=10)
figure.savefig(output / "chatdy-preview.png", dpi=130, bbox_inches="tight")
plt.close(figure)

warpstone = DmeFile.load(asset("sg_warpstone_01_lod0.dme"))
names = ["chatdy.adr", actor.mesh_file, actor.material_file, actor.skeleton_file,
         material.textures[0], "sg_warpstone_01_lod0.dme"]
report = {
    "community_source": json.loads((source / "SOURCE.json").read_text()),
    "chatdy": {"mesh_version": mesh.version, "vertices": len(vertices),
               "triangles": len(polygons), "bones": len(skeleton.bone_names),
               "animation_slots": len(actor.locomotion_animations)},
    "warpstone": {"mesh_version": warpstone.version, "chunks": len(warpstone.meshes),
                  "vertices": sum(len(m.vertices) for m in warpstone.meshes),
                  "triangles": sum(len(m.indices) // 3 for m in warpstone.meshes)},
    "inputs": [{"logical_name": name, "path": str(asset(name)),
                "sha256": hashlib.sha256(asset(name).read_bytes()).hexdigest()} for name in names],
    "limitations": ["Selected assets only", "Static pose preview; animation not tested",
                    "No game files modified", "No new binary format parser written"],
}
(output / "report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
print(json.dumps({"output": str(output), "chatdy": report["chatdy"],
                  "warpstone": report["warpstone"]}, indent=2))
