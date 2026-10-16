# Class: GameCommerce::UramDeleteCardResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6b070` | `GameCommerce::UramDeleteCardResponse::~UramDeleteCardResponse` | ✅ | dtor | Writes the DeleteCard response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b6e0` | `GameCommerce::UramDeleteCardResponse::DeletingDestructor` | ✅ | dtor | Writes the DeleteCard response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e370` | `GameCommerce::UramDeleteCardResponse::UramDeleteCardResponse` | ✅ | ctor | Initializes UramResponseBase, installs the DeleteCard response vtable, zeroes the response's trailing fields, and returns this. |
