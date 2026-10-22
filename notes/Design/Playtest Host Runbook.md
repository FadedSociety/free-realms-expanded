# Playtest Host Runbook

**Readiness: dry-run only.** Do not invite players until the blocking gates at the end pass. Bare internet stays closed. Friends use an encrypted VPN.

This guide uses Windows PowerShell from the project root. Values named in angle brackets are placeholders; replace them before running a command. Keep the bundle, account store, handoff key, security log, and saves on the owner's machine.

AUTH2, Ed25519 enrollment, account roles, and the launcher profile are reconstructed policy (R). This runbook adds no new recovered client facts. The original login packet's Session and Fingerprint fields are documented separately in [[../Research/Key-pair Login Contract]].

## 1. Before the night

1. Pick a fresh name for this private bundle.

```powershell
$bundleName = 'playtest-night-20261016'
```

Change the date or add a suffix when that folder already exists.

2. Build the bundle.

```powershell
python -I scripts/bundle/make_bundle.py --name $bundleName
```

The builder creates a fresh 32-byte server identity and an empty schema-3 account store. Keep this identity for the life of this realm. Do not rebuild the bundle to restart the server.

The bundle is built from the committed `HEAD`. Commit the content you want first: the builder prints a `WARNING` naming any edited or new `data/content` file that is not in the bundle (for example a new `power_ups.json`). Its `Runtime content:` line lists the optional files included (`kart_gates.json`, `consumables.json`, `power_ups.json`); a missing optional file is fine, the gateway then uses its built-in table or leaves the feature off.

3. Set the bundle path.

```powershell
$bundle = Join-Path (Resolve-Path '..\lab\bundles').Path $bundleName
```

4. Verify the bundle and its loopback login, gateway, and bot smoke check.

```powershell
python -I scripts/bundle/check_bundle.py --bundle $bundle
```

Stop if the manifest or security scan fails. The bundle stays under lab/bundles. It includes private client-derived and Legacy-derived data.

5. Build the server tools into a lab target directory.

```powershell
$target = Join-Path (Resolve-Path '..\lab').Path 'build\playtest-night'
$env:CARGO_TARGET_DIR = $target
```

```powershell
.\scripts\cargo.ps1 build --locked --offline --release --manifest-path .\prototype\osfr-tools\Cargo.toml --bin osfr-account --bin osfr-login --bin osfr-gateway
```

6. Create a new runtime directory.

```powershell
$runtime = Join-Path $env:LOCALAPPDATA ('FreeRealms\Playtest\' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runtime -ErrorAction Stop | Out-Null
```

7. Copy the empty account-store template into the runtime.

```powershell
Copy-Item -LiteralPath (Join-Path $bundle 'server\accounts.json') -Destination (Join-Path $runtime 'accounts.json')
```

Never put active accounts or saves into the bundle.

8. Generate the gateway handoff key.

```powershell
python -I -c "import pathlib,secrets,sys; pathlib.Path(sys.argv[1]).write_bytes(secrets.token_bytes(32))" (Join-Path $runtime 'handoff.key')
```

Keep this file private. The client and friends never receive it.

9. Print the server identity.

```powershell
$serverId = Join-Path $bundle 'server\server-id.bin'
python -I -c "import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_bytes().hex())" $serverId
```

The output is the 64-digit identity required by the launcher profile. Share it only with the trusted playtest group.

10. Print the public fingerprint.

```powershell
Get-FileHash -LiteralPath $serverId -Algorithm SHA256 | Select-Object -ExpandProperty Hash
```

Share this SHA-256 fingerprint separately so friends can compare it out of band. Never accept a changed identity without checking with the owner.

11. Record the patch-manifest pin.

```powershell
Get-FileHash -LiteralPath (Join-Path $bundle 'manifest.json') -Algorithm SHA256 | Select-Object -ExpandProperty Hash
```

The bundle is not a patch server. The host still needs a VPN-only patch URL that serves the intended client overlay, plus this trusted manifest pin. Do not serve the whole bundle directory: it contains server-side reference data.

12. Create the owner's account key with the launcher when W131 is available.

W131's Settings → Join a realm key flow is coming. A current launcher CLI key-creation attempt failed here because the managed workspace blocked access to its owner-only profile directory. Its command is not a verified host procedure. Send only the public key to the account store. Never copy or send `key.ed25519`.

13. Create the host account.

```powershell
$accountExe = Join-Path $target 'release\osfr-account.exe'
$handle = Read-Host 'Account handle'
$publicKey = Read-Host '64-character public key'
& $accountExe create --store (Join-Path $runtime 'accounts.json') --handle $handle --public-key $publicKey
```

14. Make the host account an admin before login starts.

```powershell
& $accountExe set-role --store (Join-Path $runtime 'accounts.json') --account-id 1 --role admin
```

The first account in a fresh store is ID 1. Account roles are server policy (R). W120's in-game command syntax is `realm role <account-id> <player|builder|admin>`; it is admin-only. For example:

```text
realm role 1 admin
```

On the shared gateway an admin may send it without developer tools (decision 1), but login holds the account-store lock while it runs, so the gateway answers "account store is busy or unavailable" (code: `osfr-login.rs` `run`, `Gateway::change_account_role`; not run natively). Use the offline `osfr-account set-role` command above for the playtest account.

15. Set the VPN address and server binaries.

```powershell
$vpn = '<VPN IPv4 address>'
$gatewayExe = Join-Path $target 'release\osfr-gateway.exe'
$loginExe = Join-Path $target 'release\osfr-login.exe'
$clientKey = $env:OSFR_CLIENT_LOGIN_KEY_FILE
if (-not $clientKey -or -not (Test-Path -LiteralPath $clientKey -PathType Leaf)) { throw 'Set OSFR_CLIENT_LOGIN_KEY_FILE to the host-only legacy client protocol key file.' }
```

Replace the address placeholder with the private VPN address. Keep the legacy client protocol key on the host. Never put it in the bundle or send it to players.

16. Start the gateway first, bound to the VPN address.

```powershell
$env:OSFR_GATEWAY_STARTUP_SNAPSHOT = Join-Path $bundle 'startup\startup.json'
& $gatewayExe ($vpn + ':20041') --reference (Join-Path $bundle 'reference') --state (Join-Path $runtime 'character.json') --content (Join-Path $bundle 'content') --world (Join-Path $runtime 'world') --terrain (Join-Path $bundle 'terrain') --client-locale (Join-Path $bundle 'client-locale') --account-store (Join-Path $runtime 'accounts.json') --handoff-key-file (Join-Path $runtime 'handoff.key') --security-log (Join-Path $runtime 'security.jsonl') --studio-port 0 --allow-lan --seconds 14400
```

`--world` keeps World Builder saves (placed NPCs, marks, authored text) in the runtime. The gateway creates the folder when the first character enters the world; without the flag, these saves go into the bundle's content folder (`Gameplay::live_dir`). Players see new authored text only after `scripts/build_world_locale.py` rebuilds their client locale from this folder and they restart the client.

Leave this terminal open. The 14,400-second limit is a planned four-hour clean shutdown. The command was checked with a two-second limit on loopback and bound to a VPN address in L053's run (see Evidence); a four-hour run was not exercised. Do not add --dev-tools.

17. Start login in a second terminal.

```powershell
& $loginExe --bind ($vpn + ':20042') --auth-bind ($vpn + ':20043') --gateway ($vpn + ':20041') --resources (Join-Path $bundle 'login-resources') --identity (Join-Path $runtime 'identity.json') --session $handle --state (Join-Path $runtime 'login-state.json') --account-store (Join-Path $runtime 'accounts.json') --server-id-file (Join-Path $bundle 'server\server-id.bin') --handoff-key-file (Join-Path $runtime 'handoff.key') --key-file $clientKey --allow-lan --seconds 14400
```

The login broker listens on the VPN address (Security Model decision 2). `--auth-bind` accepts a private address only with `--allow-lan`; a public or wildcard address is refused at startup. Friends' launchers accept loopback, RFC 1918, 100.64.0.0/10 and IPv6 unique-local broker endpoints, and refuse public ones. No tunnel is needed for the VPN playtest. Per-IP connection limits apply to every peer, loopback included. The VPN-interface bind ran on this PC's Tailscale address in L053's run (see Evidence); login from another PC over the VPN has not.

18. Restrict network access to the VPN.

Allow UDP 20041 and 20042 and TCP 20043 on the VPN interface only. Keep public port forwarding closed. The VPN address must be a private address accepted by --allow-lan.

19. Share the connection details through the trusted playtest channel.

Share the VPN login address, the VPN gateway address, the server identity, its SHA-256 fingerprint, the patch-manifest URL, and its SHA-256 pin. The patch URL is a blocking setup gap until a VPN-only host is configured.

For example, these are the two game endpoints:

```powershell
'Login UDP: ' + $vpn + ':20042'
'Gateway UDP: ' + $vpn + ':20041'
```

### Admin and developer tools

The offline osfr-account set-role command gives the owner the admin account role. It does not turn on developer tools.

The shared gateway refuses --dev-tools on a non-loopback bind. Studio stays off with --studio-port 0. Admin and builder `realm` commands still work there: with an account store, the gateway checks each command against the account's role and needs no --dev-tools (Security Model decision 1, `264db2a`; native evidence below). Builders get building commands only. Diagnostics and experiments (for example `realm mine`, `harvest`, `npx off`, `offer`, `race` and the kart lobby catalogue) still need --dev-tools, so they are off on the shared gateway. Do not start a second gateway against the same saves; separate processes do not share session ownership. The scavenger hunt is ordinary quest content (3000200 in `data/content/quests/scavenger_hunt.json`), announced with the admin-only `realm broadcast` (see Night-blocking gaps).

## 2. Each friend

1. Install the client and launcher build designated for this playtest. Do not send the private playtest bundle.

The W131 installer and friend join flow are coming. The intended screen is Settings → Join a realm. The launcher README describes entering the game address, the 64-digit server identity, and an account handle; comparing the host fingerprint; creating a per-server key; then sending only the public key to the owner. The join screen also has a login broker field, prefilled with the realm address's IP and port 20043; leave it unless the host says otherwise. W131's brief was not present in the checked-in notes, and this path has not passed fresh-PC acceptance.

2. Find the friend's existing account ID.

Login must be stopped because it holds the account-store writer lock. Use the account ID from this list and the public key sent over the trusted channel.

```powershell
& $accountExe list --store (Join-Path $runtime 'accounts.json')
```

3. Add the friend's public key to that account.

Login must stay stopped while the account store is edited. The gateway may remain running.

```powershell
$accountId = Read-Host 'Existing friend account ID'
$publicKey = Read-Host '64-character public key from friend'
& $accountExe add-key --store (Join-Path $runtime 'accounts.json') --account-id $accountId --public-key $publicKey
```

4. Create a new friend account when one does not exist yet.

Use the friend's first public key and an admin-assigned handle. This creates the account and its first key.

```powershell
$friendHandle = Read-Host 'New friend account handle'
$friendPublicKey = Read-Host '64-character public key from friend'
& $accountExe create --store (Join-Path $runtime 'accounts.json') --handle $friendHandle --public-key $friendPublicKey
```

5. Restart login with the same runtime paths and server identity.

Do not replace the runtime account store with the empty bundle copy.

6. Tell friends to keep their private key on their own PC.

Public-key enrollment is operator-assisted. A private key, password, handoff key, login token, or account-store file is never sent to the owner.

## 3. During the night

1. Watch the gateway and login terminals.

The gateway prints a listening event with dev_tools false. Login prints listening and auth_listening. Stop if either process exits before its planned timeout.

2. Check the gateway security log.

```powershell
Get-Content -LiteralPath (Join-Path $runtime 'security.jsonl') -Tail 20
```

The log batches admission refusals, action-budget refusals, and developer-command denials. It has no names, chat, IP addresses, or tokens. It rotates at 1 MiB and keeps one .previous file. Login authentication failures are not currently written to this gateway log. An empty file does not prove a login succeeded.

3. Run a broker-login stress preflight before friends arrive.

```powershell
python -I scripts/stress/run.py --bots 10 --duration-seconds 600 --broker-login
```

The help output confirms these flags. This run was not executed for W132: the script writes to its fixed lab/reports/broker-login-w130 folder, outside this task's assigned report path. Review its REPORT.md, processes.csv, and timeline.jsonl before inviting players. The most recent recorded fixed-gateway ten-minute run had 0 bot errors over 31 sessions, ping p99 0.259 s, purchases 10/10, and saves 10/10. It did not close the full 60-minute or broker-login gate.

4. Handle a crashed client by waiting 60 seconds.

The gateway rejects a duplicate selected account or character. After a crash, wait up to 60 seconds from that client's last packet for the idle session to expire and save. Do not launch another client on that account during the wait.

5. Drain sessions before a restart.

Ask every player to exit normally. Wait for each session-ended event in the gateway terminal. For a crashed client, wait 60 seconds from its last packet for the duplicate-session window to expire and save. After the last security-log event, allow five seconds for the log flush. The gateway's timed shutdown writes remaining sessions and flushes the log.

6. Use the planned timed shutdown.

The gateway has a clean shutdown path when --seconds expires. Ctrl+C with active sessions is not verified as a clean shutdown. Do not force-kill either server. For an early restart, drain all sessions and confirm the gateway has no active sessions. If that cannot be confirmed, wait for the planned timed shutdown. Then wait five seconds for the log flush and verify the save files before reopening.

7. Restart with the same bundle and runtime.

Use the same server-id.bin, accounts.json, handoff.key, character directory, world folder (`--world`), and gateway state path. Never copy the empty accounts.json over the active store.

### Launcher login errors

These messages come from the current launcher AuthError mapping. They cover broker authentication, not every patch-download error.

| Launcher message | Host or player action |
|---|---|
| Server unreachable. Check the broker or secure tunnel. | Check VPN membership, that login is listening on the VPN address port 20043, and the VPN firewall rule for TCP 20043. |
| Wrong server identity. Check the trusted server profile. | Stop. Compare the 64-digit identity and SHA-256 fingerprint with the owner's out-of-band values. Do not accept a changed identity automatically. |
| Login refused. Check key enrolment or try again later. | Check the account handle and public-key enrollment. AUTH2 intentionally hides whether an account is unknown, disabled, wrong-key, or locked. Five failures in five minutes lock the handle for 60 seconds; stop retrying and wait. |
| Server closed the login request. Try again later. | Check that login is running and restart it if it exited. Then try once with a fresh challenge. |
| Login timed out. Check the broker or try again later. | Check the VPN route, login terminal, and server load. Retry after the route is stable. |
| Server returned an invalid login reply. | Check the profile's broker endpoint. It must reach the AUTH2 broker (TCP 20043), not the game UDP port. |
| Invalid server profile. Check the trusted profile settings. | Correct the saved profile's broker endpoint (loopback or a private/VPN address), identity, and account handle. |
| Cannot access the private profile files. Check their permissions. | Keep the profile in the signed-in user's APPDATA. Restore its owner-only permissions; never copy the private key to another PC. |
| The operating system could not generate a secure key. | Restart the launcher and check Windows cryptographic services. Do not substitute a manually chosen key. |

## 4. After the night

1. Wait for both services to stop.

2. Create a host-only backup directory.

```powershell
$backupRoot = Read-Host 'Path on the host-only encrypted backup volume'
New-Item -ItemType Directory -Path $backupRoot -Force | Out-Null
```

3. Back up the complete private runtime.

```powershell
$backup = Join-Path $backupRoot ('playtest-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
Copy-Item -LiteralPath $runtime -Destination $backup -Recurse
```

This includes character saves, social data, World Builder saves, the account store, and the handoff key. Keep the backup on a host-only encrypted volume. Do not upload it with bug reports.

4. Save one Markdown file per bug.

Use lab/reports/playtest-night-YYYYMMDD/bugs/bug-NNN.md. Include UTC time, bundle name and source revision, client build, region, short reproduction steps, expected result, actual result, frequency, and a redacted screenshot or recording path. Use anonymous labels such as P01. Remove player names, chat, account handles, keys, tokens, and VPN addresses from screenshots and logs.

## 5. Pre-flight checklist

- [ ] Private bundle builds and its full checker passes.
- [ ] Bundle identity and manifest pin are recorded out of band.
- [ ] Fresh runtime store and handoff key exist outside the bundle.
- [ ] Host-only legacy client protocol key is configured and readable.
- [ ] Host account is admin before login starts.
- [ ] VPN and firewall expose only UDP 20041 and 20042 to VPN peers.
- [ ] Gateway starts with account-store mode, --world in the runtime, --allow-lan, --security-log, --studio-port 0, and no --dev-tools.
- [ ] Login and its AUTH2 broker start on the VPN address with --allow-lan.
- [ ] W131 launcher install/join flow works on a clean second PC.
- [ ] Private patch URL and trusted manifest pin work without exposing the server bundle.
- [ ] Two real clients pass login, character select, visibility, movement, and chat over VPN.
- [ ] Ten bots pass a 60-minute broker-login run with saved-state reload checked.
- [ ] Host-only event tools are available without enabling them for players.
- [ ] Save backup and bug-report folders are ready.

Do not invite players while any item stays unchecked.

## Night-blocking gaps

- The W131 install/join screen, key creation, and VPN broker login have not passed a clean-PC test.
- There is no verified VPN-only patch host or patch URL.
- The real client has not logged in through the launcher's key-pair (AUTH2) path, and two-client visibility/gameplay over VPN is untested.
- Gateway tickets are HMAC-bound for every account since L023 (the public fixed ticket is refused), but they are reusable and never expire: a ticket stays valid until the handoff key changes (Security Model S02/L3; the one-use, expiring part is open). The lead decides whether this blocks the night.
- The scavenger hunt and the admin's `realm broadcast` have not run natively (Build Plan 3.3).
- The broker-login 60-minute stress gate is open; the launcher key command also failed under this workspace's profile-folder ACL.

## Evidence and open gates

- Bundle build, manifest/security check, bot login, and persisted-walk smoke test passed on loopback in the W132 scratch bundle.
- Account-store create, add-key, set-role, and list passed against a disposable store.
- Login and gateway started in account-store mode on loopback with --allow-lan, --security-log, --studio-port 0, and no --dev-tools. The gateway reported dev_tools false.
- The local source supports private-network binds with --allow-lan. No second-PC session was tested.
- VPN bind, 2026-10-22 (L053, `lab/runs/lan-test-20261022T041757Z`): gateway, login and AUTH2 broker started on this PC's Tailscale address (100.64.x.x) with the commands above, and `osfr-bot --broker-profile ... --allow-lan` passed broker auth, UDP login, handoff and world entry against them from the same PC. The run found two bugs, now fixed: login refused any non-loopback `--gateway`, so the login command above could not start; and the bot bound loopback, so it could not send to a VPN address. A Windows Sandbox client did not reach the servers, because no Windows Firewall allow rule existed for the release binaries. Allow UDP 20041/20042 and TCP 20043 before a remote client joins.
- client-baseline-20261022T005637Z: the real client (1/1) entered on the HMAC handoff (L023), through an account-store gateway with login in loopback dev mode. The launcher's key-pair (AUTH2) login has not run with the real client, and no two-real-client VPN run is recorded.
- Launcher key creation failed in this managed workspace because its owner-only profile directory could not be accessed. W131's friend UI, a clean installation package, and fresh-PC login remain unverified.
- No private patch host or patch URL exists in this runbook's evidence.
- The gateway rejects --dev-tools for non-loopback binds. Role-checked `realm` commands need no --dev-tools with an account store: native run client-baseline-20261021T193008Z (loopback, no --dev-tools) ran the admin's `realm god`, and a player account's `realm give`, `goto`, `here` and `god` got the role refusal.
- The stress harness's broker-login command was not run because it writes outside W132's owned report path. The full 60-minute broker-login gate remains open.

## Source notes

| Runbook fact | Provenance |
|---|---|
| Bundle contents, server identity, account-store template, and checker scope | [[Playtest Bundle]]; `scripts/bundle/make_bundle.py`, `scripts/bundle/check_bundle.py` |
| AUTH2 endpoint, stable server ID, account-store lock, and lockout policy | [[../Research/Key-pair Login Contract]]; `prototype/osfr-tools/src/bin/osfr-login.rs` |
| Launcher join intent and displayed login errors | [[../Research/Launcher Login Client]]; `launcher/README.md`, `launcher/src/auth.rs`; W131 remains coming and its brief was not present in checked-in notes |
| `osfr-account` command syntax | `prototype/osfr-tools/src/bin/osfr-account.rs`; scratch CLI output in `lab/reports/host-runbook-w132/account-cli.log` |
| Role command and admin check | W120 / [[Build Plan]]; gateway command routing. Role assignment policy (R) |
| Role commands without --dev-tools | Decision 1 (`264db2a`): `Gameplay::may_run`, `account_store::can_run_realm_command`; native run `lab/runs/client-baseline-20261021T193008Z` (lead native notes, 19:30Z) |
| `--world` live-world folder | `prototype/osfr-tools/src/bin/osfr-gateway.rs` option parsing; `Gameplay::set_world_dir` and `live_dir` in `src/gameplay.rs`; [[World Builder]]; locale rebuild in `scripts/build_world_locale.py` (docstring) |
| Fixed ticket refused, HMAC tickets reusable (L023) | `prototype/osfr-tools/src/login.rs` character selection (`gateway_binding_ticket` for every account, 1/1 included); `src/bin/osfr-gateway.rs` `select_character_from_login` (refuses any other ticket, logs `auth_failure`); test `tests/playtest_gateway_smoke.rs` `the_fixed_ticket_is_refused_and_1_1_enters_with_its_binding_ticket`; [[Security Model]] S02/L3; native run `lab/runs/client-baseline-20261022T005637Z` (gateway `--account-store`, login dev mode) |
| Diagnostics gated on --dev-tools | `src/gameplay.rs` realm dispatch (`mine`, `harvest`), `developer_npx_off`, quest `offer`, kart `realm race` and the 102 lobby reply |
| Private bind, `--allow-lan`, dev-tools restriction, and Studio bind | [[Security Model]]; `prototype/osfr-tools/src/bin/osfr-gateway.rs` |
| Security-log event types, rotation, and lack of auth-failure events | `prototype/osfr-tools/src/security_log.rs`, login/gateway call sites; scratch startup in `lab/reports/host-runbook-w132/service-startup.log` |
| Save paths, session expiry, and shutdown persistence | [[Build Ledger]]; `prototype/osfr-tools/src/gameplay.rs`, `prototype/osfr-tools/src/bin/osfr-gateway.rs` |
| Stress flags, fixed report path, and latest ten-minute result | `scripts/stress/run.py`; [[Build Plan]] and [[Current Progress]]; help output in `lab/reports/host-runbook-w132/stress-help.log` |

AUTH2, Ed25519 enrollment, account roles, lockout behavior, and host runbook policies are reconstructed server policy (R), not recovered original-client behavior. Native-client acceptance remains open.
