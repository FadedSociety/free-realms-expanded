# Class: SoeUtil::HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107fc90` | `HashListMap64Ctor` | 🟢 | ctor | Constructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, zeroes this+0x10, +0x8, +0xc and +0x4, and clears the 64-bucket array at this+0x14 (_memset 0x100 bytes). |
| `01081a40` | `HashListMapSnsDtorBody` | 🟢 | dtor | Destructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, removes nodes from the list head through 00bb8000 until the head is null. Called from two EH unwind funclets. |
| `01081bb0` | `HashListMapSnsScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the vftable, removes list nodes through 00bb8000 until the head is null, frees this with _free when flag bit 0 of the argument is set, and returns this. |
| `01081bf0` | `HashListMapSnsDtorBody` | 🟢 | dtor | Destructor body (EH variant): stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable and removes list nodes through 00bb8000 until the head is null. Same body as 01081a40. |
