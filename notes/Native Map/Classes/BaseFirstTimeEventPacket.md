# Class: BaseFirstTimeEventPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c880b0` | `BaseFirstTimeEventPacketCtor` | ✅ | ctor | Installs BasePacket then BaseFirstTimeEventPacket vtables, stores family opcode 0x6b, and saves the supplied subtype. |
| `00c885d0` | `BaseFirstTimeEventPacketScalarDeletingDtor` | ✅ | virtual | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00c897b0` | `— ⚠reject` | ⚪ | handler | Reads an inner subtype; for subtype 2 constructs and applies a state packet, and for subtype 5 parses a script entry for immediate dispatch. |
