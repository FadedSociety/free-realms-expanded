# Class: GameCommerce::ImageDataDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c39b90` | `ImageDataDefinition::ReadImageFields` | 🟢 | method | Reads image/graphics fields of a definition from a data source: getter 00d63820(this=source,"image",row) result is assigned (IStringAssign 00402ce0) into the IString at this+4, "image_tint" into the IString at this+0x14… |
| `00d67250` | `ImageDataDefinition::Dtor ?` | 🟡 | dtor | Installs the image-definition vtable, releases its two shared string fields, then restores the BaseDefinition vtable. |
| `00d672d0` | `ImageDataDefinition::HasImage ?` | 🟡 | virtual | Returns whether the 32-bit field at object offset 0x0c is greater than zero. |
| `00d672e0` | `ImageDataDefinition::Ctor ?` | 🟡 | ctor | Installs the image-definition vtable, initializes two no-share string members to the shared empty pointer, and clears their lengths. |
| `00d67310` | `ImageDataDefinition::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the image-definition destructor and releases the object with alignment zero when the deleting flag is set. |
| `00d67360` | `ImageDataDefinition::Format ?` | 🟡 | virtual | Formats the image and tint string fields using image=%s, tint=%s and returns the formatter's result pointer. |
| `00d67390` | `ImageDataDefinition::CtorFromInput ?` | 🟡 | ctor | Initializes image definition and its two no-share strings, then copies the two input strings at offsets +4 and +0x14. |
