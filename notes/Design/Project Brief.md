# Project brief

The owner wants to revive Free Realms into something playable in its entirety. The game has personal significance from childhood. The preservation target takes priority over the earlier idea of a game inspired by FusionFall, Toontown Online, and Free Realms.

## Direction

- Preserve the supplied client, its world, appearance, activities, and progression as evidence of the intended experience.
- Prefer Rust for rebuilt services and, eventually, a replacement client where practical.
- Use the original client to verify server compatibility before replacing rendering, animation, audio, or UI.
- Study community efforts, especially Sanctuary, with explicit version and source provenance.
- Measure restoration by playable behavior and content coverage, not extracted file counts.

## First playable milestone

A local Rust service accepts the supplied client, enters one zone, supports movement, and restores the character's position after disconnect/restart. This is a proposed implementation sequence serving the user's overall goal, not a claim that a walking world is the final scope.

See [[Rust Revival Roadmap]] and [[Research/Free Realms Recovery]].

## Decisions still open

The final Rust rendering engine, persistence schema, multiplayer hosting model, and exact restoration coverage remain open. Choose them from measured requirements as the relevant systems become understood. Do not substitute speculative new features for missing original gameplay.
