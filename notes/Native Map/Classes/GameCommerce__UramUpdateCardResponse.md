# Class: GameCommerce::UramUpdateCardResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6b060` | `GameCommerce::UramUpdateCardResponse::~UramUpdateCardResponse` | ✅ | dtor | Writes the UpdateCard response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b6b0` | `GameCommerce::UramUpdateCardResponse::DeletingDestructor` | ✅ | dtor | Writes the UpdateCard response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e320` | `GameCommerce::UramUpdateCardResponse::UramUpdateCardResponse` | ✅ | ctor | Initializes UramResponseBase, installs the UpdateCard response vtable, zeroes the response's trailing fields, and returns this. |
