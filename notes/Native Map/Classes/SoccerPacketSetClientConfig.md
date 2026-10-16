# Class: SoccerPacketSetClientConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b76dc0` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 7 packet SoccerPacketSetClientConfig: calls the shared base ctor FUN_00b730a0(7), installs SoccerPacketSetClientConfig::vftable and initialises an embedded SoeUtil::IString<char> member … |
| `00b76e40` | `SoccerPacketSetClientConfig::~SoccerPacketSetClientConfig` | 🟢 | dtor | Non-deleting destructor of the sub-7 SetClientConfig packet: resets the embedded SoeUtil::IString vtable at +0x8c, and when its refcount (word before the buffer at +0x90) reaches zero calls the string release slot +8 un… |
| `00b77cc0` | `SoccerPacketSetClientConfig::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the class destructor body FUN_00b76e40 (releases the embedded SoeUtil::IString refcount then resets base vtables) and, when the flag byte's low bit is set, frees the object with _free. |
