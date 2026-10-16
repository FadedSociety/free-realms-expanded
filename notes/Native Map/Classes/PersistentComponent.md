# Class: PersistentComponent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279390` | `PersistentComponent::DtorBody` | ✅ | dtor | Destructor body: stores PersistentComponent::vftable then PersistentBase::vftable into *this and returns. Called from 87 MSVC EH unwind funclets. |
| `01279400` | `PersistentComponent::GetClassName` | ✅ | virtual | Returns the constant C string PersistentComponent (6 bytes: mov eax, imm32; ret). Virtual slot [11] of PersistentComponent::vftable. |
| `01279410` | `PersistentComponent::GetGlobal_01cbdcf8 ?` | 🟡 | virtual | Returns the 32-bit global DAT_01cbdcf8 (6 bytes: mov eax, [01cbdcf8]; ret). Virtual slot [12] of PersistentComponent::vftable. |
| `01279420` | `PersistentComponent::SetGlobal_01cbdcf8 ?` | 🟡 | virtual | Stores its single stack argument into the global DAT_01cbdcf8 and returns, popping 4 bytes (12 bytes: mov eax, [esp+4]; mov [01cbdcf8], eax; ret 4). Virtual slot [13] of PersistentComponent::vftable. |
| `01279440` | `PersistentComponent::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores PersistentComponent::vftable then PersistentBase::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127a250` | `PersistentComponent::SetBaseVftables` | ✅ | method | Stores PersistentBase::vftable then PersistentComponent::vftable into *this and returns. Two-step base-phase vtable store. |
