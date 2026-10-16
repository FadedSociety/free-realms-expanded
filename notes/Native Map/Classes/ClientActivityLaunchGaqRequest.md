# Class: ClientActivityLaunchGaqRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb37b0` | `ClientActivityLaunchGaqRequest::VSlot0Decline` | 🟢 | virtual | Vtable slot 0 delegate (decoded: mov eax,[ecx+8]; mov ecx,[ecx+4]; mov ecx,[ecx+0x218]; push eax; call 009c1c60; ret): loads the manager at (*(this+4))+0x218 into ECX and calls 009c1c60 with the request id stored at thi… |
| `00bb37d0` | `ClientActivityLaunchGaqRequest::VSlot1Accept` | 🟢 | virtual | Vtable slot 1 delegate (same shape as slot 0): loads the manager at (*(this+4))+0x218 into ECX and calls 009c19a0 with the request id at this+8. |
| `00bb37f0` | `ClientActivityLaunchGaqRequest::VSlot2Decline2` | 🟢 | virtual | Vtable slot 2 delegate: identical to slot 0, it loads the manager at (*(this+4))+0x218 into ECX and calls 009c1c60 (decline) with the id at this+8. |
