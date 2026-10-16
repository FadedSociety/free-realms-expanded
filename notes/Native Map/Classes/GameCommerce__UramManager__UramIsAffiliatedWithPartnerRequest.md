# Class: GameCommerce::UramManager::UramIsAffiliatedWithPartnerRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68bf0` | `UramIsAffiliatedWithPartnerRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 13, installs the partner-affiliation request vtable, and constructs its payload. |
| `00d68c60` | `UramIsAffiliatedWithPartnerRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases the request payload through 00f80990, then restores the base tracker vtable. |
| `00d6a1b0` | `UramIsAffiliatedWithPartnerRequest::~UramIsAffiliatedWithPartnerRequest → GameC…` | ✅ | dtor | Calls FUN_00d68c60 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
