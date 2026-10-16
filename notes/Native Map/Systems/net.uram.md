# System: net.uram

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00f889d0` | `FundWalletResponse::Parse` | FundWalletResponse | parses the FundWalletResponse XML reply of the Uram web service into the response object | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89280` | `CreditCardTypesResponse::Ctor` | CreditCardTypesResponse | initialises the CreditCardTypes response record (card-type list and error list) | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89a60` | `PurchaseOrderResponse::Ctor` | PurchaseOrderResponse | initialises the PurchaseOrderResponse fields: string members, subscription-product list and error list | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8a200` | `CountryCodesResponse::Ctor` | CountryCodesResponse | initialises the CountryCodes response record (country list and error list) | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8a3d0` | `UpdateCreditCardResponse::Parse` | UpdateCreditCardResponse | parses the updatecreditcardresponse XML reply | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8a790` | `DeletePaymentSourceResponse::Parse` | DeletePaymentSourceResponse | parses the deletepaymentsourceresponse XML reply | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8abe0` | `IsAffiliatedWithPartnerResponse::Parse` | IsAffiliatedWithPartnerResponse | parses the issteamcustomerresponse XML reply (IsAffiliatedWithPartner query) into the response object | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8b020` | `FinalizeThirdPartyOrderTransactionResponse::Parse` | FinalizeThirdPartyOrderTransactionRespo… | parses the finalizesteamtransactionresponse XML reply of a third-party order finalise | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8b780` | `PreviewOrderResponse::Ctor` | PreviewOrderResponse | initialises the PreviewOrderResponse fields (strings, subscription and software product lists, error list) | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8bf20` | `CountryCodesResponse::Parse` | CountryCodesResponse | parses the countrycodesresponse XML reply into the country list and error fields | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8c2f0` | `PreviewOrderResponse::Parse` | PreviewOrderResponse | parses the purchasepreviewresponse XML reply into the PreviewOrderResponse object | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00f87e70` | `ThreeStringsAndErrorArrayDtorBody` |  | teardown of an unbound Uram record holding three strings and an Uram::Error array (destructor body) | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f88020` | `TwoStringsAndErrorArrayDtorBody → FinalizeThirdPartyOrderTransactionResponse::D…` |  | teardown of an unbound Uram record holding two strings and an Uram::Error array (destructor body) | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f886b0` | `ParseFundWalletPreviewResponseXml` |  | parses the fundwalletpreviewresponse XML reply and reports whether it carried a response | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f892d0` | `CreditCardTypesChildWalk` |  | parses the card-type entries of a getcreditcardtypesresponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89e00` | `RedeemCodeChildWalkKey28` |  | parses the child block at key 28 of a redeemcoderesponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89f30` | `RedeemCodeChildWalkKey29` |  | parses the child block at key 29 of a redeemcoderesponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8a0b0` | `RedeemCodeResponseFields` |  | fills redeemcoderesponse fields from its child elements | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8ab50` | `ThreeStringsAndErrorArrayCtor → IsAffiliatedWithPartnerResponse::Ctor` |  | constructs an unbound Uram record with three strings and an Uram::Error list | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8b8a0` | `PurchasePreviewChildAppend → PreviewOrderSubscriptionProductArrayChildWalk` |  | for the purchasepreviewresponse child at key 14, builds one product element and appends it to a list | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8be50` | `RedeemCodeEntryFields` |  | fills redeem-code entry fields from child elements | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00f87df0` | `ResetStringsAndPopArray20 ?` |  | Resets a Uram response subobject to empty without freeing this: for the IString-shaped members at this+0x00 and this+0x10 it releases a refcounted buffer (header at buffer-4, last reference freed through the vtable slot… | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f87f70` | `ResetThreeStringsAndErrorArray ? → IsAffiliatedWithPartnerResponse::ResetFields` |  | Resets the same layout as 00f87e70 without destroying it: releases the buffers of the IString-shaped members at this+0x00, +0x10 and +0x20 and sets each to the empty sentinel with length and capacity zero, then zeroes t… | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f880f0` | `TwoStringsAndArrayReset ? → FinalizeThirdPartyOrderTransactionResponse::ResetFi…` |  | Resets the two-string record: releases the IString buffers at this+0x00 and this+0x10 (refcounted, freed through the vtable slot 2 on last reference), sets each to the empty sentinel 0x01cb4081 with length and capacity … | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8af90` | `TwoStringsAndErrorArrayCtor ? → FinalizeThirdPartyOrderTransactionResponse::Ctor` |  | Constructor for the two-string record with an error array: IString-shaped members at this+0x00 and +0x10 set to the empty sentinel 0x01cb4080 with length and capacity zero; SoeUtil::Array<Uram::Error> vtable at this+0x2… | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00f88320` | `— ⚠reject` |  | reads the errorreason text from the child elements of an XML error block | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f884c0` | `— ⚠reject` |  | fills the fields of a paymentinforesponse object from its child elements | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f88e50` | `— ⚠reject` | AddPaymentSourceResponse | parses the addpaymentsourceresponse XML reply | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f894a0` | `— ⚠reject` | PreviewOrderResponse | tears down the PreviewOrderResponse fields (strings, subscription and software product lists, error list) on unwind and destruction | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89730` | `— ⚠reject` |  | parses a child block (key 12) of a purchasepreviewresponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89860` | `— ⚠reject` |  | Clears a large record without destroying it: releases refcounted buffers of nine IString-shaped members at this+0x04, +0x14, ..., +0x84 (each set to empty sentinel 0x01cb4081 with length and capacity zero); pops the arr… | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89b10` | `— ⚠reject` |  | parses a child block (key 6) of a purchaseorderresponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f89cf0` | `— ⚠reject` |  | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against one key string at DAT_01b97628 with length DAT_01b9762c. On a match it builds four tempor… | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8a250` | `— ⚠reject` |  | parses a child block (key 4) of a countrycodesresponse element | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8b370` | `— ⚠reject` | CreditCardTypesResponse | parses the getcreditcardtypesresponse XML reply into the card-type list and error fields | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |
| `00f8b990` | `— ⚠reject` | PurchaseOrderResponse | parses the purchaseorderresponse XML reply into the PurchaseOrderResponse object | [[Native Map/Shards/S0539 00f87df0-00f8c800|S0539]] |

