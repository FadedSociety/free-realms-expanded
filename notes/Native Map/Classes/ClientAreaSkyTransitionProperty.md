# Class: ClientAreaSkyTransitionProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ceda00` | `ClientAreaSkyTransitionProperty::WriteFields` | ✅ | virtual | Writes the base client-area property fields through the property serializer. |
| `00ceda60` | `ClientAreaSkyTransitionProperty::Ctor` | ✅ | ctor | Initializes base property fields, stores constructor arguments, sets default values and installs the ClientAreaSkyTransitionProperty vtable. |
| `00cedb10` | `— ⚠reject` | ⚪ | virtual | Reads a string through the virtual property accessor, applies a bounded 0x401 rolling hash over its nonzero bytes, then finalizes and returns the hash. |
| `00cee0c0` | `ClientAreaSkyTransitionProperty::GetTypeName` | ✅ | virtual | Returns a pointer to the constant string SkyTransition. |
| `00cee120` | `ClientAreaSkyTransitionProperty::ScalarDeletingDtor` | ✅ | dtor | Resets the base ClientAreaProperty vtable and frees the object when the deleting flag bit is set; otherwise returns the object pointer. |
