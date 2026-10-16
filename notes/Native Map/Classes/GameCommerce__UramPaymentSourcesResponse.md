# Class: GameCommerce::UramPaymentSourcesResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6af30` | `GameCommerce::UramPaymentSourcesResponse::~UramPaymentSourcesResponse` | ✅ | dtor | Installs the PaymentSources response vtable, cleans its item collection and fixed-8 string, then calls the UramResponseBase destructor. |
| `00d6b670` | `GameCommerce::UramPaymentSourcesResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6af30 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e1d0` | `GameCommerce::UramPaymentSourcesResponse::UramPaymentSourcesResponse` | ✅ | ctor | Initializes UramResponseBase, installs the PaymentSources response vtable, clears its flags and fixed-8 string, and initializes the payment-source collection. |
