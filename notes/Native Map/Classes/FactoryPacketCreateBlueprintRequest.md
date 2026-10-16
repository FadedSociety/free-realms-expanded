# Class: FactoryPacketCreateBlueprintRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e1440` | `FactoryPacketCreateBlueprintRequest::Ctor` | ✅ | ctor | Initializes base packet, installs request vtable and constructs embedded ItemRequestEntry list. |
| `009e14b0` | `FactoryPacketCreateBlueprintRequest::Dtor` | ✅ | dtor | Calls 009df460 with ECX=this+0x0c to destroy the embedded ItemRequestEntry list, then restores BaseFactoryPacket and BasePacket vtables at this. |
| `009e4060` | `FactoryPacketCreateBlueprintRequest::ScalarDeletingDtor` | ✅ | dtor | Calls packet/list destructor and frees this when deleting flag is set. |
