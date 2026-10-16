# Class: BaseBrokerPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bee590` | `— ⚠reject` | ⚪ | ctor | Installs BasePacket then BaseBrokerPacket vtables, writes 0x82 to field 1, and initializes the subtype field to zero. |
| `00bee5f0` | `BaseBrokerPacket::CtorWithSubtype` | ✅ | ctor | Installs BasePacket and BaseBrokerPacket vtables, writes 0x82 to field 1, and stores the supplied subtype in field 2. |
| `00bee660` | `BaseBrokerPacket::Dtor` | ✅ | dtor | Restores BaseBrokerPacket and then BasePacket vtables during destruction; no owned payload fields are released here. |
| `00bee6b0` | `BaseBrokerPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BaseBrokerPacket and BasePacket vtables, frees the object if flag bit 0 is set, and otherwise returns its pointer. |
| `00bee8c0` | `BaseBrokerPacket::DtorException` | ✅ | dtor | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |
| `00bee970` | `BaseBrokerPacket::DtorException` | ✅ | dtor | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |
| `00bee9c0` | `BaseBrokerPacket::DtorException` | ✅ | dtor | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |
| `00c0c6e0` | `BaseBrokerPacket_DtorBody` | ✅ | dtor | Base-class destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; called from the Broker.Search handler 00c0e110 and from the unwind funclet 015f5018. |
| `00c0c7a0` | `BaseBrokerPacket_DtorBody_00c0c7a0` | ✅ | dtor | Copy of the BaseBrokerPacket destructor body in this translation unit: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from the Broker.Place path 00c0dcd0 and from the unwind funclet 0… |
| `00c0c7f0` | `BaseBrokerPacket_DtorBody_00c0c7f0 → BrokerReListItemPacket_DtorBody_00c0c7f0` | ✅ | dtor | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from the Broker.Relist path 00c0e000 and from the unwind funclet 015f4fe0. |
| `00c0c840` | `BaseBrokerPacket_DtorBody_00c0c840` | ✅ | dtor | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached from the Broker.Buy path 00c0df30 and from the unwind funclet 015f4fb0. |
| `00c0c890` | `BaseBrokerPacket_DtorBody_00c0c890 ⚠downgrade` | 🟢 | dtor | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached only from the unwind funclet 015f4f30 in this translation unit. |
| `00c0c8e0` | `BaseBrokerPacket_DtorBody_00c0c8e0 ⚠downgrade` | 🟢 | dtor | Copy of the BaseBrokerPacket destructor body: rewrites the vtable to BaseBrokerPacket::vftable then BasePacket::vftable; reached only from the unwind funclet 015f4f70 in this translation unit. |
| `00cee280` | `BaseBrokerPacketDtor` | ✅ | dtor | Resets the object's vtable first to BaseBrokerPacket and then to BasePacket. |
| `00cee340` | `BaseBrokerPacketDtor` | ✅ | dtor | Resets the object's vtable first to BaseBrokerPacket and then to BasePacket. |
