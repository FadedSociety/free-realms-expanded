# Class: GameCommerce::UramManager::UramAddCreditCardRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6a9c0` | `GameCommerce::UramManager::UramAddCreditCardRequest::~UramAddCreditCardRequest` | ✅ | dtor | Installs the request vtable, releases its fixed-string members and request state, then restores the UramRequestTracker vtable. |
| `00d6b540` | `GameCommerce::UramManager::UramAddCreditCardRequest::UramAddCreditCardRequest` | ✅ | ctor | Calls the UramRequestTracker constructor with type value 2, installs the AddCreditCardRequest vtable, initializes request state, and sets up fixed-24 and fixed-16 strings. |
| `00d6b5f0` | `GameCommerce::UramManager::UramAddCreditCardRequest::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6a9c0 as the request destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
