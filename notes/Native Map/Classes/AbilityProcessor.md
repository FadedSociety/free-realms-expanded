# Class: AbilityProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a31860` | `— ⚠reject` | ⚪ | dtor | Destructor body. Restores AbilityProcessor::vftable at +0, calls 00a2e620, releases the owned objects at +0x74, +0x78 and +0x7c (each through 00acb430 when the +0x1a8 pointer of the owner is non-null, then vtable slot 0… |
| `00a324e0` | `AbilityProcessor::AbilityProcessor` | ✅ | ctor | Constructor. Stores AbilityProcessor::vftable at +0, the owner pointer at +4 and default fields (dwords at +8..+0x20 set to 0 or -1, bytes cleared at +0x24, +0x25, +0x60, +0x61, +0x62, bytes set to 1 at +0x63 and +0x70)… |
| `00a33760` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] of AbilityProcessor::vftable (binary read). Builds an AbilityPacketLaunchAndLand working object through 00a2f450, reads the payload through 00a31f30 and returns 0 unless the reader error byte is clear a… |
| `00a35150` | `AbilityProcessor::SendStartAbilityRequest` | 🟢 | method | Gated by 00a2e540, 00a30870 and the bytes at this+0x61/+0x62 and param_2. With this+0xc negative it only runs the +0x25 branch (00a2d4f0 ctor, then 00a2d550 dtor). Otherwise it picks a target: it scans the list at (*(th… |
| `00a355f0` | `AbilityProcessor::SendIfFlagAt24` | ✅ | virtual | Virtual slot [0] of AbilityProcessor::vftable (read from the binary). Tests the byte at this+0x24; when it is non-zero it calls 00a35150 with param_2=1. No dtor shape (no free, no flag bit test), so it is an ordinary vi… |
| `00a35600` | `AbilityProcessor::DispatchTargetModeAndSend ⚠downgrade` | 🟢 | method | Returns a byte (uVar6). Reads the owner object at this+4 and its field +200; takes the 1870 pointer and compares its first two dwords with DAT_01be6870/DAT_01be6874 to set bVar4. Calls 00a30870 as a gate; on failure cal… |
| `00a35cc0` | `AbilityProcessor::HandleAbilityPacketBySubOpcode` | ✅ | virtual | Virtual slot [2] of AbilityProcessor::vftable (binary vtable read). Entry: prologue with aligned stack; param_2/param_3 are a buffer pointer and its length. Calls 00a2cb10 (BaseAbilityPacket base ctor) and 00a2fc10 (unl… |
| `00a36a70` | `— ⚠reject` | ⚪ | method | Thiscall; param_2 is a slot id (-1 means none) and param_3 is a bar type. If the current slot (this+0xc) or the pending flag (this+0x24) is set, it resets state through 00a2e620 unless the stored slot and type already e… |
