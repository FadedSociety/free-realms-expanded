# Class: GameCommerce::StoreBillboardDefinition<AppStoreBillboardPanelDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c37430` | `StoreBillboardDefinitionDtor` | ✅ | dtor | Installs the derived billboard-definition vtable, destroys its array member, then restores the BaseDefinition vtable. |
| `00c37490` | `StoreBillboardDefinition::IsValid` | ✅ | method | Requires positive fields at offsets 4, 8 and 0x14, then calls each panel's first virtual method and returns false if any panel rejects. |
| `00c375d0` | `StoreBillboardDefinition::ScalarDeletingDtor` | ✅ | dtor | Destroys the billboard definition and frees it through 00401c00 with alignment 0 when the low flag bit is set. |
| `00c380a0` | `StoreBillboardDefinitionScalarDeletingDtor ?` | 🟡 | dtor | Calls the billboard-definition destructor and frees the object allocation when the low flag bit is set. |
| `00c384e0` | `StoreBillboardDefinitionCtor` | ✅ | ctor | Installs the BaseDefinition and derived billboard vtables, clears two fields, and constructs the panel array member. |
| `00c38cb0` | `StoreBillboardFormatDebug` | ✅ | virtual | Formats each panel into a temporary string, appends those strings, then formats billboard id, location id, panel count and panel text into the output string. |
| `00c38f80` | `StoreBillboardDefinitionSetId` | 🟢 | method | Constructs a billboard definition node, copies the input integer into field offset 0x198, and returns this. |
