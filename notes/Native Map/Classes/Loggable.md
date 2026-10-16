# Class: Loggable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00720bc0` | `Loggable::~Loggable` | 🟢 | dtor | Writes Loggable::vftable into *param_1 and returns; it has no other body, and every caller is an MSVC EH unwind funclet. |
| `00720bd0` | `Loggable::ScalarDeletingDestructor` | ✅ | dtor | Installs Loggable::vftable into *param_1 and, when bit 0 of the flag byte is set, calls _free on the object; returns the object pointer. |
| `00720c80` | `SetLoggableVtable ?` | 🟡 | function | Under an SEH frame, writes Loggable::vftable into *param_1 and returns; byte-equivalent trivial body to 00720bc0 with an unwind handler installed. |
| `007226c0` | `ScalarDeletingDtor_Loggable` | ✅ | dtor | Under an SEH frame installs Loggable::vftable into this and frees the object through FUN_00401c00 when bit 0 of the flag byte is set; returns this. |
| `0090ba10` | `LoggableSubobjectCleanup ?` | 🟡 | dtor | Replaces the vtable at this+0x1fc with Loggable::vftable, then runs helpers 00907d60 and 008c70f0 under an exception frame. |
| `00cef230` | `BrokerLoggableSubobjectDtor` | ✅ | dtor | Cleans the embedded logging subobject under an exception frame, then restores its Loggable vtable at object offset 0x28. |
