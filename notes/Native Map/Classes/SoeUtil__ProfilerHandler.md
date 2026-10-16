# Class: SoeUtil::ProfilerHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d05090` | `ProfilerHandlerBaseDtorBody` | ✅ | dtor | Sets the SoeUtil::ProfilerHandler vtable at offset 0 and returns (7 bytes: mov [ecx],vtable; ret). Does not free the object. |
| `00d050a0` | `ProfilerHandlerScalarDtor` | ✅ | dtor | Scalar deleting destructor: stores the SoeUtil::ProfilerHandler vtable and calls _free on the object when bit 0 of the flag argument is set. |
