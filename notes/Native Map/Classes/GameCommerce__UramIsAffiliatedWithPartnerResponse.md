# Class: GameCommerce::UramIsAffiliatedWithPartnerResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6b090` | `GameCommerce::UramIsAffiliatedWithPartnerResponse::~UramIsAffiliatedWithPartner…` | ✅ | dtor | Releases the response's fixed-8 string, resets its buffer and lengths, and calls the UramResponseBase destructor. |
| `00d6b740` | `GameCommerce::UramIsAffiliatedWithPartnerResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6b090 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e410` | `GameCommerce::UramIsAffiliatedWithPartnerResponse::UramIsAffiliatedWithPartnerR…` | ✅ | ctor | Initializes UramResponseBase, installs the affiliation response vtable, clears its flag, initializes one fixed-8 string, and returns this. |
