# W86 dependency inventory

These are direct dependencies, pinned in `Cargo.toml`. They are not a complete
transitive licence audit. The full `Cargo.lock` now resolves offline and the
Windows GUI builds (W86b, 2026-10-16).

| Dependency | Version | Declared licence | Reason |
|---|---|---|---|
| eframe (includes egui) | 0.33.3 | MIT OR Apache-2.0 | Native window, widgets, focus, OpenGL renderer, accessibility |
| gilrs | 0.11.0 | MIT OR Apache-2.0 | Requested controller API; XInput backend on Windows |
| image | 0.25.9 | MIT OR Apache-2.0 | Decode existing PNG banner and save renderer screenshot; PNG only |
| log | 0.4.34 | MIT OR Apache-2.0 | Opt-in eframe/glutin/winit diagnostics; already present transitively, no additional package resolved |
| serde | 1.0.229 | MIT OR Apache-2.0 | Reuse the existing controller layout schema |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | Read existing `scripts/launcher/gamepad.json` |

eframe/gilrs/image metadata: [eframe](https://docs.rs/crate/eframe/0.33.3),
[gilrs](https://docs.rs/crate/gilrs/0.11.0),
[image](https://docs.rs/crate/image/0.25.9).
serde/serde_json licences were also read from the private registry's cached
Cargo.toml files. This workspace already uses the serde/serde_json family; these
exact versions are present in its private Cargo cache.

The launch core uses only std and one documented Kernel32 function. There is no
direct Windows wrapper crate, regex crate, HTTP client, async runtime, Python
runtime, authentication implementation or game-content dependency.

Before distribution, retain the full lockfile; collect the licences
of the actual Windows dependency closure and bundled default fonts (their font
licences also apply). The existing banner is compiled in from the repository;
this inventory does not assign a new licence to the art. Exact package size is
unmeasured for release. W86b's debug EXE is 119,785,472 bytes (not a release-size
estimate). It retains Glow: no WGPU dependency or toolkit licence change.

The Windows recipe statically links the llvm-mingw unwind runtime rather than
requiring a sidecar `libunwind.dll`. Its unpacked `LICENSE.TXT` identifies Apache
2.0 with LLVM exceptions; retain applicable toolchain/runtime notices when
packaging. A complete runtime/font/transitive distribution audit remains open.

## W86c bundled fonts and art (2026-10-16)

No crate was added or updated. Fonts are compiled in; nothing is fetched at
runtime. Display titles and glossy button captions use **Lilita One Regular**
by Juan Montoreano, unmodified. Body text keeps egui's Ubuntu Light; addresses
use Hack. The two existing emoji fallbacks remain in egui's default definitions.

| Font / asset | Licence / provenance | Included notice |
|---|---|---|
| `assets/fonts/LilitaOne-Regular.ttf` (28,092 bytes) | SIL Open Font License 1.1; Copyright 2011 Juan Montoreano, reserved font name Lilita | `assets/fonts/LilitaOne-OFL.txt` |
| Ubuntu Light (bundled by epaint_default_fonts 0.33.3) | Ubuntu Font Licence 1.0 | `assets/fonts/UFL.txt` |
| Hack Regular (bundled by epaint_default_fonts 0.33.3) | MIT (Source Foundry), Bitstream Vera licence; DejaVu contributions public domain | `assets/fonts/Hack-Regular.txt` |
| Noto Emoji Regular (bundled by epaint_default_fonts 0.33.3) | SIL Open Font License 1.1 | `assets/fonts/OFL.txt` |
| emoji-icon-font (bundled by epaint_default_fonts 0.33.3) | MIT; Copyright 2014 John Slegers | `assets/fonts/emoji-icon-font-mit-license.txt` |
| Pills, bevels, panel frames and status pips | New Rust/egui geometry in `src/theme.rs` / `src/main.rs`, covered by this crate's declared MIT OR Apache-2.0 code licence | No third-party image input |
| Existing `../docs/images/banner.png` | Existing project generated art, unchanged; generation brief: `lab/runs/banner/prompt-v6.md`. No separate redistribution grant was found in the repository. This task does not assign rights to game marks/characters. | Inherited art packaging/licensing question for the lead |

Lilita One was read from the primary [Google Fonts directory](https://github.com/google/fonts/tree/main/ofl/lilitaone)
using the connected GitHub reader (shell network access was unavailable).
Content-addressed Git blobs pin the actual downloaded files:

- Font: `fdec368cbd478e437f0476a538c3868723aed296`, SHA-256 `f5b641c45c69d772ee4eda687bc9fda411d5cad6b0b45371491da4580cbc8d59`.
- Licence: `055594f2f330e835940070f2d5944a2331c7c299`, SHA-256 `255d5debbb80eb2ea762644311f266a279e8778f00156655a516e2b7781a63e1`.

The other four notices were copied without modification from the private cache's
`epaint_default_fonts-0.33.3/fonts/` directory, matching the locked renderer.
There are no newly generated decorative PNGs or cut-outs. The font notices are
complete for the five fonts compiled into this launcher; the earlier full
transitive/runtime distribution audit remains a separate packaging gate.
