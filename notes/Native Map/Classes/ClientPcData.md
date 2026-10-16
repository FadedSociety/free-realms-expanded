# Class: ClientPcData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a10a80` | `ClientPcData::SetField_0x34f80` | ✅ | method | Stores param_2 into the dword field at this+0x34f80 and returns. |
| `00a10a90` | `ClientPcData::SetField_0x34f7c` | ✅ | method | Stores param_2 into the dword field at this+0x34f7c and returns. |
| `00a1d6d0` | `ClientPcData::Dtor` | 🟢 | dtor | Installs ClientPcData vftable and destroys profile, effect tags, character stats, lists, guild/name state, event queues, references, and embedded managers. |
| `00a1db70` | `ClientPcData::GetField_0x46c` | 🟢 | virtual | Returns the dword stored at this+0x46c. |
| `00a1de60` | `— ⚠reject` | ⚪ | ctor | ClientPcData constructor: stores ClientPcData::vftable, calls base constructor 00a1c160, saves the BaseClient pointer at +0x3bc, initializes embedded subsystems and the NameChangeInfo list, then makes 13 allocations thr… |
