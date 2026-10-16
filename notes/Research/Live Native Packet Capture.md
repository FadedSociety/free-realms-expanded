# Live Native Packet Capture

The native baseline runner now has an explicit `--live-capture` option. This is private diagnostic tooling for the next Cooking iteration. It does not change gateway behavior, game packets, saves, the original receive bounds, or the final authoritative manifest.

The forwarding loop still sends first and captures immutable datagram bytes/metadata in memory. Its only new action is a nonblocking `put_nowait` into a bounded 4096-item queue. A separate worker writes the existing `frames/NNNNNN.bin` files, closes each file, then publishes the unchanged record into `frames/live-index.jsonl`. A queue overflow stops accepting further live records and sets an explicit incomplete/error status. Disk failures also mark the live prefix incomplete. The worker is stopped and joined before the original final frame writes begin, so it cannot overwrite a final frame concurrently. A pending tail at shutdown is explicitly incomplete; final memory capture still contains the authoritative trace.

`frames/live-status.json` and every live decoder proof contain `authoritative: false`. The live reader accepts only newline-terminated records. The existing private decoder retains its CRC, decompression, bundle and reliable reassembly checks, while allowing and reporting an incomplete trailing reliable message only with `--live`. It writes `analysis-live`, leaving final `analysis` separate. The decoder expects the original native login handshake and a complete self message; wait until world entry before using it. A live prefix can show the exact completed 39/14 request without ending the client session. It is not final capture acceptance evidence.

Example, using the private Python environment:

```powershell
# Add --live-capture to the existing run-local-baseline.py launch command.
& ..\lab\tools\wiki-env\Scripts\python.exe ..\lab\reports\analyze-native-created-character.py --run client-baseline-YYYYMMDDTHHMMSSZ --live
```

The isolated proof replays finalized native run 154503 into a new fixture while the writer stays open. All 4780 records retain their order, length and SHA256. The original decoder verifies 4759 CRCs and 2703 applications; all original proof fields are identical. A 992-frame prefix retains the complete self message but reports 500 trailing server fragment bytes. A partial JSONL tail is ignored. Deterministic blocked-write backpressure returns from submit under 0.1s, explicitly reports overflow, and the stopped writer permits all memory-captured frames to be written afterward. An injected write error is reported without publishing its frame. Nonpositive/unbounded queues and an existing capture are rejected. The final authoritative write and decoder output match the original finalized oracle exactly. Source manifest/proof hashes remain unchanged. No game was launched and no save was accessed.

Removing exactly the five opt-in runner additions reconstructs the previously reviewed runner SHA256 `62580bc644b6667499b3f8882926a1fe3bb201869233b691c5265617a427f4ea`. The final runner SHA256 is `7b60b07f5e00a32c94ddb58f937e563af988f6273e2c6472f3ed4c06ba2a1de3`. New helper SHA256 is `e374d4645850be33892d3920de8fa59b9ddc45db01eb0c2a2f6f9e805584cd80`.

Evidence: `lab/reports/check-native-live-capture.py`, `lab/reports/native-live-capture-proof.json` SHA256 `34c613ab895d731c2901353f93f0f1f29bf15dc1a81a79ee963b4bf042711c36`. Four Python files compile. Native live-capture acceptance remains pending the next real run; this offline proof does not establish scheduling/latency under a live game load.
