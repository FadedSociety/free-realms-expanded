# Class: CWCampaignMap::CriticalSection

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012516a0` | `CWCampaignMap::CriticalSection::Dtor` | 🟢 | dtor | 7 bytes: c7 01 imm32 c3. Stores CWCampaignMap::CriticalSection::vftable into the first dword of *this and returns; no free. |
| `012516f0` | `CWCampaignMap::CriticalSection::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of CWCampaignMap::CriticalSection::vftable, 28 bytes. Stores the vtable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
