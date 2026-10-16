# Class: GameCommerce::UramResponseBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6aab0` | `GameCommerce::UramResponseBase::~UramResponseBase` | ✅ | dtor | Installs the response-base vtable, decrements the fixed-256 string reference count, resets its buffer and lengths, and restores the IString vtable. |
| `00d6b610` | `GameCommerce::UramResponseBase::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6aab0 as the response-base destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6df20` | `GameCommerce::UramResponseBase::UramResponseBase` | ✅ | ctor | Installs the response-base vtable, initializes its default status fields and fixed-256 string, zeroes two trailing fields, and returns this. |
