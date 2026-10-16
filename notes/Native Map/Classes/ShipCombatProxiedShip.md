# Class: ShipCombatProxiedShip

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c99ea0` | `ShipCombatProxiedShip::DispatchSlot1` | ✅ | virtual | Loads the receiver's vtable, reads slot one, and tail-jumps to that virtual method. |
| `00c9a170` | `ShipCombatProxiedShip::GetField90 ?` | 🟡 | virtual | Calls a helper to retrieve a global object, then returns its 32-bit field at offset +0x90. |
| `00ca1440` | `ShipCombatProxiedShip::Dtor` | ✅ | dtor | Restores the ShipCombatProxiedShip vtables, releases held references and owned arrays, clears the hold UI records, releases strings and shared state, then unregisters the asset requester. |
| `00ca17c0` | `— ⚠reject` | ⚪ | dtor | Calls the full object destructor at 00ca2100 and returns its result. |
| `00ca1d70` | `ShipCombatProxiedShip::Ctor` | ✅ | ctor | Initializes the primary and asset-requester subobject vtables, binds the ship's owner context, zeros runtime fields and initializes the embedded string, hold array, and state blocks. |
| `00ca2100` | `ShipCombatProxiedShip::DeletingDtor → ShipCombatProxiedShip::ScalarDeletingDtor` | ✅ | dtor | Runs 00ca1440, frees the object when the deleting flag's low bit is set, and otherwise returns the object pointer. |
| `00ca2120` | `ShipCombatProxiedShip::InitializeState` | ✅ | virtual | Copies the source state, copies 16 dwords into the embedded block, registers multiple references, initializes a 0xb0-byte object, and attaches matching linked entries. |
| `00ca2470` | `ShipCombatProxiedShip::Update` | 🟢 | virtual | Converts the update delta, advances timers and state helpers, updates transform and orientation state, selects movement animations from velocity thresholds, and updates attached effects. |
| `00ca4510` | `ShipCombatProxiedShip::ScalarDeletingDtor` | 🟢 | dtor | Calls the ShipCombatProxiedShip destructor body at 00ca1440, frees the allocation when requested, and otherwise returns the object pointer. |
