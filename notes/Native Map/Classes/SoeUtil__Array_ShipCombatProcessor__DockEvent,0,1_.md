# Class: SoeUtil::Array<ShipCombatProcessor::DockEvent,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a533a0` | `SoeUtil::Array_DockEvent_Swap ⚠downgrade` | 🟢 | virtual | Virtual swap hook (slot 3) of Array<ShipCombatProcessor::DockEvent,0,1>: with a zero flag it exchanges the buffer pointer at +4 and, when the values differ, XOR-exchanges the count dword at +8 and the capacity dword at … |
| `00a55980` | `SoeUtil::Array_DockEvent_Dtor` | ✅ | dtor | Installs the DockEvent array vtable, clears the count, frees its backing buffer with alignment four, and clears the storage pointer. |
| `00a563f0` | `SoeUtil::Array_DockEvent_ScalarDeletingDtor` | ✅ | dtor | Resets the DockEvent array vtable, clears count, frees the backing buffer with alignment four, then optionally frees the array object. |
