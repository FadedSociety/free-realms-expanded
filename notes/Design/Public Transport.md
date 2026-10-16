# Public Transport

Design only (lead, 2026-10-16; replaces the W137 worker brief). How players reach a public realm **without installing a VPN**, using the unmodified 2014 client. The VPN stays the answer for the ~10-friend playtest ([[Playtest Host Runbook]]). Related: [[Security Model]] (S03, S05, S15, I10–I12, Decisions), [[../Research/Key-pair Login Contract]], [[../Research/Launcher Login Client]], [[Shared World]], [[Build Plan]].

**Recommendation in one line:** the launcher embeds a **WireGuard** tunnel (the `boringtun` library, no driver or admin rights). The client talks plain UDP to the launcher on 127.0.0.1. A new server-side **edge** process ends the tunnel and forwards plain UDP to the existing login and gateway on loopback. No legacy code changes, no invented crypto.

## 1. Requirements

1. **No client changes.** The 2014 client keeps sending SOE UDP to the address in its config. The launcher already controls that config (ClientConfig overlay), so the client is pointed at 127.0.0.1.
2. **No VPN install.** No driver, no admin rights, no virtual network adapter. Everything runs inside the launcher process.
3. **Players are authenticated by their existing per-server key profile** (W100). No passwords.
4. **Confidentiality and integrity** for both login and gameplay (S15: SOE CRC and the fixed RC4 key give neither).
5. **Resists replay and session hijack** (S03: today sessions are keyed by IP:port).
6. **Works behind home NAT**, including NAT rebinding mid-session.
7. **No added head-of-line stalls.** The SOE layer already has its own reliable and unreliable channels, so the tunnel must carry independent datagrams and never retransmit them.
8. **One server handles ~1,000 players.** The W96/W106 interest grid and the W124 gateway loop fix already cover the game side.
9. **Bare legacy UDP stays closed to the internet** (I10). Only the tunnel port is public.

## 2. Options compared

Crate facts come from crates.io as of 2026-10-16.

| Option | Rust crates (licence) | Launcher build (`x86_64-pc-windows-gnullvm`, getrandom 0.2 rule) | Player auth | Server auth | Fit |
|---|---|---|---|---|---|
| **WireGuard, embedded** ([WireGuard paper](https://www.wireguard.com/papers/wireguard.pdf), [protocol](https://www.wireguard.com/protocol/)) | `boringtun` 0.7.1 (BSD-3-Clause), default features empty; pulls `ring` 0.17, `x25519-dalek` 2, `rand_core` 0.6 | **Fits.** `rand_core` 0.6 and `ring` 0.17 both use getrandom **0.2** ([ring deps](https://crates.io/crates/ring/0.17.14/dependencies)). Ring builds with clang, which llvm-mingw provides; a build spike must prove it on gnullvm. | Static X25519 key per profile, known to the server (Noise IKpsk2). Unknown keys get no reply. | Pinned server static key; the handshake fails without it | **Recommended** |
| **QUIC + datagrams** ([RFC 9000](https://www.rfc-editor.org/rfc/rfc9000), [RFC 9221](https://www.rfc-editor.org/rfc/rfc9221)) | `quinn` 0.11.12 (MIT/Apache-2.0), `rustls` 0.23 (Apache/ISC/MIT), plus ring or aws-lc-rs | **Blocked today.** quinn-proto 0.11.19 requires `getrandom ^0.4` and `rand ^0.10` (non-optional). getrandom 0.3+ needs dlltool, which our GNU toolchain lacks (Handoff 2026-10-16). aws-lc-rs also needs cmake. | AUTH2 on a stream, bound to the TLS exporter ([RFC 8446 §7.5](https://www.rfc-editor.org/rfc/rfc8446#section-7.5)) | Pinned Ed25519 key, via raw public keys ([RFC 7250](https://www.rfc-editor.org/rfc/rfc7250); in rustls's [feature list](https://docs.rs/rustls/0.23.43/rustls/manual/_04_features/index.html)) or a pinned self-signed cert | Good protocol, but blocked on toolchain. The server answers any Initial (3× anti-amplification, RFC 9000 §8). |
| **Noise IK, hand-framed** ([Noise spec](https://noiseprotocol.org/noise.html)) | `snow` 0.10.0 (Apache/MIT), `getrandom ^0.3` optional | Possible with an injected RNG resolver | Static key | Static key | We would have to design the replay window, rekeying, cookies and roaming ourselves. WireGuard is exactly Noise IK with those already specified, so this is strictly worse. |
| **DTLS 1.3** ([RFC 9147](https://www.rfc-editor.org/rfc/rfc9147)) | No mature pure-Rust stack; OpenSSL bindings | OpenSSL on gnullvm is a large build risk | Client cert | Cert pin | Rejected |
| **TLS over TCP** ([RFC 8446](https://www.rfc-editor.org/rfc/rfc8446)) | `rustls` + ring | Same ring spike | AUTH2 over the stream | Pin | **Fallback only.** TCP retransmits every loss and stalls the SOE stream behind it, which breaks requirement 7. Only for networks that block UDP (open question 3). |

### Cost per connection at 1,000 players (WireGuard)

- **CPU.** One handshake is a few X25519 operations. WireGuard rekeys every 120 s (`REKEY_AFTER_TIME`, [protocol §timers](https://www.wireguard.com/protocol/)), so 1,000 players produce about 8 handshakes a second. Traffic of about 20 packets/s × 1,000 players × ~300 B is about 6 MB/s of ChaCha20-Poly1305 ([RFC 8439](https://www.rfc-editor.org/rfc/rfc8439)), a small fraction of one core. This is an estimate; slice 4 measures it.
- **Memory.** A `Tunn` holds session keys, counters and a replay bitmap: kilobytes, not megabytes. The edge's per-player loopback sockets and buffers dominate memory; slice 4 measures that too.
- **Bytes.** WireGuard adds 32 B per packet. The inner header (below) adds 21 B. SOE packets are at most 512 B, so the 1,420 B tunnel MTU is never approached.

## 3. Recommended design

### 3.1 Shape

```
FreeRealms.exe ──UDP──> launcher 127.0.0.1:20042 (login) / :20041 (gateway)
   launcher: boringtun Tunn ──WireGuard UDP──> internet ──> osfr-edge :51820 (only public port)
      osfr-edge: one Tunn per player ──UDP on loopback──> osfr-login 127.0.0.1:20042, osfr-gateway 127.0.0.1:20041
```

### 3.2 Keys and identity

- **Server identity becomes a real public key.** Today the 64-hex "server identity" is 32 random bytes used only to bind AUTH2 signatures. For tunnel realms it becomes the edge's static X25519 public key. The join screen field and the AUTH2 binding stay the same size, and the pin now also authenticates the server (WireGuard's handshake fails without the matching private key).
- **Each launcher profile gets a second key, an X25519 tunnel key**, created alongside the existing Ed25519 login key with `x25519-dalek`, the version boringtun already pulls in. Never convert the Ed25519 key into an X25519 key: reusing one key across two protocols is a cross-protocol risk. The "public key" the player sends for enrollment becomes `osfr1:<ed25519-hex>:<x25519-hex>`. `osfr-account add-key` stores both.
- **Optional pre-shared key: not used.** WireGuard's PSK is for post-quantum hedging. Without a way to distribute it, it adds nothing here.

### 3.3 Handshake and login

1. The launcher starts the tunnel when the player presses PLAY. This is a standard WireGuard handshake initiation to the edge.
2. **The edge learns the player's key before allocating anything.** It reads the initiator's static key with `boringtun::noise::handshake::parse_handshake_anon` ([docs](https://docs.rs/boringtun/0.7.1/boringtun/noise/handshake/fn.parse_handshake_anon.html)), then:
   - **Enrolled key:** creates the player's `Tunn` and their peer slot.
   - **Unknown key:** drops the packet, with no reply. WireGuard's mac1 also requires the sender to know the server public key, so internet scanners get nothing back (no reflection, S05).
3. **AUTH2 runs unchanged, inside the tunnel.**
   - The launcher sends each AUTH2 request (≤192 B) as one datagram on the control flow.
   - The edge opens one loopback TCP connection to the existing broker per request, exactly as the launcher does today, and returns the reply as one datagram. On loss the launcher retries with a fresh challenge, which is safe because challenges and tokens are one-use.
   - So `osfr-login` and the Key-pair Login Contract need no change.
4. **The edge forwards legacy UDP only after the AUTH2 proof succeeds** on that peer (it sees reply byte 0 = 1). The gateway's existing token/HMAC checks still apply on top. Outsiders can never send a single legacy packet (closes S03 and S05 for the internet).
5. The launcher starts the client with the one-use Session token as today, and points it at 127.0.0.1.

### 3.4 Inner framing (inside WireGuard)

boringtun's `Tunn` carries IP packets and checks the IP header. So each inner packet is:

- a minimal **IPv4 header** (20 B): protocol 253 (experimental, [RFC 3692](https://www.rfc-editor.org/rfc/rfc3692)), source = the player's assigned tunnel address (100.64.x.y from the edge), destination = the edge's tunnel address;
- then **1 byte flow id**: 0 = control (AUTH2), 1 = login UDP, 2 = gateway UDP;
- then the payload, byte for byte.

The edge drops any inner packet whose source does not match that peer's assigned address. This is WireGuard's allowed-IPs rule, and it stops inner spoofing. There is no TCP/IP stack, no extra checksum (WireGuard's AEAD already covers integrity), and no retransmission (requirement 7).

### 3.5 What WireGuard gives for free (and the standard behind it)

- **Replay:** a 64-bit counter plus a sliding window ([RFC 6479](https://www.rfc-editor.org/rfc/rfc6479) style; WireGuard paper §5.4.6).
- **Hijack:** session keys come from the handshake. Roaming updates the endpoint only on an authenticated packet (paper §2.1), so IP:port spoofing (S03) no longer works.
- **NAT:** the client initiates, and persistent keepalive every 25 s holds the NAT mapping ([wg(8)](https://man7.org/linux/man-pages/man8/wg.8.html) `PersistentKeepalive`).
- **DoS under load:** cookie replies (mac2) make a spoofed source prove it can receive before the server does expensive work (paper §5.3).

### 3.6 Server side: a separate `osfr-edge` process

- It is a new standalone crate, `project/edge/` (like the launcher, so its dependencies never touch the server crate). It forwards plain UDP to the existing processes on loopback. The legacy code stays untouched, a crash in the edge cannot corrupt saves, and one edge can front several gateways later.
- **Loopback fan-out:** each admitted player gets its own loopback UDP socket per flow. The gateway therefore still sees a distinct `127.0.0.1:port` per player, and its IP:port session keying keeps working unchanged.
- **Gateway changes (small, lead-owned):**
  1. **Per-IP cap.** The gateway's 4-per-IP cap would count every player as 127.0.0.1. Add `--edge 127.0.0.1`, which exempts only that source from the per-IP cap. The edge enforces the per-real-IP cap instead.
  2. **Session cap.** Raise the 16-session cap to a configured value (needed for 1,000 regardless).
  3. **Advertised gateway (verify).** The login server must advertise the gateway at the launcher's local address. Check whether `--gateway` is only an advertisement; if not, split it into "advertised" and "actual".
- **Multi-gateway routing (later, R):** the edge keeps a table of flow → backend. If zones are split across gateway processes, the launcher listens on a small local port range (20041–20049) and the edge maps each local port to one backend gateway. Not needed until one gateway process saturates; the W124 numbers say it does not at 1,000.

### 3.7 Admission limits and the security log move to the edge

- **Limits the edge enforces:**
  - Per-real-IP handshake rate, using the existing `net_policy::IpLimiter` shape, plus WireGuard cookies under load.
  - A maximum peer count.
  - A pre-auth deadline: AUTH2 must succeed within 10 s or the peer is dropped (R).
  - A per-peer datagram token bucket and idle expiry.
- **Security log:** the edge writes the existing `security_log.rs` JSONL format with the real source IP. New kinds: `tunnel_unknown_key`, `tunnel_inner_spoof`, `tunnel_preauth_timeout`, `tunnel_rate`.
- **The gateway keeps all its own limits** (defence in depth). It just sees the edge, not the internet.

## 4. Build plan

Each slice is independently testable and reviewed by the lead (security).

| # | Slice | Gate |
|---|---|---|
| 0 | **Build spike:** `boringtun` (default features off) + `ring` compile into the launcher on `x86_64-pc-windows-gnullvm` with getrandom staying 0.2; the server side compiles too. No feature code. | `build.ps1 build` with no warnings, `cargo tree -i getrandom` shows only 0.2 on the target. Licences recorded in `launcher/DEPENDENCIES.md`. If ring fails, stop and report: the fallback is RustCrypto-only primitives, which needs a decision. |
| 1 | **Tunnel library, both ends, in memory:** inner framing (3.4), flow demux, assigned-address check, enrolled-key lookup via `parse_handshake_anon`. | Unit tests: an unknown key gets no reply; inner spoof dropped; replayed packet dropped; flows stay separate; a canary for each. |
| 2 | **`osfr-edge` binary + loopback fan-out + AUTH2 proxy + pre-auth gate**, plus the gateway `--edge` and session-cap options. | Bots (osfr-bot through a test tunnel client) log in through the edge; legacy UDP before AUTH2 is refused; per-IP limit counts real IPs; security log rows appear. Full server suite green. |
| 3 | **Launcher tunnel**: X25519 profile key, `osfr1:` enrollment string, local UDP listeners, keepalive, client pointed at 127.0.0.1. | Launcher tests; two launchers on one PC log in as different accounts; NAT rebinding simulated by changing the source port mid-session. |
| 4 | **Loss/latency stress:** 10 bots → 100 bots through the edge with 2 % loss and 80 ms added delay; then the 60-minute run. | 0 bot errors, p99 within 2× the no-tunnel baseline, edge memory and CPU per peer recorded. |
| 5 | **Native acceptance:** the owner's client through the launcher tunnel to a realm on a different network, then two clients in one shared field. | The native shared-world checklist passes over the tunnel; packet capture on the wire shows only WireGuard. |

## 5. Open questions and risks for the owner

1. **Hosting.** One public UDP port (51820 or similar) on a host you control. The cheapest is a small VPS running the edge; it could also forward to your PC over the VPN. Your call when we get there.
2. **Sign-up for strangers.** Today a host enrolls each key by hand. For public play we need self-service sign-up, which is a separate design. The edge can admit unknown keys into an "enroll-only" mode, but that reopens a pre-auth surface, so it gets its own review.
3. **UDP-blocked networks** (some schools and offices). The fallback is TLS-over-TCP with the same inner frames. It will lag under loss. Worth building only if players hit it.
4. **Toolchain risk.** Slice 0 can fail if `ring` will not build on gnullvm. The fallback is RustCrypto primitives behind boringtun, or adding dlltool to the toolchain, which would also unblock QUIC.
5. **Two keys per profile.** Slightly longer enrollment string. Later we could drop AUTH2 for tunnel realms (the WireGuard handshake already proves the key) and have the edge mint the session token, but keeping AUTH2 unchanged is the safer first step.
