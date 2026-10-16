# Class: ValueData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127a3e0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores PersistentBase::vftable, PersistentComponent::vftable then ValueData::vftable into *this; sets the type tag at +4 to 1, clears the value at +8 and the byte at +8 of the ValueData, then calls FUN_0130… |
| `0127a480` | `ValueData::GetClassName` | ✅ | virtual | Returns the constant C string ValueData (6 bytes: mov eax, imm32; ret). Virtual slot [11] of ValueData::vftable. |
| `0127a490` | `ValueData::GetGlobal_01cbda60 ?` | 🟡 | virtual | Returns the 32-bit global DAT_01cbda60 (6 bytes: mov eax, [01cbda60]; ret). Virtual slot [12] of ValueData::vftable. |
| `0127a4a0` | `ValueData::SetGlobal_01cbda60 ?` | 🟡 | virtual | Stores its single stack argument into the global DAT_01cbda60 and returns, popping 4 bytes (12 bytes: mov eax, [esp+4]; mov [01cbda60], eax; ret 4). Virtual slot [13] of ValueData::vftable. |
| `0127a4c0` | `ValueData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls FUN_01300cd0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. Returns with ret 4 (one stack argument, the delete flag), decoded from the raw bytes. |
