# W86 dependency inventory

These are direct dependencies, pinned in `Cargo.toml`. They are not a complete
transitive licence audit. No full Cargo.lock could be resolved in this session:
the GUI/controller packages are not cached and crates.io access failed.

| Dependency | Version | Declared licence | Reason |
|---|---|---|---|
| eframe (includes egui) | 0.33.3 | MIT OR Apache-2.0 | Native window, widgets, focus, OpenGL renderer, accessibility |
| gilrs | 0.11.0 | MIT OR Apache-2.0 | Requested controller API; XInput backend on Windows |
| image | 0.25.9 | MIT OR Apache-2.0 | Decode existing PNG banner and save renderer screenshot; PNG only |
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

Before distribution, resolve and retain the full lockfile; collect the licences
of the actual Windows dependency closure and bundled default fonts (their font
licences also apply). The existing banner is compiled in from the repository;
this inventory does not assign a new licence to the art. Exact package size is
unmeasured: no runnable GUI binary was produced in this restricted session.
