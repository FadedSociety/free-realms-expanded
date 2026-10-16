# Class: TargetProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acb490` | `NotifyHandlers` | 🟢 | method | Stores the event value at this+0x20 and iterates the handler chain resolved from this+0x7c (next pointer at element+8): each handler's virtual slot +8 is called with (this+0x20, this+0x24); handlers that return true are… |
| `00acb770` | `TargetProcessor::TargetProcessor` | ✅ | ctor | Stores the two constructor arguments at +0x10/+0x14, installs TargetProcessor::vftable, zeros state through +0x50, and initializes float +0x54 to zero. It installs the embedded handler-list vtable at +0x78, clears head/… |
| `00acb8c0` | `TargetProcessor::Destructor` | ✅ | dtor | Non-deleting destructor: resets *this to TargetProcessor::vftable, reinstalls the embedded list vtable at this+0x78 and removes every handler node through FUN_00acb660 (SEH-framed around the call). |
| `00acb920` | `— ⚠reject` | ⚪ | method | Scans the 0x30-byte array using base +4, count +8 and stride 0x30. In a preliminary pass enabled when param_6==0 and the context at this+0x14+200 exists, it removes type-0 records only when actor+0x1c exists and its +0x… |
| `00acbc60` | `TargetProcessor::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (27 bytes): calls the non-deleting destructor 00acb8c0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00acbc80` | `— ⚠reject` | ⚪ | method | Adds the time delta to this+0x98 and delta*DAT_017ebf0c to float +0x54. If state byte +0x88 is clear and a target at +0x28 is set, it clears the selected target and movement fields; otherwise it updates orientation. Whe… |
| `00acc160` | `— ⚠reject` | ⚪ | method | Mirrors input byte param_2+4 bits 8, 0x20 and 0x10 into state +0x24 bits 8, 0x10 and 0x20. If this+0x28 is nonzero, maps kinds 6/7/8/0xc/0xd/0xe/0xf/0x10/0x11 to event masks 1/4/2/8/0x20/0x10/0x40/0x100/0x80; kind 0xc a… |
