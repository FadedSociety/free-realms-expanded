# Rust revival roadmap

The target is the complete Free Realms experience. These milestones make progress testable without assuming that every original rule or content definition exists in the supplied files.

| Stage | Deliverable | Completion evidence | Current state |
| --- | --- | --- | --- |
| 1. Preserve and recover | Preserved archive copies, file manifests, managed source, raw assets, native analysis database | Matching hashes; all pack entries CRC-verified; decompiler outcomes recorded | Managed recovery and asset extraction complete; native export saved, initial analysis partial |
| 2. Establish the reference | Supplied client operating locally against a known emulator baseline | Recorded launch result, exact configs, local packet trace, reproducible zone entry | Confirmed world entry and walking; server observes position changes; reproducible bounded launch and screenshots saved |
| 3. Rust compatibility slice | Reliable SOE transport, gateway login, one character, one zone, movement, minimal persistence | Actual client connects, moves, disconnects, and restores its character after restart | Verified on the actual client; intentional packet loss recovered; initialization uses 46 private captured messages |
| 4. Gameplay slice | One NPC interaction, one quest, one combat/activity loop, reward and inventory update | End-to-end objective completion and persisted reward, compared with evidence | Verified reconstructed quest, combat, native equipment, rewards and checkpoint activity with restart; see Gameplay Milestone |
| 5. Content restoration | Zones, jobs, quests, encounters, gathering/crafting, collections, housing, social systems and activities | Per-system and per-content coverage ledger; repeatable playable checks | Coverage unknown |
| 6. Rust client replacement | Asset readers, one rendered zone, character animation, input, UI, audio, protocol client | Side-by-side behavior and visual checks against the reference client | Not started; engine unselected |
| 7. Complete playable release | Integrated content, progression and persistence; installation and recovery path | Fresh installation playthroughs, saved-game migration checks, documented remaining gaps | Long-term target |

Stages can overlap when evidence supports it. A Rust client does not have to precede restoring gameplay. Native pseudocode is used to identify behavior; translating it mechanically would retain unknowns and platform dependencies.

## Verified local gateway criteria

The next server slice must handle valid and malformed frames, packet loss, reordering, fragmentation, CRC, and compression before game-level messages rely on it. Fixtures must identify whether they came from the original client, the supplied emulator, a community reference, or a synthetic serializer call.

The first local Rust build passed these checks; evidence is in [[Research/Rust Gateway Milestone]]:

1. Launch the supplied client with recorded local settings.
2. Accept its actual handshake and login sequence.
3. Initialize a character and one zone.
4. Move continuously without disconnecting; distinguish client-only movement from server state updates.
5. Save the character, restart the local server, and verify restored state.

## Next acceptance criteria

The reconstructed local gameplay slice now satisfies the basic loop and persistence criteria; see [[Research/Gameplay Milestone]]. Next, diagnose the native inventory search fault and replace one reconstructed quest/encounter with evidenced original behavior. Quest1617's Petty Yeti identity remains unresolved. Original soccer is the strongest minigame candidate from archived traces; prove normal match completion and rewards, not merely arena entry.

## Restoration coverage

Track each system with five separate states: evidence located, behavior understood, implemented in Rust, tested in isolation, and verified in the game. Track content counts only after definitions are identified. A quest icon, voice line, packet class, or zone filename does not prove the associated quest or zone works.

Missing original server logic may need reconstruction from community code, packet traces, client checks, and surviving references. Record uncertain behavior and deviations. Full fidelity is a goal whose feasibility must be evaluated system by system.
