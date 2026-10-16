# Class: PlayerUpdatePacketAddPc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091d000` | `— ⚠reject` | ⚪ | ctor | Constructor of the AddPc packet: runs FUN_0089fbb0(1) (BasePlayerUpdatePacket base with sub-opcode 1), installs PlayerUpdatePacketAddPc::vftable, then FUN_0070bac0 and FUN_008f20c0 initialize the attachment list at +0xd… |
| `0091d1c0` | `PlayerUpdatePacketAddPc::~PlayerUpdatePacketAddPc` | 🟢 | dtor | Destructor of the AddPc packet: drains the EffectTag list at +0x180 (FUN_008cace0), tears down the member at +0x170 (FUN_008eaf50), releases the six IString fields, the attachment list at +0xd0, then restores BasePlayer… |
| `00922980` | `PlayerUpdatePacketAddPc_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091d1c0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
