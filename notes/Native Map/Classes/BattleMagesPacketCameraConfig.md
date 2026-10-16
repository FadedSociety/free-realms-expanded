# Class: BattleMagesPacketCameraConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b70a70` | `BattleMagesPacketCameraConfig::~BattleMagesPacketCameraConfig` | 🟢 | dtor | Destructor body of the camera-config sub-packet: calls 00b70a00 with this+0x10, which releases the SoeUtil::IString<char> embedded at this+0x30, then overwrites +0 with BaseBattleMagesPacket::vftable and finally BasePac… |
| `00b70c20` | `BattleMagesPacketCameraConfig::BattleMagesPacketCameraConfig` | ✅ | ctor | Constructor for sub-opcode 8: calls the shared base constructor with 8, stores BattleMagesPacketCameraConfig::vftable at +0, zeroes the field at +0xc and constructs the embedded camera at +0x10 (FUN_0107b840 initialises… |
| `00b70c90` | `BattleMagesPacketCameraConfig::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the camera-config packet's vtable slot 0: calls the destructor body 00b70a70 (releases the camera's SoeUtil::IString<char> and restores the base vtables) and then calls _free(this) when bi… |
| `00b71000` | `BattleMagesPacketCameraConfig::ReadBody ⚠downgrade` | 🟢 | method | Reads eight 32-bit floats into destination offsets +0..+0x1c, setting reader+0x11 if any is NaN. Then reads a signed length; if nonnegative and within the remaining bytes, calls 00894b10 with the blob and length and adv… |
