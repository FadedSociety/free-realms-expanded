# Native Login Accounts 023623

Run `lab/runs/client-baseline-20261014T023623Z`: the first native run after W55's login integration. Login and gateway share an account store and use an HMAC handoff key. The test character is a fresh copy in `lab/reports/login-native-20261014T0230*`.

## Lead fixes needed before the run

1. `run-local-baseline.py` did not pass W55's new required arguments, so `osfr-login` exited with "--handoff-key-file required". The launcher now writes a per-run 16-byte `handoff.key` into the run directory. It passes that key plus `--account-store <identity dir>/accounts.json` to both servers, and `--state` to login. A backup is in `lab/snapshots/run-local-baseline.py.pre-handoff-20261014`.
2. `osfr-gateway` failed at startup with os error 2, because it required the account store to exist. On first use, only login creates the store, by migrating the legacy save and identity. The startup check now skips a store that doesn't exist yet; sessions still load the store after login.

## Results

| Check | Result |
| --- | --- |
| Login migrates the legacy save/identity into `accounts.json` | Passed: schema 1, 1 account, 1 character, next ids 2/2. The originals are unchanged |
| Character select lists and renders the character | Passed. The list labels read "1" and "0" instead of the name, the same as run 011719 before W55, so this predates W55 |
| Select → handoff → gateway world entry | Passed: login opcode 7 selected; gateway `world_ready` |
| World: HUD, job, stats, action bar | Passed. Matches run 011719 |
| Menu exit | Passed: `client_disconnect` |

## Open

- Two-character create/list/select/delete/re-login (the plan in `Research/Login Integration.md`) is not run yet. It needs the native character creator.
- The character-select list labels ("1"/"0") predate W55. They're probably fields in the character-list payload that the list reads as text; see `Login Integration.md` readers `00fa7730`, `00fa6e30`, `00fa61e0`.
