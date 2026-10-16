# Free Realms restoration

Restore Free Realms with Rust as the preferred implementation language. CartoonMMO is the existing workspace name.

Open `notes` in Obsidian and start with [Start Here](notes/Start%20Here.md). Read the [recovery report](notes/Research/Free%20Realms%20Recovery.md), [Rust roadmap](notes/Design/Rust%20Revival%20Roadmap.md), and [journal](MODLOG.md).

This repository holds notes, recovery scripts, and original Rust implementation work. The sibling `../lab` holds preserved archives, working copies, decompiled output, extracted assets, community references, and private toolchains.

[osfr-tools](prototype/osfr-tools/README.md) now runs a playable local Rust slice in the original client: a reconstructed three-enemy quest, damage and defeat, coin/item rewards, native equipment, a timed checkpoint activity, and saved progress after restart. The actual client restored 35 coins, the reward quantity of 10 and equipped pants. Seventy-three ordinary tests pass. See the [gameplay milestone](notes/Research/Gameplay%20Milestone.md) for evidence, launch commands and limits.

```powershell
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml
```

The supplied server is a community emulator, not the original production server. Full restoration requires reconstructing missing behavior and measuring content coverage. Native pseudocode and recovered C# are research inputs, not an automatic Rust port.

Community tools were also run on the supplied assets: ForgeLight Toolbox/DbgPack independently matched an entire pack, and FR ADR Toolkit read Chatdy and a warpstone model. See [community validation](notes/Research/Community%20Asset%20Validation.md) and [Universal Modder assessment](notes/Research/Universal%20Modder%20Assessment.md).

The original emulator baseline is recorded in [Client Baseline](notes/Research/Client%20Baseline.md). Initialization still uses 46 private captured messages and the native Windows client. Quest/combat/activity rules in this slice are explicitly reconstructed; complete original quest and minigame restoration remains ahead. Inventory category browsing works; text search crashed once and needs diagnosis. Track accepted behavior in the [coverage ledger](notes/Research/Gameplay%20Coverage.md).
