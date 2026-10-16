# Class: FactoryPacketCreateBlueprintResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d9480` | `FactoryPacketCreateBlueprintResponse::FactoryPacketCreateBlueprintResponse` | ✅ | ctor | Constructor for FactoryPacketCreateBlueprintResponse: calls FUN_009d90d0(0x19), installs FactoryPacketCreateBlueprintResponse::vftable and clears the byte at +0x0c. |
| `009db420` | `FactoryPacketCreateBlueprintResponse::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketCreateBluep… |
| `009e6a00` | `FactoryPacketCreateBlueprintResponse_Handler` | ✅ | handler | Constructs the FactoryPacketCreateBlueprintResponse parser object and reads one flag byte. A clean read returns 1; only a zero flag builds FactoryHandler:OnBlueprintCreateFailed and fires it through FUN_009797a0 with no… |
