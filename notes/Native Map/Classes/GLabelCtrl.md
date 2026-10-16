# Class: GLabelCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00609d77` | `GLabelCtrl::ResetStatePost7 ? ⚠downgrade` | 🟡 | virtual | Thiscall with two unused stack arguments (ret 8). Calls this->vtable[0x8a](0,0) (slot 138) and then 00411506(7) (PostStateChange) with ECX=this. |
| `00609de0` | `— ⚠reject` | ⚪ | virtual | Thiscall with four stack arguments (ret 0x10). Calls 00609bb3(this+0xe8 dword, 1), then 0043d515 with three stack words A2, A3, A4; A1 is not forwarded. Ghidra prints four forwarded parameters for the second call. |
| `006153ea` | `GLabelCtrl::StyleWordFromEc` | 🟢 | virtual | Thiscall without stack arguments. Reads this+0xec. When bit 0x10 is set and the 0xc0 field is 0xc0 or 0x80, returns 0x220000 OR a word built from 00609c78 (returns this+0x14c) and 00609c57 (returns this+0x148), with a 0… |
