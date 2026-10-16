# Class: Engine::CollectionRespondTickets

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277340` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionRespondTickets::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012773d0` | `Engine::CollectionRespondTickets::DebugName` | ✅ | virtual | Returns the constant C string kUserDataCommandRespondTicketBalanceDisplayAction : 0 : ... (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionRespondTickets::vftable. |
| `012773e0` | `Engine::CollectionRespondTickets::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `01278db0` | `Engine::CollectionRespondTickets::Handle` | ✅ | virtual | Virtual slot [2] of its vtable (241 bytes). Reads ValueData entry 0 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 and 0), builds the RespondTickets command with FUN_012ec220, sets… |
