# Class: InGamePurchaseDataSource::CreditCardPayment

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997e50` | `CreditCardPaymentFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of CreditCardPayment, one stack arg (index), no calls. Switch: 0 PaymentSourceId, 1 CardTypeId, 2 CardSpan, 3 CanDelete, 4 AccountText; any other index returns "<error>". |
| `0099f4d0` | `CreditCardPayment::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps to 0099D680, a callback that appends payment entries. |
| `009a4450` | `CreditCardPayment::GetValue` | ✅ | virtual | Resolves a selected payment record and handles field selectors 0 through 3, writing stored values or string fields to the requested output. |
| `009a7280` | `InGamePurchaseDataSource::CreditCardPayment::~CreditCardPayment` | ✅ | dtor | Destructor body: writes CreditCardPayment::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a7de0` | `InGamePurchaseDataSource::CreditCardPayment::CreditCardPayment` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs CreditCardPayment::vftable at [0], [0xc], [0xd], [0x95] and zeroes [0x9… |
