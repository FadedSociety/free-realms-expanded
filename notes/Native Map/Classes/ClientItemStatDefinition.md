# Class: ClientItemStatDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008acd60` | `— ⚠reject` | ⚪ | ctor | Two-stage constructor: writes BaseItemStatDefinition::vftable at offset 0, zeroes +4/+8/+0xc and the byte at +0x10, then overwrites offset 0 with ClientItemStatDefinition::vftable; returns nothing. Called from 008bfba0. |
| `008ace20` | `ClientItemStatDefinition::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of ClientItemStatDefinition::vftable: rewrites BaseItemStatDefinition::vftable at offset 0 inside an SEH frame and, when bit 0 of the flag argument is set, frees this through the _fr… |
