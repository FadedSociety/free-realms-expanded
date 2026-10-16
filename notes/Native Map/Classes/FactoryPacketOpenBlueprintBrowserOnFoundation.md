# Class: FactoryPacketOpenBlueprintBrowserOnFoundation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e1520` | `FactoryPacketOpenBlueprintBrowserOnFoundation::Dtor` | ✅ | dtor | Calls 0081ca60 with ECX=this+0x6c and 009e13d0 with ECX=this+0x18 to destroy the embedded maps, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e4210` | `FactoryPacketOpenBlueprintBrowserOnFoundation::Ctor` | ✅ | ctor | Initializes BaseFactoryPacket with subopcode 0x18, installs the packet vtable, and initializes the 16-bucket map at +0x18 and 128-bucket map at +0x6c. |
| `009e4280` | `FactoryPacketOpenBlueprintBrowserOnFoundation::ScalarDeletingDtor ? ⚠rename` | 🟡 | dtor | Calls packet destructor and frees this when deleting flag is set. |
