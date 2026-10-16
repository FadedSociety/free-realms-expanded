# System: platform.win32.gdi

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00490fd5` | `DrawGlyphGDI` |  | glyph rasterisation path of the GDI font device: draws the character twice (mask and fill) and post-processes the destination surface pixels. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491295` | `MeasureCharABCWin32` |  | per-character metric query of the GDI font device used to size glyph cells. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

