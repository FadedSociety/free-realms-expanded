# Class: GameCommerce::UramAddNewCardResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6aff0` | `GameCommerce::UramAddNewCardResponse::~UramAddNewCardResponse` | ✅ | dtor | Installs the AddNewCard response vtable, releases the two string members through FUN_00d59180, then calls the response-base destructor. |
| `00d6b690` | `GameCommerce::UramAddNewCardResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6aff0 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e290` | `GameCommerce::UramAddNewCardResponse::UramAddNewCardResponse` | ✅ | ctor | Initializes UramResponseBase, installs the AddNewCard response vtable, and initializes its two string members through FUN_00d59310. |
