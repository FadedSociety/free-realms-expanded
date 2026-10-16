# Windows atomic save reader compatibility

2026-10-08. Native173814 accepts six Forging chains and two hammer drops, score6600. Its server session then ends with `Access is denied. (os error 5)`. Last Forging reply is frame10693; later native chains receive no replies. The saved first-column cells(3,8),(3,9),(3,10) are402/402/402, making the first unanswered native chain10743 valid. Later client-local board changes explain the eventual visible/saved mismatch. No gravity repair is justified by that evidence.

The last saved Forging wall time116932 places the failure after OnIdle10705 and before chain10743, consistent with a subsequent periodic snapshot tick. The gateway stages state, persists the complete snapshot and only then commits live state/sends replies. This protects economy but an IO failure ends the session. There is no native file-operation call stack identifying a particular reader process.

`character_store::replace_bytes` writes and syncs a unique temporary file in the same directory, closes its handle, then atomically replaces the destination with `fs::rename`. A Windows observer opened with read/write sharing but without `FILE_SHARE_DELETE` can reject replacement. A controlled100ms reader regression failed on the old code with exact Windows error5.

The narrow fix retries only the same atomic rename, only on Windows errors5/access denied and32/sharing violation, up to20 retries25ms apart (500ms retry-wait budget). It never deletes/truncates the old save or falls back to copying. Persistent denial still propagates and cleans the uncommitted temporary file. Validation, synced bytes, schemas and persist-before-reply transaction order remain intact; non-Windows errors retain their prior immediate behavior. The shared primitive covers character, complete gameplay and identity saves.

The new transient-reader regression passes: the open reader sees complete old bytes, replacement succeeds after release, a fresh load reads complete new state, and no temp remains. The existing permanent-denial regression still preserves exact old bytes/state and cleans its temp. Eight character-store tests, one game-store migration/corruption test and three library tests pass. No Forging source, campaign/content/save, release or native input changes were made.

Private proof and reproducer: `../lab/reports/windows-atomic-save-retry-proof.json` and `prove-windows-atomic-save-retry.py`. Independent review and fresh native acceptance under observer reads remain pending.
