# Class: GameCommerce::UramManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68190` | `UramManager::LogNullDebug ? → UramManager::OnUploaderLogDebug` | 🟡 | virtual | When the message argument is null, calls the manager logger at this+0x10c with the fixed debug diagnostic; otherwise returns. |
| `00d681b0` | `UramManager::LogNullInfo ? → UramManager::OnUploaderLogInfo` | 🟡 | virtual | When the message argument is null, calls the manager logger with the fixed info diagnostic; otherwise returns. |
| `00d681d0` | `UramManager::LogNullError ? → UramManager::OnUploaderLogError` | 🟡 | virtual | When the error-message argument is null, calls the manager logger with the fixed error diagnostic; otherwise returns. |
| `00d6a3e0` | `GameCommerce::UramManager::~UramManager` | ✅ | dtor | Releases the manager's request map, embedded strings, helper state, and logging header while changing the vtable through the base teardown sequence. |
| `00d6a910` | `GameCommerce::UramManager::AdjustorThunkTo00d6b4f0` | ✅ | virtual | Raw bytes subtract 4 from ECX, then jump to 00d6b4f0; this function is an adjustor thunk and does not contain the target body. |
| `00d6b4f0` | `GameCommerce::UramManager::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6a3e0 as the manager destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6d1a0` | `GameCommerce::UramManager::UramManager` | ✅ | ctor | Initializes the UploaderHandler and UramManager vtables, clears manager flags and request-map buckets, initializes its string members, writes 60000 at +0x28, assigns the supplied string, and registers the manager log he… |
| `00d6e920` | `— ⚠reject` | ⚪ | handler | Finds the request by tracking ID in its keyed bucket, formats the result, builds a PaymentSources response, copies successful entries into manager state, invokes the request callback, unlinks the request and releases te… |
| `00d6eba0` | `GameCommerce::UramManager::HandleFundWalletPreviewResponse` | ✅ | handler | Finds the pending request by tracking ID, formats the result, copies successful response fields into a typed preview response, invokes the request callback, removes tracking state and releases temporary objects. |
| `00d6ee10` | `GameCommerce::UramManager::HandleFundWalletResponse` | ✅ | handler | Finds the pending request by tracking ID, formats the result, copies successful response fields into a typed wallet-fund response, invokes the request callback, removes tracking state and releases temporary objects. |
| `00d6f0a0` | `— ⚠reject` | ⚪ | handler | Finds the pending request by tracking ID, formats the result, passes successful response strings to a state helper, invokes the request callback, removes tracking state and releases temporary objects. |
