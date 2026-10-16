# Class: ClientAreaProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be90d0` | `ClientAreaProperty::GetFloat24` | ✅ | virtual | Returns the float at this+0x24 (mov eax,[ecx+0x24]; ret); no other effect. |
| `00be90e0` | `ClientAreaProperty::DtorBody` | ✅ | dtor | Stores the ClientAreaProperty vtable at *this and returns; no members are released. |
| `00be9110` | `ClientAreaProperty::ScalarDtor` | 🟢 | dtor | Stores the ClientAreaProperty vtable at *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00cee0d0` | `ClientAreaProperty::BaseCtor` | ✅ | ctor | Stores the ClientAreaProperty vtable pointer at the beginning of the supplied object. |
