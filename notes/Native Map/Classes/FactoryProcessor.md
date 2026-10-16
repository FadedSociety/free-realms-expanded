# Class: FactoryProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e42a0` | `FactoryProcessor::Ctor` | ✅ | ctor | Installs processor vtable, initializes state and tooltip record, stores context, creates/retains two helpers and clears flags. |
| `009e4410` | `— ⚠reject` | ⚪ | dtor | Restores the FactoryProcessor vtable; for helper pointers at +0x23a4 and +0x23a8, sets ECX to each helper and calls virtual slot +0x28 with one zero argument, pushes returned EAX and calls 0081ad80 with ECX=DAT_01be1090… |
