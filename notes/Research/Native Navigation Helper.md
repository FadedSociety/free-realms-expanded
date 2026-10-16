# Native navigation helper

2026-10-08 UTC. Prepared for root's review; no live input was performed by this subagent.

`lab/tools/navigate-local-game.py` guides the already-running local client toward an explicit X/Z waypoint using original A/D turning and W walking. It reads the existing Rust snapshot's position and horizontal direction after every batch, then recomputes the next action. Initial rates, measured by root, are 1.45 radians/second and 8.5 units/second; observed batches update those estimates.

Default execution is a dry-run printing one proposed action. `--execute` calls the existing `project/scripts/drive-local-game.py`, preserving Universal Modder foreground checks and the run's `external-inputs.jsonl`. No focus, click, chat, teleport, direct network packet or saved-state write is included. Each key hold is bounded to 500 ms. The game must already be foreground and its visible modal/route state reviewed.

The helper records before/after positions, direction, snapshot hashes, each proposed hold and its measured effect to a separate `navigation-*.jsonl` in the selected run. It stops on a zone mismatch, an unchanged snapshot after two seconds, two ineffective batches, opposite turning, movement away from the waypoint, unexpected altitude/displacement, or the configured travel/time limit. Default arrival radius is 2.5 units; default route bound150 units/time90 seconds, configurable up to300 units/180 seconds. This is a straight-line waypoint helper; it does not solve obstacles or terrain paths. A stopped run requires visible review and a new waypoint, not repeated blind retries.

Example dry-run from the current supported character:

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/tools/navigate-local-game.py' --run '../lab/runs/client-baseline-20261008T081219Z' --state '../lab/state/opening-supported-character.json' --target -1897.1788 413.7308 --expected-zone 0
```

Append `--execute` only for a reviewed real navigation session. The helper does not itself request or infer permission; root applies the session's existing input authorization.

`lab/reports/test-native-navigation.py` passes offline heading-wrap/left/right/forward/arrival cases, a simulated closed-loop arrival, a dry-run that rejects any drive call, a non-game-foreground refusal and a changed-zone refusal. All simulated holds are at most500 ms and contain no focus command. Both scripts pass Python compilation. These checks establish the bounded helper behavior, not successful live navigation; root owns that acceptance.

## Root's bounded live checks

Root invoked the reviewed helper twice in ended run081219. Independent read-only log inspection confirms `navigation-20261008T082952334731Z.jsonl` has17 batches,8.605 units of movement and28.203 seconds; `navigation-20261008T083204553289Z.jsonl` has18 batches,38.637 units and33.016 seconds. Both stop explicitly after two ineffective batches, with every recorded key hold at most500 ms. These establish the live obstacle-stop behavior; neither claims destination arrival or general terrain navigation. This subagent sent no native input.
