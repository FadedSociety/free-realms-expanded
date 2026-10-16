# Class: GameCommerce::BaseDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997140` | `BaseDefinitionDtorBody` | 🟢 | dtor | Fastcall (ECX=this), 7 bytes: stores the GameCommerce::BaseDefinition vtable (0180d2a8) to [this] and returns (c7 01 imm32; c3). Releases nothing. |
| `0099d100` | `BaseDefinition::DtorBody → BaseDefinition::ScalarDeletingDtor` | ✅ | virtual | Restores the BaseDefinition vtable, conditionally frees the object when the deleting flag is set, and returns the object pointer. |
| `00b26120` | `GameCommerce::BaseDefinition::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls FUN_00d58300 with this in ECX (the 7-byte destructor body that stores GameCommerce::BaseDefinition::vftable) and frees this via _free when the flags low bit is set, returning this. |
| `00d58300` | `GameCommerce::BaseDefinition::Dtor` | ✅ | dtor | Bytes c7 01 a8 d2 80 01 c3: stores GameCommerce::BaseDefinition::vftable into *this and returns; no base call and no storage release. |
| `00d58520` | `GameCommerce::BaseDefinition::ScalarDeletingDtorAlign0 ⚠downgrade` | 🟢 | dtor | Stores GameCommerce::BaseDefinition::vftable into *this, then calls FreeMemory (FUN_00401c00) with alignment 0 when bit 0 of the flags argument is set; returns this. |
