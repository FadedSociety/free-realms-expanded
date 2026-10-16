# Class: OverHeadElement

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00957850` | `OverHeadElementCtor ?` | 🟡 | ctor | Installs OverHeadElement vtable, initializes several scalar fields, copies supplied vectors/values into offsets 0x14 through 0x4c, and stores static defaults. |
| `009578e0` | `OverHeadElementDtor ?` | 🟡 | dtor | Writes OverHeadElement::vftable to the object and returns. |
| `009578f0` | `OverHeadElement::GetOffset14 ?` | 🟡 | virtual | Returns this pointer plus 0x14. |
| `00957900` | `OverHeadElement::GetOffset18` | 🟢 | virtual | Returns this pointer plus 0x18. |
| `00957910` | `OverHeadElement::GetOffset40` | 🟢 | virtual | Returns this pointer plus 0x40. |
| `00957920` | `OverHeadElement::SetField40 ?` | 🟡 | virtual | Copies one dword from the input pointer to this+0x40. |
| `00957930` | `OverHeadElement::GetOffset30 ?` | 🟡 | virtual | Returns this pointer plus 0x30. |
| `00957940` | `OverHeadElement::SetField30Block ?` | 🟡 | virtual | Copies four dwords from the input into object offsets 0x30, 0x34, 0x38, and 0x3c. |
| `00957960` | `OverHeadElement::GetOffset44 ?` | 🟡 | virtual | Returns this pointer plus 0x44. |
| `00957970` | `OverHeadElement::SetFloat44 ?` | 🟡 | virtual | Loads and stores one floating-point value at this+0x44. |
| `00957980` | `OverHeadElement::GetOffset48 ?` | 🟡 | virtual | Returns this pointer plus 0x48. |
| `00957990` | `OverHeadElement::SetFloat48 ?` | 🟡 | virtual | Loads and stores a floating-point value at this+0x48. |
| `009579a0` | `OverHeadElement::SetByte4c ?` | 🟡 | virtual | Copies one byte from input to this+0x4c. |
| `009579b0` | `OverHeadElement::GetOffset4c ?` | 🟡 | virtual | Returns this pointer plus 0x4c. |
| `009579c0` | `OverHeadElement::GetOffset4d ?` | 🟡 | virtual | Returns this pointer plus 0x4d. |
| `009579d0` | `OverHeadElement::SetField20Block ?` | 🟡 | virtual | Copies four dwords from input offsets 0,4,8,0xc into object offsets 0x20,0x24,0x28,0x2c. |
| `009579f0` | `OverHeadElement::SetFloat34 ?` | 🟡 | virtual | Moves one scalar float into this+0x34. |
| `00957a00` | `OverHeadElement::GetFloat34 ?` | 🟡 | virtual | Returns the float stored at this+0x34. |
| `00957a10` | `OverHeadElement::ScalarDeletingDtor ?` | 🟡 | dtor | Installs the OverHeadElement vtable, then frees the object when deletion flag bit 0 is set; returns the object otherwise. |
