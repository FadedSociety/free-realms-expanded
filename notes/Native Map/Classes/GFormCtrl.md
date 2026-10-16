# Class: GFormCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0043edc1` | `GFormCtrl::ProcessControlMessage` | ✅ | handler | Reads the 16-bit message id at param_2+0x1C and compares it with global message-id constants. It routes matched ids to this object vtable slots from +0x130 through +0x24C or to helpers such as 0043833B, 004072E2, 0042F0… |
| `00609fee` | `GFormCtrl::FlagsE8Or220000` | 🟢 | virtual | Thiscall without stack arguments: returns the dword this+0xe8 OR 0x220000. |
