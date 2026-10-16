# 043 Pet trainer and card progression

Status: bounded recovery + reconstruction returned for lead review, 2026-10-09 04:07 UTC. Queue
acceptance was "recover profile/unlock data; link only validated activity outcomes". Delivered:
the recovered job profile rows and catalogue cross-check, the job level-up notification contract,
the milestone unlock locale blocks, and a bounded ledger that credits only validated activity
outcomes. It is **not** a playability claim: no native level-up was run and no XP value is
fabricated. Commands below ran from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_043.rs` | Recovered level-up/milestone tables for Card Duelist and Pet Trainer plus the validated-outcome ledger. Not registered in `src/lib.rs` (lead-owned). |
| `prototype/osfr-tools/tests/piece_043.rs` | 13 behavioral tests; `#[path]`-includes the module so it compiles before registration. |
| `scripts/pieces/piece_043.py` | Read-only audit/checker over the preserved inputs; writes the private lab bundle. |
| `tests/pieces/test_piece_043.py` | 13 stdlib tests: synthetic rejection paths plus hash/shape pins on real shipped files. |
| `../lab/reports/piece-043-20261009T040732Z/` | Private evidence: `progression.json`, `findings.json`, `provenance.json`, `red-rust-baseline.log`, `red-python-baseline.log`, `green-rust-piece043.log`, `python-tests-piece043.log`, `extractor-run.log`. |

## Recovered originals

**Profile rows.** `lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`
(opcode 12; `FUN_0089b240` installs `PacketSendSelfToClient::vftable` and writes `0xc`) carries 16
`ClientPcProfiles` rows with `JobGUID`, `JobName`, `JobDescription`, `JobIcon`, `JobBadgeBorder`,
`JobBadge`, `MembersOnly`, `JobLevel`, `JobLevelXP` and `JobLevelTotalXP`. Card Duelist is job
**120**, name locale **2410**, description **2411**, recorded level **20** with 0 XP. No row is Pet
Trainer (name locale 17419), matching the wiki note that the job was removed on 2009-12-14.
The 16 recorded rows match the pinned community catalogue
`lab/references/FreeRealms-Legacy/src/Resources/Profiles.json` (SHA-256 `334feb47…`) **16/16** on
name/description/icon/badge-border/badge/membership; that catalogue has no Pet Trainer row either,
so the Pet Trainer job id stays unrecovered. Observation, not a claim: the field the community
serializer calls `IsCombat` runs 0,2,3,…,17 and skips 1 and 14, so at least two historical job rows
are unaccounted for.

**Level-up contract.** `lab/decompiled/client-ui/ScriptsBase.lua` (SHA-256 `c7c71c7c…`, line 11776)
`NotificationHandler:JobLevelUp(jobType, jobName, iconId, jobLevel, jobBadgeImageSetId)` accepts
badge image sets `{10,11,12,13,19,20,32,1783,1785,1999,46,4842,4843}` and maps
`t[1783] = "levelup_tcg"` / `t[1785] = "levelup_pettrainer"` with per-level sound ids
`s[1783] = {1:16114,4:16115,8:16116,12:16117,16:16118}` and
`s[1785] = {1:16119,4:16120,8:16121,12:16122,16:16123}`. Level 1 shows locale **919922**, every
other level **919923**; the sound table is indexed by the exact level, so a level outside
{1,4,8,12,16} still plays the movie without a recovered sound. The native side dispatches the same
five-value `JobLevelUp` notification (`pseudocode.c:1099444`). Card Duelist's badge image set 1783
is confirmed twice: the pinned catalogue's `BadgeImageSet` and the Lua table key. Pet Trainer's
1785 is Lua-only; its `NameId` (17419), gear row (2436) and achievement category (4624) exist in the
locale export but no shipped catalogue row does.

**Milestones and assets.** The achievement blocks are contiguous and complete: 34070-34074
"Card Duelist: Level 4/8/12/16/20" with 34150-34154 "Reach Level N as a Card Duelist." and
34110-34114 / 34190-34194 for Pet Trainer. `Assets_manifest.txt` ships `levelup_pettrainer.gfx`,
`levelup_tcg.gfx` and the per-job audio `levelup_{pettrainer,cardduelist}_level{1,5,10,15,20,_misc}.mp3`;
the pack catalogue carries five badge tiers per job (`icon_job_badge_pettrainer01..05`,
`icon_job_badge_tcg01..05`). The audio names use 1/5/10/15/20 while the Lua sound keys use
1/4/8/12/16; both are shipped data and the mapping between them is **not** resolved here.

## Reconstructed (not historical)

`piece_043.rs` labels these at every call site: a once-per-instance credit ledger (replayed
completions are refused; abandoned/lost outcomes credit nothing and do not consume the instance),
a caller-supplied XP curve because no threshold table is recovered, and the non-member level-4 cap,
which is community-documented (fandom wiki "Jobs") rather than first-party. No award amount,
grade formula or title grant is invented.

## Verification

```
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_043
# RED before implementation: error: couldn't find file `tests\..\src\restoration\piece_043.rs`
# GREEN: test result: ok. 13 passed; 0 failed

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_043.py" -v
# RED before implementation: FileNotFoundError scripts\pieces\piece_043.py (1 error)
# GREEN: Ran 13 tests ... OK

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_043.py' --stamp 20261009T040732Z
# {"card_duelist_level": 20, "catalogue_mismatches": 0, "recorded_job_rows": 16,
#  "findings": ["pet-trainer-profile-row-absent", "pet-trainer-job-id-unrecovered",
#  "xp-thresholds-unrecovered", "level-up-message-locale-missing",
#  "tcg-badge-tier04-256-missing", "pettrainer-badge-256-missing-in-pack-catalog"]}
```

## Gaps and remaining gates

* XP thresholds per level, and therefore any honest level-up pacing, are unrecovered; the ledger
  takes a curve from the caller. This is the largest gap.
* Locale ids 919922/919923 (level-up messages) are absent from the export, and the Pet Trainer job
  id, its own profile catalogue row and its 256px badge tier entries are unrecovered.
* No capture contains a job level-up, so the server-to-client trigger, the exact `jobType` value and
  the badge-tier-to-level rule stay unverified end to end.
* Integration: the module is unregistered. XP may only be credited from outcomes the owning activity
  modules validated (piece 011 trick rounds, piece 010 card matches); the final atomic grant stays
  with the lead-owned economy layer.
* Native acceptance is lead-controlled (shared client window). `code/test-candidate` is the
  strongest status this piece supports.

## Skills applied

`reverse-engineering` for the Lua handler, the native opcode-12 constructor and the five-value
notification dispatch; `game-recon` to separate the client's Scaleform/UI data from server state;
community-first practice via the pinned FreeRealms-Legacy catalogue, OpenSourceFreeRealms serializer
and the read-only wiki corpus before any custom parsing; plus RTK (normal commands first), Ponytail
(reused the existing locale export and catalogues instead of new readers) and Caveman (concise
reporting). `asset-pipeline` and `game-automation` were not needed: no asset output and the shared
native window is lead-controlled.

Cross-links: [[../../MMO Work Queue]], [[010 Trading card game]], [[011 Pet training activities]],
[[../Free Realms Recovery]].
