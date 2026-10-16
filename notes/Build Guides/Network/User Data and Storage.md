---
type: build-guide
section: Network
updated: 2026-10-12
---

# User Data and Storage

Part of [[Build Guides]] › Network. Related: [[Command System]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

852 accepted functions: 427 confirmed, 242 likely, 116 speculative, 67 unknown. Network-linked (`wire`): 0.

Subsystems: `net.uram` (473), `net.userdata` (261), `net.userdata_command` (118)

### `net.uram`

**AccountPaymentInfoRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f797b0` | AccountPaymentInfoRequest::Validate | confirmed | Checks this+0x48 (url), this+0x58 (userAgent) and this+0x18 (sessionId) for non-null in that order. On the first null it builds a stack ObjectReportHandlerAdap… |

**AccountPaymentInfoResponse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f798c0` | AccountPaymentInfoResponse::Validate | confirmed | Checks this+0x3c (version) and this+0x2c (status) for non-null. On the first null it logs a level-2 error through 00f99fd0 with a stack ObjectReportHandlerAdap… |

**AddPaymentSourceRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f79d30` | AddPaymentSourceRequest::Validate | confirmed | Checks fields in order: url (this+0x108), userAgent (+0x118), sessionId (+0xb8), number (+0x98), type (+0xf8), expMonth (+0x48), expYear (+0x58), name (+0x88),… |

**AddPaymentSourceResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a000` | AddPaymentSourceResponse::Validate | confirmed | Checks this+0x28 (version) and this+0x18 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f88e50` | — | unknown | XML reply parser with the same structure as FundWalletResponse::Parse: loads the document (00d549d0), gets the root (00d50ac0), reports 'AddPaymentSourceRespon… |

**CasGetSessionTicketRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b2e0` | CasGetSessionTicketRequest::Validate | confirmed | Checks this+0x18 (url), this+0x28 (userAgent) and this+0x08 (sessionId) for non-null in that order; logs a level-2 error through 00f99fd0 on the first null and… |

**CasKeepAliveRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b150` | CasKeepAliveRequest::Validate | confirmed | Checks this+0x18 (url), this+0x28 (userAgent) and this+0x08 (sessionId) for non-null in that order; logs a level-2 error through 00f99fd0 on the first null and… |

**CasKeepAliveResponse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b260` | CasKeepAliveResponse::Validate | confirmed | Reads this+0x08 (status) and returns 0 after logging a level-2 error through 00f99fd0 when it is zero; otherwise returns 1. Only a status check; there is no ve… |

**CountryCodesRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7aa10` | CountryCodesRequest::Validate | confirmed | Checks this+0x28 (url), this+0x38 (userAgent) and this+0x08 (locale) for non-null in that order; on the first null logs a level-2 error through 00f99fd0 and re… |

**CountryCodesResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7ab20` | CountryCodesResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8a200` | CountryCodesResponse::Ctor | confirmed | Constructor for the CountryCodes response block: IString-shaped members at this+0x00 and this+0x10 set to the empty sentinel 0x01cb4080 with length and capacit… |
| `00f8bf20` | CountryCodesResponse::Parse | confirmed | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'CountryCodesResponse::Parse, failed with invalid xml'. Re… |

**CreditCardTypesRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a0d0` | CreditCardTypesRequest::Validate | confirmed | Checks this+0x28 (url), this+0x38 (userAgent) and this+0x18 (sessionId) for non-null in that order; on the first null logs a level-2 error through 00f99fd0 and… |

**CreditCardTypesResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a1e0` | CreditCardTypesResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f89280` | CreditCardTypesResponse::Ctor | confirmed | Constructor for the CreditCardTypes response block: IString-shaped members at this+0x00 and this+0x10 set to the empty sentinel 0x01cb4080 with length and capa… |
| `00f8b370` | — | unknown | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'CreditCardTypesResponse::Parse, failed with invalid xml'.… |

**DeletePaymentSourceRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7af40` | DeletePaymentSourceRequest::Validate | confirmed | Checks this+0x58 (url), this+0x68 (userAgent), this+0x38 (sessionId) and this+0x28 (paymentSourceId) in that order; each failure logs a level-2 message through… |

**DeletePaymentSourceResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b080` | DeletePaymentSourceResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8a790` | DeletePaymentSourceResponse::Parse | confirmed | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'DeletePaymentSourceResponse::Parse, failed with invalid x… |

**FinalizeThirdPartyOrderTransactionRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b670` | FinalizeThirdPartyOrderTransactionRequest::Validate | confirmed | Checks this+0x2c (url), this+0x18 (order id), this+0x3c (userAgent) and this+0x08 (sessionId) in that order; each failure logs a level-2 message through 00f99f… |
| `00f7b880` | FinalizeThirdPartyOrderTransactionRequest::Validate_B | likely | Second Validate body for the same class and message texts: checks this+0x18 (url) and this+0x38 (userAgent); each failure logs a level-2 message through 00f99f… |

**FinalizeThirdPartyOrderTransactionResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b7b0` | FinalizeThirdPartyOrderTransactionResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8b020` | FinalizeThirdPartyOrderTransactionResponse::Parse | confirmed | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'FinalizeThirdPartyOrderTransactionResponse::Parse, failed… |

**FundWalletRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f79a10` | FundWalletRequest::Validate | confirmed | Checks FundWalletRequest fields in order: url (this+0x98), userAgent (+0xa8), sessionId (+0x78), paymentSourceId (+0x38), a third-party rule (fails when +0x58 … |

**FundWalletResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f79c30` | FundWalletResponse::Validate | confirmed | Checks this+0x28 (version) and this+0x18 (status) for non-null. On the first null it logs a level-2 error through 00f99fd0 with a stack ObjectReportHandlerAdap… |
| `00f889d0` | FundWalletResponse::Parse | confirmed | XML reply parser (second argument is the document, third the report handler). Loads the document with 00d549d0 and gets the root with 00d50ac0; with no root it… |

**GameCommerce::UramCreditCardTypesResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d720d0` | GameCommerce::UramCreditCardTypesResponse::~UramCreditCardT… | confirmed | Destructor body (28 bytes, __fastcall, no delete). Stores the UramCreditCardTypesResponse vtable at *this, calls 00b3c390 (CreditCardAttributesInline8ArrayDtor… |
| `00d72710` | GameCommerce::UramCreditCardTypesResponse::DeletingDestruct… | confirmed | Scalar deleting destructor, vtable slot [0] of UramCreditCardTypesResponse::vftable (49 bytes, ret 4). Installs the vtable, calls 00b3c390 (CreditCardAttribute… |
| `00d726a0` | — | unknown | Constructor (97 bytes, __fastcall, returns this). Installs the UramResponseBase vtable, sets the second dword to the sentinel DAT_01bf3114 and the third to 0, … |

**GameCommerce::UramManager** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d72a70` | GameCommerce::UramManager::HandleCardTypesResponse | confirmed | Virtual method in UramManager vftable slot 4 (4384 pseudocode, 720 bytes). Walks the request chain in bucket (this+0x4c + (id&7)*4) comparing node+0x34 to the … |
| `00d730a0` | GameCommerce::UramManager::HandleRedeemCodeResponse | confirmed | Virtual method in UramManager vftable slot 7 (1627 bytes). Scans the bucket chain (this+0x4c + (id&7)*4, compare node+0x34). Miss: logs '(%s) Failed to find re… |
| `00d70460` | GameCommerce::UramManager::RequestPaymentSources | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping the sentinel DAT_01bf3114), creates the request through 00d69450 (re… |
| `00d70540` | GameCommerce::UramManager::RequestAddCard | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping the sentinel DAT_01bf3114) into a local id, allocates a request thro… |
| `00d70930` | GameCommerce::UramManager::RequestUpdateCard | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d70ce0` | GameCommerce::UramManager::RequestDeleteCard | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d70e60` | GameCommerce::UramManager::RequestCardTypes | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d70f80` | GameCommerce::UramManager::RequestPreviewWalletFund | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d71120` | GameCommerce::UramManager::RequestFundWallet | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d71550` | GameCommerce::UramManager::RequestPurchaseMembership | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d71900` | GameCommerce::UramManager::RequestSelectCodeRedemptionOption | likely | If the manager flag at this+0x1c is set and param_5 (the option value) is positive: allocates a tracking id through 00d681f0 (IncrementCounterSkippingValue on … |
| `00d71a10` | GameCommerce::UramManager::RequestFinalizeThirdPartyOrderTr… | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d71ae0` | GameCommerce::UramManager::RequestIsAffiliatedWithPartner | likely | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d6f2a0` | — | unknown | Virtual method in UramManager vftable slot 9. Walks the pending-request chain in bucket (this+0x4c + (id&7)*4) comparing node+0x34 to the tracking id. Miss: lo… |
| `00d6f480` | — | unknown | Virtual method in UramManager vftable slot 10. Same request-chain lookup as the UpdateCard handler (bucket by id&7, compare node+0x34). Miss: logs '(DeleteCard… |
| `00d6f660` | — | unknown | Virtual method in UramManager vftable slot 6. Same request-chain lookup (bucket id&7, compare node+0x34). Miss: logs '(MembershipPurchase) Failed to find reque… |
| `00d6f880` | — | unknown | Virtual method in UramManager vftable slot 13. Walks the request chain (bucket id&7, compare node+0x34). Miss: logs '(IsAffiliatedWithPartner) Failed to find r… |
| `00d6fa90` | — | unknown | Virtual method in UramManager vftable slot 14. Walks the request chain (bucket id&7, compare node+0x34). Miss: logs '(FinalizeThirdPartyOrderTransaction) Faile… |
| `00d71340` | — | unknown | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d71740` | — | unknown | If the manager flag at this+0x1c is set, advances the tracking counter this+0x24 (skipping sentinel DAT_01bf3114) into a local id, creates a request through 00… |
| `00d72d40` | — | unknown | Virtual method in UramManager vftable slot 5 (854 bytes). Walks the request chain (bucket id&7, compare node+0x34). Miss logs '(MembershipPreview) Failed to fi… |

**GameCommerce::UramManager::UramAddCreditCardRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6fd40` | GameCommerce::UramManager::UramAddCreditCardRequest::Comple… | likely | Virtual slot 1 of UramAddCreditCardRequest::vftable. Builds an AddNewCard response with constructor 00d6e290 on the stack, stores this+0xc and the argument int… |

**GameCommerce::UramManager::UramDeleteCreditCardRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6fef0` | GameCommerce::UramManager::UramDeleteCreditCardRequest::Com… | likely | Virtual slot 1 of UramDeleteCreditCardRequest::vftable. Builds a DeleteCard response inline on the stack (same pattern as 00d6fdf0 with the UramDeleteCardRespo… |

**GameCommerce::UramManager::UramFinalizeThirdPartyOrderTransactionRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d70200` | GameCommerce::UramManager::UramFinalizeThirdPartyOrderTrans… | likely | Virtual slot 1 of UramFinalizeThirdPartyOrderTransactionRequest::vftable. Builds a FinalizeThirdPartyOrderTransaction response inline on the stack (UramRespons… |

**GameCommerce::UramManager::UramFundWalletRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d700a0` | GameCommerce::UramManager::UramFundWalletRequest::CompleteL… | likely | Virtual slot 1 of UramFundWalletRequest::vftable. Builds a wallet-fund response with constructor 00d6e090 on the stack, stores this+0xc and the argument into i… |

**GameCommerce::UramManager::UramGetCreditCardsRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d73920` | GameCommerce::UramManager::UramGetCreditCardsRequest::Compl… | likely | Virtual slot [1] of UramGetCreditCardsRequest::vftable (190 bytes). Constructs a CreditCardTypes response on the stack through 00d726a0, stores this+0xc and th… |

**GameCommerce::UramManager::UramIsAffiliatedWithPartnerRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d702f0` | GameCommerce::UramManager::UramIsAffiliatedWithPartnerReque… | likely | Virtual slot 1 of UramIsAffiliatedWithPartnerRequest::vftable. Builds an IsAffiliatedWithPartner response with constructor 00d6e410 on the stack, stores this+0… |

**GameCommerce::UramManager::UramPaymentSourceRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6fc90` | GameCommerce::UramManager::UramPaymentSourceRequest::Comple… | likely | Virtual slot 1 of UramPaymentSourceRequest::vftable. Builds a PaymentSources response with constructor 00d6e1d0 on the stack, stores this+0xc and the argument … |

**GameCommerce::UramManager::UramPreviewMembershipPurchaseRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d73710` | GameCommerce::UramManager::UramPreviewMembershipPurchaseReq… | likely | Virtual slot [1] of UramPreviewMembershipPurchaseRequest::vftable (170 bytes). Stores this+0xc and the argument into a stack MembershipPurchasePreview response… |

**GameCommerce::UramManager::UramPreviewWalletFundRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6fff0` | GameCommerce::UramManager::UramPreviewWalletFundRequest::Co… | likely | Virtual slot 1 of UramPreviewWalletFundRequest::vftable. Builds a wallet-preview response with constructor 00d6df70 on the stack, stores this+0xc and the argum… |

**GameCommerce::UramManager::UramPurchaseMembershipRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d70150` | GameCommerce::UramManager::UramPurchaseMembershipRequest::C… | likely | Virtual slot 1 of UramPurchaseMembershipRequest::vftable. Builds a MembershipPurchase response with constructor 00d6e480 on the stack, stores this+0xc and the … |

**GameCommerce::UramManager::UramRedeemCodeRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d737c0` | GameCommerce::UramManager::UramRedeemCodeRequest::CompleteL… | likely | Virtual slot [1] of UramRedeemCodeRequest::vftable (170 bytes). Stores this+0xc and the argument into a stack RedeemCode response (constructed by 00d72860), ca… |

**GameCommerce::UramManager::UramSelectCodeRedemptionOptionRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d73870` | GameCommerce::UramManager::UramSelectCodeRedemptionOptionRe… | likely | Virtual slot [1] of UramSelectCodeRedemptionOptionRequest::vftable (170 bytes). Stores this+0xc and the argument into a stack RedeemCode response (constructed … |

**GameCommerce::UramManager::UramUpdateCreditCardRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6fdf0` | GameCommerce::UramManager::UramUpdateCreditCardRequest::Com… | likely | Virtual slot 1 of UramUpdateCreditCardRequest::vftable. Builds an UpdateCard response inline on the stack (UramResponseBase vtable, a fixed-256 string construc… |

**GameCommerce::UramMembershipPurchasePreviewResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d720f0` | GameCommerce::UramMembershipPurchasePreviewResponse::~UramM… | confirmed | Destructor body (168 bytes, __fastcall, no delete). Stores the MembershipPurchasePreview vtable at *this, calls 00d71c90 (the one-inline-slot NextOrderCharge a… |
| `00d72840` | GameCommerce::UramMembershipPurchasePreviewResponse::Deleti… | confirmed | Scalar deleting destructor, vtable slot [0] of UramMembershipPurchasePreviewResponse::vftable (30 bytes, ret 4). Calls 00d720f0 (the response destructor body),… |
| `00d72750` | — | unknown | Constructor (225 bytes, __fastcall, returns this). Installs the UramResponseBase vtable, sets the second dword to the sentinel DAT_01bf3114 and clears the thir… |

**GameCommerce::UramRedeemCodeResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d721a0` | GameCommerce::UramRedeemCodeResponse::~UramRedeemCodeRespon… | confirmed | Destructor body (310 bytes, __fastcall, no delete). Stores the UramRedeemCodeResponse vtable at *this; releases the two SoeUtil::Array<Uram::RedeemCodeResponse… |
| `00d72860` | GameCommerce::UramRedeemCodeResponse::UramRedeemCodeResponse | confirmed | Constructor (303 bytes, __fastcall, returns this). Installs the UramResponseBase vtable with the sentinel dword and fixed-256 string as in the other response c… |
| `00d72990` | GameCommerce::UramRedeemCodeResponse::DeletingDestructor | confirmed | Scalar deleting destructor, vtable slot [0] of UramRedeemCodeResponse::vftable (30 bytes, ret 4). Calls 00d721a0 (the response destructor body), then frees thi… |

**IsAffiliatedWithPartnerRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b400` | IsAffiliatedWithPartnerRequest::Validate | confirmed | Checks this+0x48 (url), this+0x18 (gameCode), this+0x68 (userAgent), this+0x08 (sessionId) in that order; a failure on url, sessionId or the partner check logs… |

**IsAffiliatedWithPartnerResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7b5a0` | IsAffiliatedWithPartnerResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8abe0` | IsAffiliatedWithPartnerResponse::Parse | confirmed | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'IsAffiliatedWithPartnerResponse::Parse, failed with inval… |

**PreviewOrderRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a2b0` | PreviewOrderRequest::Validate | confirmed | Checks this+0xa8 (url), this+0xb8 (userAgent), this+0x58 (sessionId), this+0x38 (paymentSourceId), this+0x68 (sku) and this+0x08 (gameCode) in that order; each… |

**PreviewOrderResponse** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a450` | PreviewOrderResponse::Validate | confirmed | Checks this+0x38 (version) and this+0x28 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8b780` | PreviewOrderResponse::Ctor | confirmed | Constructor for the PreviewOrderResponse fields: nine IString-shaped members at this+0x00 to +0x80 set to empty, SoeUtil::Array<Uram::PreviewOrderResponse::Sub… |
| `00f8c2f0` | PreviewOrderResponse::Parse | confirmed | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'PreviewOrderResponse::Parse, failed with invalid xml'. Re… |
| `00f894a0` | — | unknown | Destructor body (does not free this) for the PreviewOrderResponse layout: releases nine IString-shaped members at this+0x00 to +0x80 (step 0x10), the subscript… |

**PurchaseOrderRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a550` | PurchaseOrderRequest::Validate | confirmed | Checks this+0x88 (url), this+0x98 (userAgent), this+0x68 (sessionId), this+0x48 (paymentSourceId), this+0x08 (currency), this+0x78 (sku) and this+0x18 (gameCod… |

**PurchaseOrderResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a710` | PurchaseOrderResponse::Validate | confirmed | Checks this+0x28 (version) and this+0x18 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f89a60` | PurchaseOrderResponse::Ctor | confirmed | Constructor for the PurchaseOrderResponse field block: eight IString-shaped members at this+0x00 to +0x78 set to the empty sentinel 0x01cb4080 with length and … |
| `00f8b990` | — | unknown | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'PurchaseOrderResponse::Parse, failed with invalid xml'. R… |

**RedeemCodeRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a7e0` | RedeemCodeRequest::Validate | confirmed | Checks this+0x58 (url), this+0x68 (userAgent), this+0x48 (sessionId), this+0x08 (activationCode) and this+0x28 (gameCode) in that order; each failure logs a le… |

**RedeemCodeResponse** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7a940` | RedeemCodeResponse::Validate | confirmed | Checks this+0x38 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |

**Smart::ReportHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f79070` | Smart::ReportHandler::ResetBaseVtable | confirmed | Stores the address of Smart::ReportHandler::vftable into the first dword of this and returns; a single mov and ret with no other effect. |
| `00f79100` | Smart::ReportHandler::ScalarDeletingDtor | confirmed | Stores Smart::ReportHandler::vftable into this, frees this through _free (00d82463) when bit 0 of the flag argument is set, and returns this (ret 4). |

**UpdateCreditCardRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7ac20` | UpdateCreditCardRequest::Validate | confirmed | Checks fields in order: url (this+0x118), userAgent (+0x128), sessionId (+0xb8), paymentSourceId (+0xa8), name (+0x88), address (+0x08), city (+0x28), state (+… |

**UpdateCreditCardResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f7ae70` | UpdateCreditCardResponse::Validate | confirmed | Checks this+0x18 (version) and this+0x08 (status) for non-null; logs a level-2 error through 00f99fd0 on the first null and returns 0, otherwise returns 1. |
| `00f8a3d0` | UpdateCreditCardResponse::Parse | confirmed | XML reply parser (same template as FundWalletResponse::Parse): loads the document (00d549d0) and root (00d50ac0); no root reports 'UpdateCreditCardResponse::Pa… |

**UploaderImpl** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f62760` | UploaderImplCtor | confirmed | Constructor body: stores Smart::ReportHandler::vftable at +0x04 and then Uram::UploaderLogHandlerAdapter::vftable at +0x00 and +0x04; installs SoeUtil::Deque<U… |
| `00f740f0` | UploaderImpl::AddRequest | confirmed | Thiscall. If the byte at this+0x70 is clear it logs 'UploaderImpl::AddRequest, failed with uploader not configured' via 00f99fd0 (level 2) and returns 0. Other… |
| `00f627d0` | UploaderImplDtorBody | likely | Destructor body (does not free this): runs the function-object deque pointers (count at +0x40, start at +0x44, map at +0x20) through slot 0 with flag 1 for eac… |
| `00f62950` | UploaderImplScalarDeletingDtor | likely | Calls the destructor body 00f627d0 on this, then frees this through _free when the flag byte at param+8 has bit 0 set; returns this in EAX; the body is stdcall… |
| `00f62920` | — | unknown | Calls operator new (00d833a6) with 0x74 bytes; on success passes the block in ECX to the UploaderImpl constructor 00f62760 and stores the returned pointer into… |

**Uram::AccountPaymentInfoResponse** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8d490` | AccountPaymentInfoResponse::Parse | confirmed | Parses the AccountPaymentInfo XML reply: arg2 is the document buffer (loaded through 00d549d0, TiXmlDocument vftable 017ed9dc stored on exit, freed through 00d… |
| `00f8ca60` | AccountPaymentInfoResponse::DtorBody | likely | Destructor body (does not free this): stores the Error-array vftable at +0x54 and destroys that array through 00f63010 and 00401c00; stores the Card-array vfta… |
| `00f8cbc0` | AccountPaymentInfoResponse::Reset | likely | Releases the string members at +0x00, +0x10, +0x24 and +0x34 (decrement refcount, free when it reaches zero) and resets each to the static empty string with le… |
| `00f8d060` | AccountPaymentInfoResponse::Ctor | likely | Constructor body: installs IString vftables at +0x00, +0x10, +0x24 and +0x34 with the static empty-string buffer and zero lengths, installs the Card-array vfta… |
| `00f8d110` | AccountPaymentInfoResponse::ParseCardChildren | likely | Walks the child list of arg1 (head at +0x18, next sibling at +0x28). For each child it calls the child's vtable slot 5 (+0x14) and matches the returned element… |

**Uram::AccountPaymentInfoResponse::Card** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f75100` | Card::CopyCtor | likely | Thiscall copy-constructor of a 0x48-byte element. Builds three IString members at +0x00, +0x10 and +0x20 (vftable, empty sentinel, zero length and capacity) an… |

**Uram::FunctionObjectT** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f78f50` | Uram::FunctionObjectT::GiveTime | confirmed | Switches on the state word at this+0x4b4: states 0-4 call 00f70810, 00f685e0, 00f68740, 00f68ce0 or 00f78b50 respectively; state 5 calls 00f68450 and returns; … |
| `00f667f0` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 (trackingId), the error string for the impl error code (00f91e10 then 00f91dd0) … |
| `00f67700` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 and the error strings (00f91e10, 00f91dd0, 00f8de10), then sets state this+0x4ac… |
| `00f68250` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 and the error strings (00f91e10, 00f91dd0, 00f8de10), then sets state this+0x4b4… |
| `00f695f0` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 and the error strings (00f91e10, 00f91dd0, 00f8de10), then sets state this+0x3d4… |
| `00f6a100` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 and the error strings (00f91e10, 00f91dd0, 00f8de10), then sets state this+0x4cc… |
| `00f6aa80` | Uram::FunctionObjectT::FireEventWithCode | likely | When the code argument is nonzero: logs FunctionObjectT::FireEvent with this+4 and the error strings (00f91e10, 00f91dd0, 00f8de10), then sets state this+0x3f4… |
| `00f62cf0` | — | unknown | When param_2 (error code) is non-zero: logs level 2 with the tracking id at this+0x424 and the error strings through 00f99fd0 using the FunctionObjectT::FireEv… |
| `00f62f00` | — | unknown | Same error completion as 00f62cf0 but with the constant error code 0xffffffef (-17) passed to the callback: logs the FunctionObjectT::FireEvent message through… |

**Uram::FunctionObjectT<Uram::AccountPaymentInfoRequest,Uram::AccountPaymentInfoResponse,Uram::GetAccountPaymentInfoCallback>** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f62970` | FunctionObjectT_AccountPaymentInfo_Ctor | confirmed | Constructor body: stores the FunctionObjectT<AccountPaymentInfoRequest,...> vtable at +0x00 and param_2 at +0x04; calls 00f60ef0 on param_3 (a seven-string rec… |
| `00f6eaa0` | FunctionObjectT::Init | confirmed | Start step of the request: sets dword this+0xf0 to 2, parses the URL at this+0x4c (failure logs 'failed to parse url', fires -3 through the callback vtable slo… |
| `00f787e0` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the AccountPaymentInfo GiveTime. Requires the transport word at this+0x298 to be 2. Size at this+0x3e8 above 0x100000 fires … |
| `00f78e70` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x42c: cases 0 to 3 call 00f6eaa0, 00f630f0, 00f63240, 00f636d0; case 4 calls the CheckD… |
| `00f630f0` | FunctionObjectT_AccountPayment_ConnectStep1 | likely | Connect poll for the AccountPaymentInfo instance: calls 00f91e00 (connection status). Status 0: calls 00f92720 (SslConnectionFilter GiveTime); when it returns … |
| `00f63240` | FunctionObjectT_AccountPayment_ConnectStep2 | likely | Second connect-phase poll for the AccountPaymentInfo instance: calls 00f91e00. Status 1: calls 00f92720 and, when false, logs the GiveTime failure with the url… |
| `00f635f0` | FunctionObjectT_AccountPayment_DtorBody | likely | Destructor body (does not free this): installs the AccountPaymentInfo FunctionObjectT vtable, calls 00f99910, 00f91de0, 00f99670, 00f603d0, 00f98190, 00f8ca60 … |
| `00f636b0` | FunctionObjectT_AccountPayment_ScalarDeletingDtor | likely | Scalar deleting destructor: calls the body 00f635f0 on this, frees this through _free when the flag byte has bit 0 set, returns this (ret 4). |
| `00f636d0` | FunctionObjectT_AccountPayment_SendStep | likely | Send step for the AccountPaymentInfo instance: builds a temporary SoeUtil::IString<char> (00f810a0, 00f96090) from the prepared request, calls 00f99920 (HttpCo… |
| `00f62ce0` | FunctionObjectT_AccountPayment_SetState5 | speculative | Virtual slot [3] of the AccountPaymentInfo FunctionObjectT: stores 5 into the state dword at this+0x42c and returns. |

**Uram::FunctionObjectT<Uram::AddPaymentSourceRequest,Uram::AddPaymentSourceResponse,Uram::AddPaymentSourceCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f75a20` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the AddPaymentSource GiveTime. Requires the transport word at this+0x348 to be 2. Size at this+0x498 above 0x100000 fires -1… |
| `00f77830` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x4dc: cases 0 to 3 call 00f6f730, 00f65440, 00f655a0, 00f657e0; case 4 calls the CheckD… |
| `00f65440` | FunctionObjectT_AddPaymentSource_ConnectStep1 | likely | Connect poll for the AddPaymentSource instance (same shape as 00f630f0): status 0 with a false 00f92720 result gives -4 (url at this+0x314, state +0x4dc = 6, c… |
| `00f655a0` | FunctionObjectT_AddPaymentSource_ConnectStep2 | likely | Second connect-phase poll for the AddPaymentSource instance (same shape as 00f63240): GiveTime failure on status 1 gives -5 (url at this+0x314, state 6, callba… |
| `00f65700` | FunctionObjectT_AddPaymentSource_DtorBody | likely | Destructor body (does not free this): installs the AddPaymentSource vtable, calls 00f99910, 00f91de0, 00f99670, 00f603d0, 00f98190, 00f86c00 and 00f7d720 on it… |
| `00f657c0` | FunctionObjectT_AddPaymentSource_ScalarDeletingDtor | likely | Scalar deleting destructor: calls 00f65700 on this, frees this through _free when the flag byte has bit 0 set, returns this. |
| `00f657e0` | FunctionObjectT_AddPaymentSource_SendStep | likely | Send step for the AddPaymentSource instance (same shape as 00f636d0): calls 00f81240 first (request body build, not decoded), then sends through 00f99920; fail… |
| `00f65190` | FunctionObjectT_AddPaymentSource_Ctor | speculative | Constructor body: stores the AddPaymentSource FunctionObjectT vtable at +0x00 and param_2 at +0x04; copies the twenty-string request from param_3 through 00f61… |
| `00f65320` | FunctionObjectStateIs6Or7_AddPaymentSource | speculative | Returns 1 when the state dword at this+0x4dc is 6 or 7, else 0 (__fastcall, this in ECX). |
| `00f65340` | FunctionObjectT_AddPaymentSource_SetState5 | speculative | Stores 5 into the state dword at this+0x4dc and returns. |
| `00f65350` | — | unknown | FireEvent for the AddPaymentSource instance: when the error argument is non-zero, logs 'FunctionObjectT::FireEvent, trackingId=%u' with the tracking id at +0x4… |
| `00f653d0` | — | unknown | Logs the FireEvent line with the tracking id at +0x4d4 for error 0xffffffef (-17), sets state +0x4dc to 6, and calls slot 3 of the callback at +0x4cc with (thi… |

**Uram::FunctionObjectT<Uram::CasGetSessionTicketRequest,Uram::CasGetSessionTicketResponse,Uram::CasGetSessionTicketCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6beb0` | FunctionObjectT::Ctor | confirmed | Constructor with ten stack arguments (ret 0x28): stores argument 2 at this+4, installs the CasGetSessionTicketRequest vftable, calls 00f622b0 with argument 3, … |
| `00f6c030` | FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the dword at this+0x3b4 is 6 or 7, otherwise 0. Read-only. |
| `00f6c060` | FunctionObjectT::FireEvent | confirmed | If argument 2 (error code) is non-zero: formats the error text with 00f8de10, logs the FireEvent line at level 2 on channel this+0x3ac, and sets state this+0x3… |
| `00f6c0d0` | FunctionObjectT::FireEventErr17 | confirmed | Unconditionally formats error -17 (0xffffffef) through 00f8de10, logs the FireEvent line at level 2 on channel this+0x3ac, sets state this+0x3b4 to 6 and calls… |
| `00f6c140` | FunctionObjectT::ConnectStep0 | confirmed | Same routine as ConnectStep0 of the CasKeepAlive request with class offsets: calls 00f91e00; result 0 runs SslConnectionFilter GiveTime 00f92720 and fails with… |
| `00f6c290` | FunctionObjectT::ConnectStep1 | confirmed | Same routine as ConnectStep1 of the CasKeepAlive request with class offsets: result 1 runs GiveTime 00f92720 and fails with -5 on a 0 return; result 2 sets sta… |
| `00f6c3e0` | FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Sets the CasGetSessionTicket vftable, calls member teardown helpers 00f99910 (no-op stub), 00f91de0, 00f99670, 00f603d0, … |
| `00f6c490` | FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6c3e0, then frees this when flag bit 0 of the second argument is set, and returns this. |
| `00f6c4b0` | FunctionObjectT::Send | confirmed | Send step with class-B offsets: builds the request body through 00f81ba0 (length at this+0x88/+0x8c), builds the header block through 00f96090, sends header th… |
| `00f6c800` | FunctionObjectT::CheckDone | confirmed | Poll step for the CasGetSessionTicket response with class-B offsets (response state this+0x220, size this+0x370, complete flag this+0x37c, status this+0x228, s… |
| `00f742b0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump, same shape as 00f741d0. State dword at this+0x3b4 selects 00f71cc0, 00f6c140, 00f6c290, 00f6c4b0, 00f6c800 (cases 0 to 4) … |
| `00f6c050` | FunctionObjectT::SetState5 | speculative | Stores the constant 5 into the dword at this+0x3b4 and returns. |

**Uram::FunctionObjectT<Uram::CasKeepAliveRequest,Uram::CasKeepAliveResponse,Uram::CasKeepAliveCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6b200` | FunctionObjectT::Ctor | confirmed | Constructor with ten stack arguments (ret 0x28): stores argument 2 at this+4 (trackingId), installs the CasKeepAliveRequest vftable at this+0, stores arguments… |
| `00f6b380` | FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the dword at this+0x394 is 6 or 7, otherwise 0 (bytes: mov eax,[ecx+0x394]; cmp 6 / je; cmp 7 / je; xor eax,eax; ret). Read-only. |
| `00f6b3b0` | FunctionObjectT::FireEvent | confirmed | If argument 2 (error code) is non-zero: formats the error text with 00f8de10(code, 00f91dd0(00f91e10())), logs the FireEvent line at level 2 on channel this+0x… |
| `00f6b420` | FunctionObjectT::FireEventErr17 | confirmed | Unconditionally formats error -17 (0xffffffef) through 00f8de10, logs the FireEvent line at level 2 on channel this+0x38c, sets state this+0x394 to 6 and calls… |
| `00f6b490` | FunctionObjectT::ConnectStep0 | confirmed | Calls 00f91e00 and branches on its result. Result 0: runs SslConnectionFilter GiveTime 00f92720; if that returns 0 it logs the GiveTime failure text, fires -4 … |
| `00f6b5e0` | FunctionObjectT::ConnectStep1 | confirmed | Calls 00f91e00 and branches on its result. Result 1: runs SslConnectionFilter GiveTime 00f92720; on failure logs the GiveTime text, fires -5 (0xfffffffb) and s… |
| `00f6b730` | FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Sets the CasKeepAlive vftable, calls member teardown helpers 00f99910 (a no-op stub), 00f91de0, 00f99670, 00f603d0, 00f98… |
| `00f6b7e0` | FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6b730, then calls _free on this when flag bit 0 of the second argument is set, and returns this. |
| `00f6b800` | FunctionObjectT::Send | confirmed | Send step: builds the request body into a temporary IString through 00f81b60 (length stored at this+0x68/+0x6c), builds the header block through 00f96090, send… |
| `00f6bb40` | FunctionObjectT::CheckDone | confirmed | Poll step for the CasKeepAlive response, active when response state this+0x200 is 2. Rejects a body whose size at this+0x350 is at least 0x100001 (logs the siz… |
| `00f741d0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switches on the state dword at this+0x394: cases 0 to 4 call 00f718c0, 00f6b490, 00f6b5e0, 00f6b800, 00f6bb40; case 5 call… |
| `00f6b3a0` | FunctionObjectT::SetState5 | speculative | Stores the constant 5 into the dword at this+0x394 and returns (bytes: mov dword [ecx+0x394],5; ret; 11 bytes). |

**Uram::FunctionObjectT<Uram::CountryCodesRequest,Uram::CountryCodesResponse,Uram::GetCountryCodesCallback>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f69430` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the CountryCodes FunctionObjectT vtable, calls 00f61cd0 and 00f8a200 (CountryCodesResponse construct… |
| `00f695c0` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x3d4 is 6 or 7, otherwise 0. No calls. |
| `00f695e0` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x3d4. No calls, returns nothing. |
| `00f696b0` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17), sets state this+0x3d4 = 6, and calls the callback at this+0x3c4 vtabl… |
| `00f69760` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | CountryCodes layout (url +0x20c, log +0x3cc, state +0x3d4, callback +0x3c4 with data +0x48). Branches on the status from 00f91e00: status 0 with 00f92720 retur… |
| `00f698b0` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | CountryCodes layout. Status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state 6 and calls callback slot +0x20 with (tr… |
| `00f69a10` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the CountryCodes FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f98190, … |
| `00f69ad0` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f69a10, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f69af0` | Uram::FunctionObjectT::SendRequest | confirmed | CountryCodes layout (url +0x20c, log +0x3cc, state +0x3d4, callback +0x3c4 with data +0x48). Builds the request body as an IString through 00f81820, stores a 6… |
| `00f763e0` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the CountryCodes GiveTime. Requires the transport word at this+0x240 to be 2. Size at this+0x390 above 0x100000 fires -16. C… |
| `00f77ad0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x3d4: cases 0 to 3 call 00f70c40, 00f69760, 00f698b0, 00f69af0; case 4 calls the CheckD… |

**Uram::FunctionObjectT<Uram::CreditCardTypesRequest,Uram::CreditCardTypesResponse,Uram::GetCreditCardTypesCallback>** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f65e40` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | Branches on the status from 00f91e00 (which reads the impl pointer and maps its state). Status 0: when 00f92720 returns 0 it logs the SSL GiveTime failure with… |
| `00f65f90` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | Branches on the status from 00f91e00. Status 1: when 00f92720 returns 0 it logs the SSL GiveTime failure with code -5 (0xfffffffb), sets this+0x3e4 = 6 and cal… |
| `00f660f0` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the CreditCardTypes FunctionObjectT vtable at this+0, then tears members down in order with calls to 00f99910 (emp… |
| `00f661b0` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f660f0, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f661d0` | Uram::FunctionObjectT::SendRequest | confirmed | Builds the request body as an IString through 00f81470, stores a 64-bit value (local_28 from the builder call) at this+0xb8/+0xbc, obtains a buffer from 00f960… |
| `00f75d50` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the CreditCardTypes GiveTime. Requires the transport word at this+0x250 to be 2. Size at this+0x3a0 above 0x100000 fires -16… |
| `00f77910` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x3e4: cases 0 to 3 call 00f6fb70, 00f65e40, 00f65f90, 00f661d0; case 4 calls the CheckD… |
| `00f65b10` | FunctionObjectT_CreditCardTypes_Ctor | speculative | Constructor body: stores the CreditCardTypes FunctionObjectT vtable at +0x00 and param_2 at +0x04; copies the five-string request from param_3 through 00f616c0… |
| `00f65ca0` | FunctionObjectStateIs6Or7_CreditCardTypes | speculative | Returns 1 when the state dword at this+0x3e4 is 6 or 7, else 0 (__fastcall, this in ECX). |
| `00f65cc0` | FunctionObjectT_CreditCardTypes_SetState5 | speculative | Stores 5 into the state dword at this+0x3e4 and returns. |
| `00f65d70` | FunctionObjectT_CreditCardTypes_ScalarDeletingDtor | speculative | Scalar deleting destructor: calls the body 00f7e160 on this, frees this through _free when the flag byte has bit 0 set, returns this. |
| `00f65cd0` | — | unknown | FireEvent for the CreditCardTypes instance: when the error argument is non-zero, logs the FireEvent line with the tracking id at +0x3dc and sets state +0x3e4 t… |
| `00f65d90` | — | unknown | Logs the FireEvent line with the tracking id at +0x3dc for error 0xffffffef (-17), sets state +0x3e4 to 6, and calls slot 4 of the callback at +0x3d4 with (thi… |

**Uram::FunctionObjectT<Uram::DeletePaymentSourceRequest,Uram::DeletePaymentSourceResponse,Uram::DeletePaymentSourceCallback>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6a8c0` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the DeletePaymentSource FunctionObjectT vtable, calls 00f620d0 and 00f87ce0 for the embedded record,… |
| `00f6aa50` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x3f4 is 6 or 7, otherwise 0. No calls. |
| `00f6aa70` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x3f4. No calls, returns nothing. |
| `00f6aaf0` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17), sets state this+0x3f4 = 6, and calls the callback at this+0x3e4 vtabl… |
| `00f6ab60` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | DeletePaymentSource layout (url +0x22c, log +0x3ec, state +0x3f4, callback +0x3e4 with data +0x78). Branches on the status from 00f91e00: status 0 with 00f9272… |
| `00f6acb0` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | DeletePaymentSource layout. Status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state 6 and calls callback slot +0x28 w… |
| `00f6ae00` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the DeletePaymentSource FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f… |
| `00f6aec0` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6ae00, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f6aee0` | FunctionObjectT::Send | confirmed | Send step: builds the request body into a temporary IString through 00f81ab0 (length stored at this+0xc8/+0xcc), builds the header block through 00f96090, send… |
| `00f76a80` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the DeletePaymentSource GiveTime. Requires the transport word at this+0x260 to be 2. Size at this+0x3b0 above 0x100000 fires… |
| `00f77c90` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x3f4: cases 0 to 3 call 00f714a0, 00f6ab60, 00f6acb0, 00f6aee0; case 4 calls the CheckD… |

**Uram::FunctionObjectT<Uram::FinalizeThirdPartyOrderTransactionRequest,Uram::FinalizeThirdPartyOrderTransactionResponse,Uram::FinalizeThirdPartyOrderTransactionCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6d4b0` | FunctionObjectT::Ctor | confirmed | Constructor with ten stack arguments (ret 0x28): stores argument 2 at this+4, installs the FinalizeThirdPartyOrderTransactionRequest vftable, calls 00f62490 wi… |
| `00f6d640` | FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the dword at this+0x3dc is 6 or 7, otherwise 0. Read-only. |
| `00f6d6e0` | FunctionObjectT::FireEventErr17 | confirmed | Unconditionally formats error -17 (0xffffffef) through 00f8de10, logs the FireEvent line at level 2 on channel this+0x3d4, sets state this+0x3dc to 6 and calls… |
| `00f6d750` | FunctionObjectT::ConnectStep0 | confirmed | Connect-phase step with class-D offsets: calls 00f91e00; result 0 runs GiveTime 00f92720 and fails with -4 on a 0 return; result 1 sets state this+0x3dc to 2; … |
| `00f6d8a0` | FunctionObjectT::ConnectStep1 | confirmed | Connect-phase step with class-D offsets: result 1 runs GiveTime 00f92720 and fails with -5 on a 0 return; result 2 sets state this+0x3dc to 3; other results lo… |
| `00f6d9f0` | FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Sets the FinalizeThirdPartyOrderTransaction vftable, calls teardown helpers 00f99910 (no-op stub), 00f91de0, 00f99670, 00… |
| `00f6dab0` | FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6d9f0, then frees this when flag bit 0 of the second argument is set, and returns this. |
| `00f6dad0` | FunctionObjectT::Send | confirmed | Send step with class-D offsets: builds the request body through 00f81d30 (length at this+0xb0/+0xb4), builds the header block through 00f96090, sends header th… |
| `00f77160` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the FinalizeThirdPartyOrderTransaction GiveTime. Requires the transport word at this+0x248 to be 2. Size at this+0x398 above… |
| `00f77e50` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x3dc: cases 0 to 3 call 00f724f0, 00f6d750, 00f6d8a0, 00f6dad0; case 4 calls the CheckD… |
| `00f6d670` | FunctionObjectT::FireEvent | likely | If argument 2 (error code) is non-zero: formats the error text with 00f8de10, logs the FireEvent line at level 2 on channel this+0x3d4, and sets state this+0x3… |
| `00f6d660` | FunctionObjectT::SetState5 | speculative | Stores the constant 5 into the dword at this+0x3dc and returns. |

**Uram::FunctionObjectT<Uram::FundWalletPreviewRequest,Uram::FundWalletPreviewResponse,Uram::FundWalletPreviewCallback>** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6eec0` | FunctionObjectT::Init | confirmed | Start step of the request with class-H offsets: sets dword this+0x158 to 2, parses the URL at this+0x6c (failure logs, fires -3 through callback vtable slot +4… |
| `00f753e0` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the FundWalletPreview GiveTime. Requires the transport word at this+0x300 to be 2. Response size at this+0x450 above 0x10000… |
| `00f77670` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x494: cases 0 to 3 call 00f6eec0, 00f64090, 00f641f0, 00f64430; case 4 calls the CheckD… |
| `00f64090` | FunctionObjectT_FundWalletPreview_ConnectStep1 | likely | Connect poll for the FundWalletPreview instance (same shape as 00f630f0): status 0 from 00f91e00 then 00f92720 (GiveTime) false gives the -4 completion, with t… |
| `00f641f0` | FunctionObjectT_FundWalletPreview_ConnectStep2 | likely | Second connect-phase poll for the FundWalletPreview instance (same shape as 00f63240): status 1 with GiveTime failure gives the -5 completion (url at this+0x2c… |
| `00f64350` | FunctionObjectT_FundWalletPreview_DtorBody | likely | Destructor body (does not free this): installs the FundWalletPreview FunctionObjectT vtable, calls 00f99910, 00f91de0, 00f99670, 00f603d0, 00f98190, 00f86480 a… |
| `00f64410` | FunctionObjectT_FundWalletPreview_ScalarDeletingDtor | likely | Scalar deleting destructor: calls 00f64350 on this, frees this through _free when the flag byte has bit 0 set, returns this. |
| `00f64430` | FunctionObjectT_FundWalletPreview_SendStep | likely | Send step for the FundWalletPreview instance (same shape as 00f636d0): calls 00f81150 first (request body build, not decoded) and then sends through 00f99920; … |
| `00f63de0` | FunctionObjectT_FundWalletPreview_Ctor | speculative | Constructor body: stores the FundWalletPreview FunctionObjectT vtable at +0x00 and param_2 at +0x04; copies the nine-string request from param_3 through 00f610… |
| `00f63f70` | FunctionObjectStateIs6Or7_FundWalletPreview | speculative | Returns 1 when the state dword at this+0x494 is 6 or 7, else 0 (__fastcall, this in ECX). |
| `00f63f90` | FunctionObjectT_FundWalletPreview_SetState5 | speculative | Stores 5 into the state dword at this+0x494 and returns. |
| `00f63fa0` | — | unknown | FireEvent for the FundWalletPreview instance: if the error argument is non-zero, logs 'FunctionObjectT::FireEvent, trackingId=%u' with the tracking id at +0x48… |
| `00f64020` | — | unknown | Logs 'FunctionObjectT::FireEvent, trackingId=%u' with the tracking id at +0x48c and the error text for code 0xffffffef (-17), sets state +0x494 to 6, and calls… |

**Uram::FunctionObjectT<Uram::FundWalletRequest,Uram::FundWalletResponse,Uram::FundWalletCallback>** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6f2f0` | FunctionObjectT::Init | confirmed | Start step of the request with class-I offsets: sets dword this+0x1b8 to 2, parses the URL at this+0x9c (the log prints the string at this+0x98; failure fires … |
| `00f75700` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the FundWallet GiveTime. Requires the transport word at this+0x360 to be 2. Size at this+0x4b0 above 0x100000 logs 'response… |
| `00f77750` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x4f4: cases 0 to 3 call 00f6f2f0, 00f64a60, 00f64bc0, 00f64e00; case 4 calls the CheckD… |
| `00f64a60` | FunctionObjectT_FundWallet_ConnectStep1 | likely | Connect poll for the FundWallet instance (same shape as 00f630f0): status 0 with a false 00f92720 result gives the -4 completion (url at this+0x32c, state +0x4… |
| `00f64bc0` | FunctionObjectT_FundWallet_ConnectStep2 | likely | Second connect-phase poll for the FundWallet instance (same shape as 00f63240): status 1 with GiveTime failure gives -5 (url at this+0x32c, state 6, callback s… |
| `00f64d20` | FunctionObjectT_FundWallet_DtorBody | likely | Destructor body (does not free this): installs the FundWallet FunctionObjectT vtable, calls 00f99910, 00f91de0, 00f99670, 00f603d0, 00f98190, 00f86880 and 00f7… |
| `00f64de0` | FunctionObjectT_FundWallet_ScalarDeletingDtor | likely | Scalar deleting destructor: calls 00f64d20 on this, frees this through _free when the flag byte has bit 0 set, returns this. |
| `00f64e00` | FunctionObjectT_FundWallet_SendStep | likely | Send step for the FundWallet instance (same shape as 00f636d0): calls 00f843b0 first (request body build, not decoded), then sends through 00f99920; send failu… |
| `00f647b0` | FunctionObjectT_FundWallet_Ctor | speculative | Constructor body: stores the FundWallet FunctionObjectT vtable at +0x00 and param_2 at +0x04; copies the twelve-string request from param_3 through 00f61190; c… |
| `00f64940` | FunctionObjectStateIs6Or7_FundWallet | speculative | Returns 1 when the state dword at this+0x4f4 is 6 or 7, else 0 (__fastcall, this in ECX). |
| `00f64960` | FunctionObjectT_FundWallet_SetState5 | speculative | Stores 5 into the state dword at this+0x4f4 and returns. |
| `00f64970` | — | unknown | FireEvent for the FundWallet instance: when the error argument is non-zero, logs 'FunctionObjectT::FireEvent, trackingId=%u' with the tracking id at +0x4ec and… |
| `00f649f0` | — | unknown | Logs 'FunctionObjectT::FireEvent, trackingId=%u' with the tracking id at +0x4ec for error 0xffffffef (-17), sets state +0x4f4 to 6, and calls slot 2 of the cal… |

**Uram::FunctionObjectT<Uram::GenericRequest,Uram::GenericResponse,Uram::GenericRequestCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6ddf0` | FunctionObjectT::Ctor | confirmed | Constructor with ten stack arguments (ret 0x28): stores argument 2 at this+4, installs the GenericRequest vftable, calls 00f62580 with argument 3, stores argum… |
| `00f6df70` | FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the dword at this+0x3a4 is 6 or 7, otherwise 0. Read-only. |
| `00f6dfa0` | FunctionObjectT::FireEvent | confirmed | If argument 2 (error code) is non-zero: formats the error text with 00f8de10, logs the FireEvent line at level 2 on channel this+0x39c, and sets state this+0x3… |
| `00f6e010` | FunctionObjectT::FireEventErr17 | confirmed | Unconditionally formats error -17 (0xffffffef) through 00f8de10, logs the FireEvent line at level 2 on channel this+0x39c, sets state this+0x3a4 to 6 and calls… |
| `00f6e080` | FunctionObjectT::ConnectStep0 | confirmed | Connect-phase step with class-E offsets: calls 00f91e00; result 0 runs GiveTime 00f92720 and fails with -4 on a 0 return; result 1 sets state this+0x3a4 to 2; … |
| `00f6e1d0` | FunctionObjectT::ConnectStep1 | confirmed | Connect-phase step with class-E offsets: result 1 runs GiveTime 00f92720 and fails with -5 on a 0 return; result 2 sets state this+0x3a4 to 3; other results lo… |
| `00f6e320` | FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Sets the Generic vftable, calls teardown helpers 00f99910 (no-op stub), 00f91de0, 00f99670, 00f603d0, 00f98190, then 00f8… |
| `00f6e3d0` | FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6e320, then frees this when flag bit 0 of the second argument is set, and returns this. |
| `00f6e3f0` | FunctionObjectT::Send | confirmed | Send step with class-E offsets: builds the request body through 00f83820 (length at this+0x78/+0x7c), builds the header block through 00f96090, sends header th… |
| `00f6e730` | FunctionObjectT::CheckDone | confirmed | Poll step for the Generic response with class-E offsets (response state this+0x210, size this+0x360, complete flag this+0x36c, status this+0x218, state this+0x… |
| `00f74390` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump, same shape as 00f741d0. State dword at this+0x3a4 selects 00f72910, 00f6e080, 00f6e1d0, 00f6e3f0, 00f6e730 (cases 0 to 4) … |
| `00f6df90` | FunctionObjectT::SetState5 | speculative | Stores the constant 5 into the dword at this+0x3a4 and returns. |

**Uram::FunctionObjectT<Uram::IsAffiliatedWithPartnerRequest,Uram::IsAffiliatedWithPartnerResponse,Uram::IsAffiliatedWithPartnerCallback>** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6cb70` | FunctionObjectT::Ctor | confirmed | Constructor with ten stack arguments (ret 0x28): stores argument 2 at this+4, installs the IsAffiliatedWithPartnerRequest vftable, calls 00f62360 with argument… |
| `00f6cd00` | FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the dword at this+0x40c is 6 or 7, otherwise 0. Read-only. |
| `00f6cd30` | FunctionObjectT::FireEvent | confirmed | If argument 2 (error code) is non-zero: formats the error text with 00f8de10, logs the FireEvent line at level 2 on channel this+0x404, and sets state this+0x4… |
| `00f6cda0` | FunctionObjectT::FireEventErr17 | confirmed | Unconditionally formats error -17 (0xffffffef) through 00f8de10, logs the FireEvent line at level 2 on channel this+0x404, sets state this+0x40c to 6 and calls… |
| `00f6ce10` | FunctionObjectT::ConnectStep0 | confirmed | Connect-phase step with class-C offsets: calls 00f91e00; result 0 runs GiveTime 00f92720 and fails with -4 on a 0 return; result 1 sets state this+0x40c to 2; … |
| `00f6cf60` | FunctionObjectT::ConnectStep1 | confirmed | Connect-phase step with class-C offsets: result 1 runs GiveTime 00f92720 and fails with -5 on a 0 return; result 2 sets state this+0x40c to 3; other results lo… |
| `00f6d0b0` | FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Sets the IsAffiliatedWithPartner vftable, calls teardown helpers 00f99910 (no-op stub), 00f91de0, 00f99670, 00f603d0, 00f… |
| `00f6d170` | FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6d0b0, then frees this when flag bit 0 of the second argument is set, and returns this. |
| `00f6d190` | FunctionObjectT::Send | confirmed | Send step with class-C offsets: builds the request body through 00f81be0 (length at this+0xe0/+0xe4), builds the header block through 00f96090, sends header th… |
| `00f76df0` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the IsAffiliatedWithPartner GiveTime. Requires the transport word at this+0x278 to be 2. Size at this+0x3c8 above 0x100000 f… |
| `00f77d70` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x40c: cases 0 to 3 call 00f720d0, 00f6ce10, 00f6cf60, 00f6d190; case 4 calls the CheckD… |
| `00f6cd20` | FunctionObjectT::SetState5 | speculative | Stores the constant 5 into the dword at this+0x40c and returns. |

**Uram::FunctionObjectT<Uram::PreviewOrderRequest,Uram::PreviewOrderResponse,Uram::PreviewOrderCallback>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f66630` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the PreviewOrder FunctionObjectT vtable, calls 00f617b0 and 00f8b780 (PreviewOrderResponse construct… |
| `00f667c0` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x50c is 6 or 7, otherwise 0. No calls. |
| `00f667e0` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x50c. No calls, returns nothing. |
| `00f66920` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17, via 00f8de10 and 00f91dd0), sets state this+0x50c = 6, and calls the c… |
| `00f66a20` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | PreviewOrder layout (url +0x344, log +0x504, state +0x50c, callback +0x4fc). Branches on the status from 00f91e00: status 0 with 00f92720 returning 0 logs the … |
| `00f66b80` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | PreviewOrder layout. Branches on the status from 00f91e00: status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state th… |
| `00f66d00` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the PreviewOrder FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f98190, … |
| `00f66dc0` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f66d00, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f66de0` | Uram::FunctionObjectT::SendRequest | confirmed | PreviewOrder layout (url +0x344, log +0x504, state +0x50c, callback +0x4fc). Builds the request body as an IString through 00f82610, stores a 64-bit value (fro… |
| `00f77f30` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the PreviewOrder GiveTime. Requires the transport word at this+0x378 to be 2. Size at this+0x4c8 above 0x100000 fires -16. C… |
| `00f783b0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x50c: cases 0 to 3 call 00f6ff90, 00f66a20, 00f66b80, 00f66de0; case 4 calls the CheckD… |

**Uram::FunctionObjectT<Uram::PurchaseOrderRequest,Uram::PurchaseOrderResponse,Uram::PurchaseOrderCallback>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f67540` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the PurchaseOrder FunctionObjectT vtable, constructs the PurchaseOrderResponse block through 00f89a6… |
| `00f676d0` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x4ac is 6 or 7, otherwise 0. No calls. |
| `00f676f0` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x4ac. No calls, returns nothing. |
| `00f677d0` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17), sets state this+0x4ac = 6, and calls the callback at this+0x49c vtabl… |
| `00f67880` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | Purchase layout (url +0x2e4, log +0x4a4, state +0x4ac, callback +0x49c). Branches on the status from 00f91e00: status 0 with 00f92720 returning 0 logs the SSL … |
| `00f679e0` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | Purchase layout. Status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state this+0x4ac = 6 and calls callback slot +0x18… |
| `00f67b50` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the PurchaseOrder FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f98190,… |
| `00f67c10` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f67b50, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f67c30` | Uram::FunctionObjectT::SendRequest | confirmed | Purchase layout (url +0x2e4, log +0x4a4, state +0x4ac, callback +0x49c). Builds the request body as an IString through 00f816f0, stores a 64-bit value (from th… |
| `00f760c0` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the PurchaseOrder GiveTime. Requires the transport word at this+0x318 to be 2. Size at this+0x468 above 0x100000 fires -16. … |
| `00f779f0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x4ac: cases 0 to 3 call 00f703d0, 00f67880, 00f679e0, 00f67c30; case 4 calls the CheckD… |

**Uram::FunctionObjectT<Uram::RedeemCodeRequest,Uram::RedeemCodeResponse,Uram::RedeemCodeCallback>** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f68090` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the RedeemCode FunctionObjectT vtable, calls 00f61b80 and 00f8d250 for the embedded record, stores t… |
| `00f68220` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x4b4 is 6 or 7, otherwise 0. No calls. |
| `00f68240` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x4b4. No calls, returns nothing. |
| `00f68450` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17), sets state this+0x4b4 = 6, and calls the callback at this+0x4a4 vtabl… |
| `00f685e0` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | RedeemCode layout (url +0x2ec, log +0x4ac, state +0x4b4, callback +0x4a4). Branches on the status from 00f91e00: status 0 with 00f92720 returning 0 logs the SS… |
| `00f68740` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | RedeemCode layout. Status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state 6 and calls callback slot +0x1c with (trac… |
| `00f68c00` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the RedeemCode FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f98190, 00… |
| `00f68cc0` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f68c00, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f68ce0` | Uram::FunctionObjectT::SendRequest | confirmed | RedeemCode layout (url +0x2ec, log +0x4ac, state +0x4b4, callback +0x4a4). Builds the request body as an IString through 00f84e10, stores a 64-bit value (from … |
| `00f78b50` | — | unknown | Thiscall state handler called by the RedeemCode GiveTime (its switch cases 0 to 4 in 00f78f50 call this). Requires the transport word at this+0x320 to be 2. Si… |

**Uram::FunctionObjectT<Uram::UpdateCreditCardRequest,Uram::UpdateCreditCardResponse,Uram::UpdateCreditCardCallback>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f69f40` | Uram::FunctionObjectT::Ctor | confirmed | Constructor: stores the first argument at this+4, installs the UpdateCreditCard FunctionObjectT vtable, calls 00f61da0 and 00f87b50 for the embedded record, st… |
| `00f6a0d0` | Uram::FunctionObjectT::StateIs6Or7 | confirmed | Returns 1 when the state field this+0x4cc is 6 or 7, otherwise 0. No calls. |
| `00f6a0f0` | Uram::FunctionObjectT::SetState5 | confirmed | Stores 5 into the state field this+0x4cc. No calls, returns nothing. |
| `00f6a180` | Uram::FunctionObjectT::FireEventCodeMinus17 | confirmed | Logs FunctionObjectT::FireEvent with this+4 and the error text for code 0xffffffef (-17), sets state this+0x4cc = 6, and calls the callback at this+0x4bc vtabl… |
| `00f6a1f0` | Uram::FunctionObjectT::ConnectStatus0Handler | confirmed | UpdateCreditCard layout (url +0x304, log +0x4c4, state +0x4cc, callback +0x4bc with data +0x150). Branches on the status from 00f91e00: status 0 with 00f92720 … |
| `00f6a350` | Uram::FunctionObjectT::ConnectStatus1Handler | confirmed | UpdateCreditCard layout. Status 1 with 00f92720 returning 0 logs the SSL GiveTime failure with -5 (0xfffffffb), sets state 6 and calls callback slot +0x24 with… |
| `00f6a4b0` | Uram::FunctionObjectT::DtorBody | confirmed | Destructor body (does not free this). Stores the UpdateCreditCard FunctionObjectT vtable at this+0, calls 00f99910 (stub), 00f91de0, 00f99670, 00f603d0, 00f981… |
| `00f6a570` | Uram::FunctionObjectT::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 00f6a4b0, then frees this through _free when flag bit 0 of the second argument is set. Returns this. |
| `00f6a590` | Uram::FunctionObjectT::SendRequest | confirmed | UpdateCreditCard layout (url +0x304, log +0x4c4, state +0x4cc, callback +0x4bc with data +0x150). Builds the request body as an IString through 00f81880, store… |
| `00f76750` | FunctionObjectT::CheckDone | confirmed | Thiscall state-4 handler called by the UpdateCreditCard GiveTime. Requires the transport word at this+0x338 to be 2. Size at this+0x488 above 0x100000 fires -1… |
| `00f77bb0` | FunctionObjectT::GiveTime | confirmed | Virtual vftable[1] time-sliced pump. Switch on the state dword at this+0x4cc: cases 0 to 3 call 00f71060, 00f6a1f0, 00f6a350, 00f6a590; case 4 calls the CheckD… |

**Uram::ObjectReportHandlerAdapter** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f79080` | Uram::ObjectReportHandlerAdapter::ForwardSlot40 | likely | If the pointer at this+4 is non-null, tail-jumps through vtable slot +0x40 of the object at this+4 with ECX set to that object; otherwise returns with ret 4. B… |
| `00f790a0` | Uram::ObjectReportHandlerAdapter::ForwardSlot44 | likely | Same shape as 00f79080: if this+4 is non-null it tail-jumps through vtable slot +0x44 of the object at this+4 (ECX = that object); otherwise ret 4. |
| `00f790c0` | Uram::ObjectReportHandlerAdapter::ForwardSlot48 | likely | Same shape as 00f79080: if this+4 is non-null it tail-jumps through vtable slot +0x48 of the object at this+4 (ECX = that object); otherwise ret 4. |

**Uram::RedeemCodeResponse** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8d990` | RedeemCodeResponse::Parse | confirmed | Parses the RedeemCode XML reply: arg2 is the document buffer (loaded through 00d549d0; TiXmlDocument vftable 017ed9dc stored on exit; freed through 00d51500 an… |
| `00f8cd00` | RedeemCodeResponse::DtorBody | likely | Destructor body (does not free this): destroys the RedeemCodeResponseOption arrays at +0xc4 and +0xb4 through 00f68580 and 00401c00, the Error array at +0xa4 t… |
| `00f8d250` | RedeemCodeResponse::Ctor | likely | Constructor body: installs IString vftables at +0x00, +0x10, +0x20 and +0x30 with the static empty-string buffer; constructs six IStrings at +0x44 (eh_vector_c… |
| `00f8d350` | RedeemCodeResponse::ParseOptionChildren | likely | Walks the child list of arg1 and matches each child name against the 30-entry key table at 0x01b975d0 (pointer) and 0x01b975d4 (length), stride 8, through 0040… |
| `00f8d910` | RedeemCodeResponse::WalkOptionGroups | likely | Walks the child list of arg1; for each child whose name matches key index 21 of the RedeemCode key table (0x01b975d0, stride 8) it calls 00f8d350 with arg2, an… |

**Uram::RedeemCodeResponseOption** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f751e0` | RedeemCodeResponseOption::CopyCtor | likely | Thiscall copy-constructor of a 0x4c-byte element. Sets the StationCashGrant array vftable at +0x00 and copies its ptr and count from the source with 00f73220; … |

**(no class)** (170)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f810a0` | AppendSessionLocaleThemeQuery | confirmed | Thiscall: this (request-info object) in ECX, destination SoeUtil IString* as the one stack argument (ret 4). Ensures the destination has capacity >=0x201 throu… |
| `00f81150` | AppendSessionPaymentQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x401 through 00401df0 when needed, then appends "sessionID=" + this+0… |
| `00f81240` | AppendPaymentSourceForm | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x801 through 00401df0 when needed, then appends a long form body: "se… |
| `00f81470` | AppendSessionLocaleThemeMinimalQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x201 through 00401df0 when needed, appends "sessionID=" + this+0x14, … |
| `00f816f0` | AppendPromotionOrderQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x401 through 00401df0 when needed, then appends "sessionID=" + this+0… |
| `00f81820` | AppendLocaleThemeQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x201 through 00401df0 when needed, appends "&locale=" + this+0x04 unc… |
| `00f81880` | AppendPaymentSourceIdForm | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x801 through 00401df0 when needed, then appends a payment-source form… |
| `00f81ab0` | AppendPaymentSourceIdParentQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x801 through 00401df0 when needed, appends "sessionID=" + this+0x34, … |
| `00f81b60` | AppendSessionTokenQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x801 through 00401df0 when needed, then appends "sessionToken=" plus … |
| `00f81ba0` | AppendSessionTokenQuery | confirmed | Byte-equivalent copy of 00f81b60: reserves capacity 0x801 through 00401df0 when needed, then appends "sessionToken=" plus the C string at this+0x04 through 004… |
| `00f81be0` | AppendSessionGameSteamQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument. Reserves capacity 0x801 through 00401df0 when needed, appends "sessionID=" + this+0x04 unconditi… |
| `00f81d30` | AppendOrderAuthorizedQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x801 through 00401df0 when needed, then appends "sessionID=" + this+0… |
| `00f82610` | AppendAddressPromotionQuery | confirmed | Thiscall: this in ECX, destination IString* as stack argument (ret 4). Reserves capacity 0x401 through 00401df0 when needed, then appends "sessionID=" + this+0… |
| `00f83590` | GetTicketResponseParse | confirmed | Thiscall with two stack arguments (ret 8). Loads the first argument as an XML document through 00d549d0 into a local TiXmlDocument, takes the root element with… |
| `00d70f20` | Wrapper_00d6e830_PAYPAL | likely | 43-byte stdcall-style forwarder (ends ret 0x14, five stack arguments): pushes the constant string 'PAYPAL' (0x1842b74) and the five arguments to 00d6e830 and r… |
| `00f60c20` | TimedDequePump | likely | Round-robin service loop over the deque whose map pointer is at this+0x20 with count at this+0x40 and start index at this+0x44. Per item it pops the front (00f… |
| `00f60db0` | UploaderImplConfigAdjustor | likely | Thunk: mov ecx,[ecx] (loads the implementation pointer stored at offset 0 of the wrapper) then jumps to 00f60a40 with the arguments unchanged. |
| `00f60dc0` | TimedDequePumpAdjustor | likely | Thunk: mov ecx,[ecx] (loads the implementation pointer from offset 0 of the wrapper) then jumps to 00f60c20 with the budget argument unchanged. |
| `00f62b70` | HttpStatusToUramError | likely | Takes an HTTP status code in EAX and returns a negative error code: 400 gives -9, 401 (0x191) gives -11, 403 (0x193) gives -12, 500 gives -13, 501 (0x1f5) give… |
| `00f74790` | Wrapper_00f74790 | likely | Seven-byte thunk: mov ecx,[ecx] replaces ECX with the dword stored at the object it was given, then jmp 00f733d0 with the stack arguments unchanged. |
| `00f747a0` | Wrapper_00f747a0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f734b0 with the stack arguments unchanged. |
| `00f747b0` | Wrapper_00f747b0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73590 with the stack arguments unchanged. |
| `00f747c0` | Wrapper_00f747c0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73670 with the stack arguments unchanged. |
| `00f747d0` | Wrapper_00f747d0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73750 with the stack arguments unchanged. |
| `00f747e0` | Wrapper_00f747e0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73830 with the stack arguments unchanged. |
| `00f747f0` | Wrapper_00f747f0 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73910 with the stack arguments unchanged. |
| `00f74800` | Wrapper_00f74800 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f739f0 with the stack arguments unchanged. |
| `00f74820` | Wrapper_00f74820 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73bb0 with the stack arguments unchanged. |
| `00f74830` | Wrapper_00f74830 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73c90 with the stack arguments unchanged. |
| `00f74860` | Wrapper_00f74860 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73f30 with the stack arguments unchanged. |
| `00f74870` | Wrapper_00f74870 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f74010 with the stack arguments unchanged. |
| `00f74880` | Wrapper_00f74880 | likely | Seven-byte thunk: mov ecx,[ecx] then jmp 00f740f0 (UploaderImpl::AddRequest) with the stack arguments unchanged. |
| `00f796b0` | ParseBoolTrueText | likely | Reads the text at (object at this+0x18 -> vtable +0x30 result) +0x20 +8. Returns 0 when the text is null, empty, or its length is not 4 or 5. Otherwise calls S… |
| `00f79720` | ParseFirstDigitIntText | likely | Gets the text from the same node chain as 00f796b0 (this+0x18 -> vtable +0x30 -> +0x20 -> +8). Scans characters until one passes the iswctype digit test, then … |
| `00f79990` | AllRequestFieldsSet_08_28_48_68_78_58 | likely | Returns true only when this+0x68, this+0x78, this+0x48, this+0x28 and this+0x08 are all non-null and this+0x58 is non-null; otherwise returns false. Pure predi… |
| `00f81f50` | PaymentInfoChildWalk | likely | Stdcall, one stack argument (root node, ret 4), destination IString in ECX. Walks the child list from root+0x18 following next at +0x28. For each child it call… |
| `00f82020` | AppendStationCashPreviewText | likely | Thiscall: this (preview object) in ECX, destination IString* as the one stack argument (ret 4). Appends the literal "StationCashProductPreviewed_BEG ", then fo… |
| `00f820c0` | ParseFundWalletPreviewChildren | likely | Thiscall: this (result object) in ECX, root node as the one stack argument (ret 4). Walks the child list from root+0x18 (next at +0x28). For each child, vtable… |
| `00f82230` | AppendStationCashOrderText | likely | Thiscall: this (order object) in ECX, destination IString* as the stack argument (ret 4). Appends "StationCashProductOrdered_BEG " through 00402770, then forma… |
| `00f822e0` | ParseFundWalletResponseChildren | likely | Thiscall: this (result object) in ECX, root node as the stack argument (ret 4). Walks the child list from root+0x18 (next at +0x28). For each child, vtable slo… |
| `00f82460` | ParseCreditCardTypesChildren | likely | Thiscall: this (result object) in ECX, root node as the one stack argument (ret 4). Walks every child of the root (list at root+0x18, next at +0x28; each child… |
| `00f82570` | AppendCreditCardTypeText | likely | Thiscall: this in ECX, destination IString* as the stack argument (ret 4; the destination is passed to 00402ff0 each time). Appends "CreditCardType_BEG " throu… |
| `00f828b0` | PreviewEntryFieldsParse | likely | Walks the child list of the stack-argument node (head at +0x18, next at +0x28). For each child whose value (vtable +0x14) is non-null, compares the child's nam… |
| `00f82a00` | PreviewEntryNextFieldsParse | likely | Walks the child list of the stack-argument node (head +0x18, next +0x28) and compares each child name with 00401940 against the 35-entry key table at 0x01b9743… |
| `00f82cb0` | SoftwareProductAppendText | likely | Appends a text block to the string object passed as the stack argument: the literal SoftwareProduct_BEG, then name="%s" from the char pointer at this+4, price=… |
| `00f82d30` | NextDetailsAppendText | likely | Appends a text block to the string object passed as the stack argument: the literal NextDetails_BEG, then isDiscounted=%d from the byte at this+0, priceSubtota… |
| `00f82f30` | RedeemCodeChildTextKeys12To16 | likely | Walks the child list of the stack-argument node (head +0x18, next +0x28) and compares each child name with 00401940 against the 30-entry key table at 0x01b975d… |
| `00f83060` | RedeemCodeChildWalkKey16And26 | likely | Walks every child of the stack-argument node (head +0x18, next +0x28) with no early exit. Each child name is compared with 00401940 against the 30-entry key ta… |
| `00f83110` | RedeemCodeChildKeys13To16 | likely | Walks the child list of the stack-argument node (head +0x18, next +0x28). Each child name is compared with 00401940 against the 30-entry key table at 0x01b975d… |
| `00f83310` | CountryCodesChildTextKeys5And6 | likely | Walks the child list of the stack-argument node (head +0x18, next +0x28). Each child name is compared with 00401940 against the 8-entry key table at 0x01b976c0… |
| `00f83430` | TextAssignTrimThenCall | likely | Takes two stack arguments. If the first is non-null, assigns it to the IString at ECX with IStringAssign, then calls 0097a940 on ECX (buffer grow and whitespac… |
| `00f834a0` | CasKeepAliveResponseTempText | likely | Fastcall with the object in ECX. Builds a temporary SoeUtil::IString holding CasKeepAliveResponse_BEG, then status="%s" from the char pointer at this+4, then C… |
| `00f83730` | CasGetSessionTicketResponseTempText | likely | Fastcall with the object in ECX. Builds a temporary SoeUtil::IString holding CasGetSessionTicketResponse_BEG, then status="%s" from the char pointer at this+4,… |
| `00f83830` | GenericResponseAppendText | likely | Thiscall with this in ECX and one stack argument (the destination string). Appends the literal GenericResponse_BEG, then payload="%s" with the char pointer at … |
| `00f83870` | GenericResponseTempText | likely | Fastcall with the object in ECX. Builds a temporary SoeUtil::IString holding GenericResponse_BEG, then payload="%s" from the char pointer at this+4, then Gener… |
| `00f83960` | RegisteredServiceListAppendText | likely | Thiscall with this in ECX (element array pointer at this+4, count at this+8) and three stack arguments: a head literal, a tail literal and the destination stri… |
| `00f83a50` | ErrorReasonListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: a head literal, a tail literal and the destination string. Appe… |
| `00f83b20` | CountedList28AppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: a head literal, a tail literal and the destination string. Appe… |
| `00f83bf0` | CountedList44AppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: a head literal, a tail literal and the destination string. Appe… |
| `00f83cc0` | SubscriptionProductListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f83dc0` | StationCashGrantListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f83ec0` | FeatureGrantListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f83fd0` | FeatureListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and two stack arguments: a head literal and the destination string (ECX for the appends). … |
| `00f84090` | CountryListAppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f841b0` | CardRecordAppendText | likely | Thiscall with this in ECX and one stack argument (the destination string). Appends Card_BEG, then cardtype="%s" from this+0x24, cardspan="%s" from this+0x14, p… |
| `00f84240` | FundWalletPreviewResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends FundWalletPreviewResponse_BEG, then version="%s" from this+0x… |
| `00f843b0` | PaymentQueryStringAppend | likely | Thiscall with the request object in ECX and one stack argument: the destination string (capacity at +0xc, grown to 0x401 bytes when smaller). Appends sessionID… |
| `00f84540` | FundWalletResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends FundWalletResponse_BEG, then version="%s" from this+0x24, sta… |
| `00f846d0` | AddPaymentSourceResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends AddPaymentSourceResponse_BEG, then version="%s" from this+0x2… |
| `00f84800` | CreditCardTypesResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends CreditCardTypesResponse_BEG, then version="%s" from this+0x14… |
| `00f84950` | PreviewChildWalkKey30 | likely | Stdcall-style with two stack arguments (ret 8) and no use of ECX: the first argument is an element whose child list is walked (head +0x18, next +0x28); the sec… |
| `00f849c0` | PreviewEntryFieldsWithNextParse | likely | Thiscall with the output object in EBX (ECX at entry) and one stack argument: the element whose child list is walked (head +0x18, next +0x28). For each child w… |
| `00f84b90` | SubscriptionProductAppendText | likely | Thiscall with this in ECX and one stack argument (the destination string). Appends SubscriptionProduct_BEG, then name="%s" from this+4, price=%s from this+0x14… |
| `00f84c60` | PurchaseOrderResponseAppendText | likely | Thiscall with this in EDI and one stack argument (the destination string in ESI). Appends PurchaseOrderResponse_BEG, then version="%s" from this+0x24, status="… |
| `00f84e10` | ActivationQueryStringAppend | likely | Thiscall with the request object in ECX and one stack argument: the destination string (capacity at +0xc, grown to 0x201 bytes when smaller). Appends sessionID… |
| `00f84ef0` | OptionAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string in ESI). Appends Option_BEG, then gameCode="%s" from this+0x38, id=%d f… |
| `00f84fb0` | CountryCodesResponseAppendText | likely | Thiscall with this in EDI and one stack argument (the destination string in ESI). Appends CountryCodesResponse_BEG, then version="%s" from this+0x14 and status… |
| `00f85100` | UpdateCreditCardResponseAppendText | likely | Thiscall with this in EDI and one stack argument (the destination string in ESI). Appends UpdateCreditCardResponse_BEG, then version="%s" from this+0x14 and st… |
| `00f85220` | DeletePaymentSourceResponseAppendText | likely | Thiscall with this in EDI and one stack argument (the destination string in ESI). Appends DeletePaymentSourceResponse_BEG, then version="%s" from this+0x14 and… |
| `00f85340` | IsAffiliatedWithPartnerResponseAppendText | likely | Thiscall with this in EDI and one stack argument (the destination string in ESI). Appends IsAffiliatedWithPartnerResponse_BEG, then version="%s" from this+0x14… |
| `00f85480` | FinalizeThirdPartyOrderTransactionResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends FinalizeThirdPartyOrderTransactionResponse_BEG, then version=… |
| `00f855a0` | CountedList72AppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f85690` | CountedList168AppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f85790` | CountedList76AppendText | likely | Thiscall with this in ECX (element array at this+4, count at this+8) and three stack arguments: head literal, tail literal and destination string. Appends the … |
| `00f86130` | AccountPaymentInfoCardDtorBody | likely | Fastcall with the record in ECX (kept as param_1). Sets the vtable of SoeUtil::Array<Uram::AccountPaymentInfoResponse::Card::RegisteredService,0,1> at +0x30, c… |
| `00f86230` | AccountPaymentInfoResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument (the destination string). Appends AccountPaymentInfoResponse_BEG, then version="%s" from this+0… |
| `00f86480` | RecordWithErrorArrayDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x9c, calls 00f63010 with +0xa4 pushed, … |
| `00f86880` | WideRecordDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0xc4, calls 00f63010 with +0xcc pushed, … |
| `00f86a60` | WideRecordReset | likely | Fastcall with the record in ECX (kept as param_1). For the IString members at +0x00, +0x10, +0x20, +0x48, +0x58 and +0x68 it releases a refcounted buffer when … |
| `00f86bb0` | ThreeStringsAndErrorArrayCtor | likely | Fastcall constructor with the record in ECX (kept as param_1). Sets three IString members at +0x00, +0x10 and +0x20 to the empty state (vtable SoeUtil::IString… |
| `00f86c00` | ThreeStringsAndErrorArrayDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x30, calls 00f63010 with +0x38 pushed, … |
| `00f86d00` | ThreeStringsAndArrayReset | likely | Fastcall with the record in ECX (kept as param_1). Releases the IString members at +0x00, +0x10 and +0x20 (refcounted buffers freed through vtable slot 2 on th… |
| `00f86f00` | TwoStringsAndArraysReset | likely | Fastcall with the record in ECX (kept as param_1). Releases the IString members at +0x00 and +0x10 (refcounted buffers freed through vtable slot 2 on the last … |
| `00f86fe0` | PreviewOrderResponseAppendText | likely | Thiscall with this in EDI (ECX at entry) and one stack argument: the destination string, kept in ESI. Appends PreviewOrderResponse_BEG, then version="%s" from … |
| `00f87250` | PurchaseOrderResponseDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x90, calls 00f63010 with +0x98 pushed, … |
| `00f87480` | EightStringsAndArraysReset | likely | Fastcall with the record in ECX (kept as param_1). For the IString members at +0x00, +0x10, +0x20, +0x30, +0x40, +0x50, +0x60 and +0x70 it releases a refcounte… |
| `00f876c0` | RedeemCodeResponseOptionDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::RedeemCodeResponseOption::FeatureGrant,0,1> vtable at +0x10, c… |
| `00f877b0` | RedeemCodeResponseAppendText | likely | Thiscall with this in EBP (ECX at entry) and one stack argument (the destination string). Appends RedeemCodeResponse_BEG, version="%s" from this+0x34, status="… |
| `00f879c0` | CountryCodesResponseDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x30, calls 00f63010 with +0x38 pushed, … |
| `00f87ac0` | TwoStringsAndArraysResetAlt | likely | Fastcall with the record in ECX (kept as param_1). Releases the IString members at +0x00 and +0x10 (refcounted buffers freed through vtable slot 2 on the last … |
| `00f87b50` | TwoStringsAndErrorArrayCtor | likely | Fastcall constructor with the record in ECX (kept as param_1). Sets two IString members at +0x00 and +0x10 to the empty state (vtable SoeUtil::IString<char>, s… |
| `00f87b90` | TwoStringsAndErrorArrayDtorBody | likely | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x20, calls 00f63010 with +0x28 pushed, … |
| `00f87c60` | TwoStringsAndErrorArrayReset | likely | Fastcall with the record in ECX (kept as param_1). Releases the IString members at +0x00 and +0x10 (refcounted buffers freed through vtable slot 2 on the last … |
| `00f87ce0` | TwoStringsAndErrorArrayCtor | likely | Fastcall constructor, identical in body to 00f87b50: two IString members at +0x00 and +0x10 set to the empty state (vtable SoeUtil::IString<char>, sentinel 0x0… |
| `00f87d20` | TwoStringsAndErrorArrayDtorBody | likely | Fastcall destructor body, same body as 00f87b90: sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x20, calls 00f63010 with +0x28 pushed, frees the buffer a… |
| `00f87e70` | ThreeStringsAndErrorArrayDtorBody | likely | Destructor body (does not free this). Stores the SoeUtil::Array<Uram::Error> vtable at this+0x34, pops its elements with 00f63010 (count at +0x3c) and frees it… |
| `00f88020` | TwoStringsAndErrorArrayDtorBody | likely | Destructor body (does not free this). Stores the SoeUtil::Array<Uram::Error> vtable at this+0x20, pops its elements with 00f63010 (count at +0x28) and frees th… |
| `00f886b0` | ParseFundWalletPreviewResponseXml | likely | Parses an XML document (second argument) with the TiXml load 00d549d0 and root lookup 00d50ac0. Requires root tag fundwalletpreviewresponse through 00401940, t… |
| `00f892d0` | CreditCardTypesChildWalk | likely | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against a 13-entry key table at 0x01b… |
| `00f89e00` | RedeemCodeChildWalkKey28 | likely | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against a 30-entry key table at 0x01b… |
| `00f89f30` | RedeemCodeChildWalkKey29 | likely | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against the 30-entry redeemcoderespon… |
| `00f8a0b0` | RedeemCodeResponseFields | likely | Fills the redeemcoderesponse object (this, first argument) from the child elements of the second argument (children at +0x18, next at +0x28). Child names are m… |
| `00f8ab50` | ThreeStringsAndErrorArrayCtor | likely | Constructor for an unbound Uram record: IString-shaped members at this+0x00, +0x10 and +0x20 set to the empty sentinel 0x01cb4080 with length and capacity zero… |
| `00f8b8a0` | PurchasePreviewChildAppend | likely | Walks the child list of the element passed as the first argument (head +0x18, next +0x28). Each child name is compared through 00401940 against the 35-entry pu… |
| `00f8be50` | RedeemCodeEntryFields | likely | Fills a redeem-code entry (this, first argument) from the child elements of the second argument, with destination block param_3. Child names are matched throug… |
| `00d70f50` | Wrapper_00d6e830_SMS | speculative | 43-byte stdcall-style forwarder (ends ret 0x14, five stack arguments), the same shape as 00d70f20: pushes the address 0x1842b7c (bytes 'SMS' followed by a NUL)… |
| `00f62cc0` | FunctionObjectStateIs6Or7 | speculative | Reads the dword at this+0x42c; returns 1 when it equals 6 or 7 and 0 otherwise (__fastcall, this in ECX). |
| `00f62de0` | ScalarDeletingDtorBody00f7ca00 | speculative | Scalar deleting destructor shape: calls the destructor body 00f7ca00 on this, frees this through _free when param bit 0 is set, returns this (ret 4). |
| `00f62e60` | ScalarDeletingDtorBody00f86130 | speculative | Scalar deleting destructor shape: calls the destructor body 00f86130 on this, frees this through _free when param bit 0 is set, returns this (ret 4). |
| `00f62e80` | ScalarDeletingDtorBody00f7c6a0 | speculative | Scalar deleting destructor shape: calls the destructor body 00f7c6a0 on this, frees this through _free when param bit 0 is set, returns this (ret 4). |
| `00f6c7d0` | ThreeIStringAssign | speculative | Copy-assigns three IString-shaped members: this+0x00, +0x10 and +0x20 receive the same-offset members of the stack argument, one call to 006f9e60 (IString copy… |
| `00f74840` | Wrapper_00f74840 | speculative | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73d70 with the stack arguments unchanged. |
| `00f74850` | Wrapper_00f74850 | speculative | Seven-byte thunk: mov ecx,[ecx] then jmp 00f73e50 with the stack arguments unchanged. |
| `00f74b40` | RecordCopyAssign_00f74b40 | speculative | Thiscall copy-assign of a response record from the stack argument. IString assign (006f9e60) for source +0x00, +0x10, +0x20, +0x30, +0x40, +0x50, +0x68, +0x78 … |
| `00f74c20` | RecordCopyAssign_00f74c20 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00, +0x10, +0x20, +0x38, +0x48, +0x58 and +0x68; dwords +0x30 and +0x34 copi… |
| `00f74ce0` | RecordCopyAssign_00f74ce0 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00, +0x10 and +0x20; the array count at this+0x38 is cleared with 00f63010, … |
| `00f74d50` | RecordCopyAssign_00f74d50 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00 and +0x10; the array count at this+0x28 is cleared with 00f65e00 and then… |
| `00f74dd0` | RecordCopyAssign_00f74dd0 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00, +0x10, +0x20, +0x30, +0x40, +0x50, +0x60 and +0x70; the array count at t… |
| `00f74eb0` | RecordCopyAssign_00f74eb0 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00 and +0x10; the array count at this+0x28 is cleared with 00f69720 and 00f7… |
| `00f74f30` | RecordCopyAssign_00f74f30 | speculative | Thiscall copy-assign of a two-string response record. IString assign (006f9e60) for source +0x00 and +0x10; the array count at this+0x28 is cleared with 00f630… |
| `00f74f90` | RecordCopyAssign_00f74f90 | speculative | Thiscall copy-assign of a two-string response record, identical in body to 00f74f30: IString assign for source +0x00 and +0x10; array count at this+0x28 cleare… |
| `00f74ff0` | RecordCopyAssign_00f74ff0 | speculative | Thiscall copy-assign of a response record. IString assign (006f9e60) for source +0x00, +0x10 and +0x20; byte +0x30 copied; the array count at this+0x3c is clea… |
| `00f75060` | RecordCopyAssign_00f75060 | speculative | Thiscall copy-assign of a two-string response record, same body as 00f74f30: IString assign for source +0x00 and +0x10; array count at this+0x28 cleared with 0… |
| `00f774d0` | RecordCopyAssign_00f774d0 | speculative | Thiscall copy-assign of a preview-order record (source at the stack argument). IString assign (006f9e60) for source +0x00 through +0x80 in 0x10 steps; array at… |
| `00f78590` | RecordCopyAssign_00f78590 | speculative | Thiscall copy-assign of an account-payment record. IString assign (006f9e60) for source +0x00, +0x10, +0x24 and +0x34; byte +0x20 copied; the Card array at thi… |
| `00f78630` | RecordCopyAssign_00f78630 | speculative | Thiscall copy-assign of a redeem-code record. IString assign (006f9e60) for source +0x00, +0x10, +0x20 and +0x30; dword +0x40 copied; six arrays with counts at… |
| `00f799e0` | SetShortTextAt0xC0 | speculative | Copies at most 7 characters from its argument into the 8-byte buffer at this+0xc0 with _strncpy_s (00daf0a6, size 8) and writes a zero byte at this+0xc7. |
| `00f79d00` | SetShortTextAt0x140 | speculative | Copies at most 7 characters from its argument into the 8-byte buffer at this+0x140 with _strncpy_s (00daf0a6, size 8) and writes a zero byte at this+0x147. Sam… |
| `00f7a520` | SetShortTextAt0xB0 | speculative | Copies at most 7 characters from its argument into the 8-byte buffer at this+0xb0 with _strncpy_s (00daf0a6, size 8) and writes a zero byte at this+0xb7. |
| `00f7abf0` | SetShortTextAt0x140_Copy | speculative | Byte-identical copy of 00f79d00: copies at most 7 characters from its argument into the 8-byte buffer at this+0x140 with _strncpy_s (00daf0a6, size 8) and writ… |
| `00f842f0` | TempTextWrap_00f84240 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f84610` | TempTextWrap_00f84540 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f84740` | TempTextWrap_00f846d0 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f84890` | TempTextWrap_00f84800 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f84d50` | TempTextWrap_00f84c60 | speculative | Function with no arguments and a plain return, installing an SEH frame and a stack cookie. Builds a temporary SoeUtil::IString, calls 00f84c60 with the tempora… |
| `00f85040` | TempTextWrap_00f84fb0 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f85160` | TempTextWrap_00f85100 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f85280` | TempTextWrap_00f85220 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f853c0` | TempTextWrap_00f85340 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f854e0` | TempTextWrap_00f85480 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f86300` | TempTextWrap_00f86230 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f863c0` | RecordWithErrorArrayCtor | speculative | Fastcall constructor with the record in ECX. Sets nine IString members to the empty state (vtable SoeUtil::IString<char> at +0x00, +0x10, +0x20, +0x30, +0x40, … |
| `00f86780` | WideRecordCtor | speculative | Fastcall constructor with the record in ECX. Sets eleven IString members to the empty state (vtable SoeUtil::IString<char> at +0x00, +0x10, +0x20, +0x38, +0x48… |
| `00f87140` | TempTextWrap_00f86fe0 | speculative | Function with no arguments and a plain return, installing an SEH frame and a stack cookie. Builds a temporary SoeUtil::IString, calls 00f86fe0 with the tempora… |
| `00f878b0` | TempTextWrap_00f877b0 | speculative | Thiscall with no stack arguments and a plain return, installing an SEH frame and a stack cookie. Saves ECX in ESI, builds a temporary SoeUtil::IString, calls 0… |
| `00f87df0` | ResetStringsAndPopArray20 | speculative | Resets a Uram response subobject to empty without freeing this: for the IString-shaped members at this+0x00 and this+0x10 it releases a refcounted buffer (head… |
| `00f87f70` | ResetThreeStringsAndErrorArray | speculative | Resets the same layout as 00f87e70 without destroying it: releases the buffers of the IString-shaped members at this+0x00, +0x10 and +0x20 and sets each to the… |
| `00f880f0` | TwoStringsAndArrayReset | speculative | Resets the two-string record: releases the IString buffers at this+0x00 and this+0x10 (refcounted, freed through the vtable slot 2 on last reference), sets eac… |
| `00f8af90` | TwoStringsAndErrorArrayCtor | speculative | Constructor for the two-string record with an error array: IString-shaped members at this+0x00 and +0x10 set to the empty sentinel 0x01cb4080 with length and c… |
| `00f82be0` | — | unknown | Walks the child list of the stack-argument node (head +0x18, next +0x28) and compares each child name with 00401940 against the 35-entry key table at 0x01b9743… |
| `00f82e10` | — | unknown | Walks the child list of the stack-argument node (head +0x18, next +0x28). For each child whose value (vtable +0x14) is non-null, compares the child name with 0… |
| `00f86630` | — | unknown | Fastcall with the record in ECX (kept as param_1). For each of the IString members at +0x00, +0x10, +0x20, +0x30, +0x40 and +0x50 it releases a refcounted buff… |
| `00f86e00` | — | unknown | Fastcall destructor body with the record in ECX (kept as param_1). Sets the SoeUtil::Array<Uram::Error,0,1> vtable at +0x30, calls 00f63010 with +0x38 pushed, … |
| `00f88320` | — | unknown | Walks the child list of the element passed as this (list head at +0x18, next sibling at +0x28). For each child it calls the child vtable slot 0x14 to get a nam… |
| `00f884c0` | — | unknown | Walks the child list of the element passed as the second argument (head +0x18, next +0x28) and matches each child name through 00401940 against a 20-entry key … |
| `00f89730` | — | unknown | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against a 35-entry key table at 0x01b… |
| `00f89860` | — | unknown | Clears a large record without destroying it: releases refcounted buffers of nine IString-shaped members at this+0x04, +0x14, ..., +0x84 (each set to empty sent… |
| `00f89b10` | — | unknown | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against a 16-entry key table at 0x01b… |
| `00f89cf0` | — | unknown | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against one key string at DAT_01b9762… |
| `00f8a250` | — | unknown | Walks the child list of the element passed as this (head +0x18, next +0x28). Each child name is compared through 00401940 against an 8-entry key table at 0x01b… |

### `net.userdata`

**UserDataCommand** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01528920` | UserDataCommand::UserDataCommand | confirmed | Base constructor: runs 0140c090, stores the UserDataCommand vftable and sets this+4 to 0; calls 012d46a0 and, when it returns nonzero, calls 012d08e0; when tha… |
| `015289d0` | UserDataCommand::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand'. |
| `01528a10` | UserDataCommand::DestructorBody | confirmed | Stores the UserDataCommand vftable into *this and calls 0140c160 (a helper outside this shard, not read here); returns nothing. |
| `01528b20` | UserDataCommand::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand (00d8d382); on failure reports the clone assertion via 012f5a60; calls 0140c28… |
| `01528b70` | UserDataCommand::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], then one integer into this+4 (013c24d0), then the leaf body through this->vtable[5]. Returns the succ… |
| `01528bb0` | UserDataCommand::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), appends the integer at this+4 with 013c23f0, then calls this->vtable[3] (body write). |
| `01528c60` | UserDataCommand::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the UserDataCommand vftable, calls 0140c160, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01528cc0` | UserDataCommand::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command' with a newline via 01241650, calls stub 0140c290, then formats 'AccountID: %d' from this+4 with sprint… |
| `01528d50` | UserDataCommand::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument into ECX and appends the 13-character literal 'Event Command' with 01241650; returns with ret 4. |
| `01528ab0` | UserDataCommand::Clone | likely | Virtual slot 17: allocates 8 bytes with 00d833a6; when non-null constructs the object with 01528920; then calls this->vtable[0x48/4] (CopyFieldsTo) with the ne… |
| `01528be0` | UserDataCommand::Create | likely | Allocates 8 bytes with 00d833a6; when non-null constructs the object with 01528920 and returns it, otherwise returns 0. |
| `015289e0` | UserDataCommand::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec1c. |
| `015289f0` | UserDataCommand::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec1c. |
| `01528a80` | UserDataCommand::GetKeyTriple | speculative | Virtual slot 21 (thiscall, ret 0xc): writes 0x3b9aca07 to the first out-pointer, the dword at this+4 to the second, and 0 to the third. |

**UserDataCommand_DataUpdated** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01527800` | UserDataCommand_DataUpdated::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends the integers at this+0x14 and this+0x18 with 013c23f0, appends the diff … |
| `01527a90` | UserDataCommand_DataUpdated::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command DataUpdated' with a newline via 01241650, calls stub 0140c290, then formats 'mAccountID: %d' (this+4), … |
| `01527b40` | UserDataCommand_DataUpdated::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 28-character literal 'UserData Command DataUpdated' with 01241650; returns… |
| `01527be0` | UserDataCommand_DataUpdated::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_DataUpdated'. |
| `01527c20` | UserDataCommand_DataUpdated::UserDataCommand_DataUpdated | confirmed | Constructor body: runs base constructor 01528920, stores the DataUpdated vftable, calls 01300450, zeroes this+0x14 and this+0x18, releases the old tree through… |
| `01527cb0` | UserDataCommand_DataUpdated::ScalarDeletingDtor | confirmed | Virtual slot 0: calls the destructor body 01527b60, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01527ec0` | UserDataCommand_DataUpdated::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_DataUpdated (00d8d382); on failure reports the clone assertion via 012f5a60; c… |
| `01527f20` | UserDataCommand_DataUpdated::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, two integers into this+0x14 and this+0x18 (013c24d0), the diff map into this+… |
| `01527b60` | UserDataCommand_DataUpdated::DestructorBody | likely | Stores the DataUpdated vftable into *this, calls 012b58e0 (diff-map teardown; the receiver is not shown by Ghidra, the cookie argument is a stack-cookie artifa… |
| `01527cd0` | UserDataCommand_DataUpdated::AssignDiffMapFrom | likely | Thiscall with a source diff map argument: when the source is null reports the 'diffMap' assertion through 012f5a60; then calls 013c2210, 012fc750 with the sour… |
| `01527e50` | UserDataCommand_DataUpdated::Clone | likely | Virtual slot 17: allocates 0x1c bytes with 00d833a6; when non-null constructs the object with 01527c20; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01527fa0` | UserDataCommand_DataUpdated::Create | likely | Allocates 0x1c bytes with 00d833a6; when non-null constructs the object with 01527c20 and returns it, otherwise returns 0. |
| `01527bf0` | UserDataCommand_DataUpdated::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec10. |
| `01527c00` | UserDataCommand_DataUpdated::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec10. |

**UserDataCommand_RequestAdvertisement** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01527380` | UserDataCommand_RequestAdvertisement::UserDataCommand_Reque… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestAdvertisement vftable and zeroes this+8. |
| `01527400` | UserDataCommand_RequestAdvertisement::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_RequestAdvertisement'. |
| `01527440` | UserDataCommand_RequestAdvertisement::DestructorBody | confirmed | Stores the RequestAdvertisement vftable into *this, then calls the base destructor body 01528a10. |
| `01527530` | UserDataCommand_RequestAdvertisement::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_RequestAdvertisement (00d8d382); on failure reports the clone assertion via 01… |
| `01527580` | UserDataCommand_RequestAdvertisement::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, one integer into this+8 (013c24d0), then the leaf body through this->vtable[5… |
| `015275d0` | UserDataCommand_RequestAdvertisement::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends the integer at this+8 with 013c23f0, then calls this->vtable[3] (body wr… |
| `01527690` | UserDataCommand_RequestAdvertisement::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the RequestAdvertisement vftable, calls base destructor body 01528a10, then calls _free(this) only when bit 0 of the argument is set. |
| `015276f0` | UserDataCommand_RequestAdvertisement::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command Request Advertisement' with a newline via 01241650, calls stub 0140c290, then formats 'mLanguageID: %d'… |
| `01527780` | UserDataCommand_RequestAdvertisement::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 38-character literal 'UserData Command Request Advertisement' with 0124165… |
| `015274c0` | UserDataCommand_RequestAdvertisement::Clone | likely | Virtual slot 17: allocates 0x0c bytes with 00d833a6; when non-null constructs the object with 01527380; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01527610` | UserDataCommand_RequestAdvertisement::Create | likely | Allocates 0x0c bytes with 00d833a6; when non-null constructs the object with 01527380 and returns it, otherwise returns 0. |
| `01527410` | UserDataCommand_RequestAdvertisement::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec0c. |
| `01527420` | UserDataCommand_RequestAdvertisement::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec0c. |
| `01527490` | UserDataCommand_RequestAdvertisement::GetDword08 | speculative | Virtual slot 22: returns the dword at this+8. |
| `015274a0` | UserDataCommand_RequestAdvertisement::SetDword08 | speculative | Virtual slot 23: stores its stack argument at this+8. |

**UserDataCommand_RequestDBAvatars** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151c580` | UserDataCommand_RequestDBAvatars::UserDataCommand_RequestDB… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestDBAvatars vtable at +0 and zeroes the int at +0x8. |
| `0151c600` | UserDataCommand_RequestDBAvatars::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestDBAvatars. |
| `0151c640` | UserDataCommand_RequestDBAvatars::~UserDataCommand_RequestD… | confirmed | Destructor body without free: stores the RequestDBAvatars vtable into this and calls base destructor body 01528a10. |
| `0151c690` | UserDataCommand_RequestDBAvatars::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151c6a0` | UserDataCommand_RequestDBAvatars::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151c6c0` | UserDataCommand_RequestDBAvatars::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151c580 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151c730` | UserDataCommand_RequestDBAvatars::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestDBAvatars via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), base… |
| `0151c780` | UserDataCommand_RequestDBAvatars::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151c7d0` | UserDataCommand_RequestDBAvatars::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151c890` | UserDataCommand_RequestDBAvatars::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestDBAvatars vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argument… |
| `0151c8f0` | UserDataCommand_RequestDBAvatars::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request DB Avatars and a newline, calls stub 0140c290, then sprintf of the int at +0x8 as mLanguageID appended… |
| `0151c980` | UserDataCommand_RequestDBAvatars::DisplayName | confirmed | Virtual slot [20] display name: appends the 35-character text UserData Command Request DB Avatars to the std::string argument via 01241650. |
| `0151c610` | UserDataCommand_RequestDBAvatars::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebd4. |
| `0151c620` | UserDataCommand_RequestDBAvatars::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebd4. |
| `0151c810` | UserDataCommand_RequestDBAvatars::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151c580 and returns the object, otherwise 0. |
| `0151c870` | UserDataCommand_RequestDBAvatars::RegisterFactory | likely | Pushes factory 0151c810 and key 0x229 into the registry: calls 013ce130(0x229, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestDBAwards** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151c160` | UserDataCommand_RequestDBAwards::UserDataCommand_RequestDBA… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestDBAwards vtable at +0 and zeroes the int at +0x8. |
| `0151c1e0` | UserDataCommand_RequestDBAwards::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestDBAwards. |
| `0151c220` | UserDataCommand_RequestDBAwards::~UserDataCommand_RequestDB… | confirmed | Destructor body without free: stores the RequestDBAwards vtable into this and calls base destructor body 01528a10. |
| `0151c270` | UserDataCommand_RequestDBAwards::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151c280` | UserDataCommand_RequestDBAwards::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151c2a0` | UserDataCommand_RequestDBAwards::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151c160 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151c310` | UserDataCommand_RequestDBAwards::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestDBAwards via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), base … |
| `0151c360` | UserDataCommand_RequestDBAwards::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151c3b0` | UserDataCommand_RequestDBAwards::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151c470` | UserDataCommand_RequestDBAwards::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestDBAwards vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argument … |
| `0151c4d0` | UserDataCommand_RequestDBAwards::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request DB Awards and a newline, calls stub 0140c290, then sprintf of the int at +0x8 as mLanguageID appended … |
| `0151c560` | UserDataCommand_RequestDBAwards::DisplayName | confirmed | Virtual slot [20] display name: appends the 34-character text UserData Command Request DB Awards to the std::string argument via 01241650. |
| `0151c1f0` | UserDataCommand_RequestDBAwards::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebd0. |
| `0151c200` | UserDataCommand_RequestDBAwards::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebd0. |
| `0151c3f0` | UserDataCommand_RequestDBAwards::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151c160 and returns the object, otherwise 0. |
| `0151c450` | UserDataCommand_RequestDBAwards::RegisterFactory | likely | Pushes factory 0151c3f0 and key 0x22a into the registry: calls 013ce130(0x22a, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestDBHonors** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151bd40` | UserDataCommand_RequestDBHonors::UserDataCommand_RequestDBH… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestDBHonors vtable at +0 and zeroes the int at +0x8. |
| `0151bdc0` | UserDataCommand_RequestDBHonors::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestDBHonors. |
| `0151be00` | UserDataCommand_RequestDBHonors::~UserDataCommand_RequestDB… | confirmed | Destructor body without free: stores the RequestDBHonors vtable into this and calls base destructor body 01528a10. |
| `0151be50` | UserDataCommand_RequestDBHonors::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151be60` | UserDataCommand_RequestDBHonors::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151be80` | UserDataCommand_RequestDBHonors::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151bd40 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151bef0` | UserDataCommand_RequestDBHonors::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestDBHonors via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), base … |
| `0151bf40` | UserDataCommand_RequestDBHonors::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151bf90` | UserDataCommand_RequestDBHonors::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151c050` | UserDataCommand_RequestDBHonors::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestDBHonors vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argument … |
| `0151c0b0` | UserDataCommand_RequestDBHonors::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request DB Honors and a newline, calls stub 0140c290, then sprintf of the int at +0x8 as mLanguageID and appen… |
| `0151c140` | UserDataCommand_RequestDBHonors::DisplayName | confirmed | Virtual slot [20] display name: appends the 34-character text UserData Command Request DB Honors to the std::string argument via 01241650. |
| `0151bdd0` | UserDataCommand_RequestDBHonors::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebcc. |
| `0151bde0` | UserDataCommand_RequestDBHonors::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebcc. |
| `0151bfd0` | UserDataCommand_RequestDBHonors::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151bd40 and returns the object, otherwise 0. |
| `0151c030` | UserDataCommand_RequestDBHonors::RegisterFactory | likely | Pushes factory 0151bfd0 and key 0x22b into the registry: calls 013ce130(0x22b, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestFeatureRotator** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015264d0` | UserDataCommand_RequestFeatureRotator::UserDataCommand_Requ… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestFeatureRotator vftable and zeroes this+8. |
| `01526550` | UserDataCommand_RequestFeatureRotator::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_RequestFeatureRotator'. |
| `01526590` | UserDataCommand_RequestFeatureRotator::DestructorBody | confirmed | Stores the RequestFeatureRotator vftable into *this, then calls the base destructor body 01528a10. |
| `01526680` | UserDataCommand_RequestFeatureRotator::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_RequestFeatureRotator (00d8d382); on failure reports the clone assertion via 0… |
| `015266d0` | UserDataCommand_RequestFeatureRotator::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, one integer into this+8 (013c24d0), then the leaf body through this->vtable[5… |
| `01526720` | UserDataCommand_RequestFeatureRotator::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends the integer at this+8 with 013c23f0, then calls this->vtable[3] (body wr… |
| `015267e0` | UserDataCommand_RequestFeatureRotator::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the RequestFeatureRotator vftable, calls base destructor body 01528a10, then calls _free(this) only when bit 0 of the argument is set. |
| `01526840` | UserDataCommand_RequestFeatureRotator::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command Request Feature Rotator' with 01241650, calls stub 0140c290, then formats 'mLanguageID: %d' from this+8… |
| `015268d0` | UserDataCommand_RequestFeatureRotator::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument into ECX and appends the 40-character literal 'UserData Command Request Feature Rotator' with 01241650; ret 4. |
| `01526610` | UserDataCommand_RequestFeatureRotator::Clone | likely | Virtual slot 17: allocates 0x0c bytes with 00d833a6; when non-null constructs the object with 015264d0; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01526760` | UserDataCommand_RequestFeatureRotator::Create | likely | Allocates 0x0c bytes with 00d833a6; when non-null constructs the object with 015264d0 and returns it, otherwise returns 0. |
| `01526560` | UserDataCommand_RequestFeatureRotator::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec04. |
| `01526570` | UserDataCommand_RequestFeatureRotator::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec04. |
| `015265e0` | UserDataCommand_RequestFeatureRotator::GetDword08 | speculative | Virtual slot 22: returns the dword at this+8. |
| `015265f0` | UserDataCommand_RequestFeatureRotator::SetDword08 | speculative | Virtual slot 23: stores its stack argument at this+8. |

**UserDataCommand_RequestNewsMessage** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015253d0` | UserDataCommand_RequestNewsMessage::DestructorBody | confirmed | Stores the UserDataCommand_RequestNewsMessage vftable into *this, then calls the base destructor body 01528a10 (which resets the vtable to UserDataCommand and … |
| `015254c0` | UserDataCommand_RequestNewsMessage::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_RequestNewsMessage (00d8d382); on failure reports the clone assertion through … |
| `01525510` | UserDataCommand_RequestNewsMessage::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], then the base body 01528b70, then one integer into this+8 (013c24d0), then the leaf body through this… |
| `01525560` | UserDataCommand_RequestNewsMessage::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write, 013aede0), the base body 01528bb0, appends the integer at this+8 with 013c23f0, then calls this->vtable[3… |
| `01525620` | UserDataCommand_RequestNewsMessage::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the RequestNewsMessage vftable, calls base destructor body 01528a10, then calls _free(this) only when bit 0 of the argument is set; retu… |
| `01525680` | UserDataCommand_RequestNewsMessage::Dump | confirmed | Virtual slot 19 (debug dump): appends the text 'UserData Command Request News Message' with 01241650, calls stub 0140c290, then formats 'mLanguageID: %d' from … |
| `01525710` | UserDataCommand_RequestNewsMessage::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 37-character literal 'UserData Command Request News Message' with 01241650… |
| `01525450` | UserDataCommand_RequestNewsMessage::Clone | likely | Virtual slot 17: allocates 0x0c bytes with 00d833a6; when the allocation is non-null it constructs the command with 01525310. It then calls this->vtable[0x48/4… |
| `015255a0` | UserDataCommand_RequestNewsMessage::Create | likely | Allocates 0x0c bytes with 00d833a6; when the allocation is non-null it constructs the command with 01525310 and returns that pointer, otherwise returns 0. |
| `015253a0` | UserDataCommand_RequestNewsMessage::GetGlobalDword | speculative | Virtual slot 12 of UserDataCommand_RequestNewsMessage: returns the 32-bit global DAT_01cbebfc (mov eax,[DAT_01cbebfc]); takes no arguments. |
| `015253b0` | UserDataCommand_RequestNewsMessage::SetGlobalDword | speculative | Virtual slot 13 of UserDataCommand_RequestNewsMessage: stores its stack argument into the global DAT_01cbebfc and returns. |
| `01525420` | UserDataCommand_RequestNewsMessage::GetDword08 | speculative | Virtual slot 22 of UserDataCommand_RequestNewsMessage: returns the dword at this+8. |
| `01525430` | UserDataCommand_RequestNewsMessage::SetDword08 | speculative | Virtual slot 23 of UserDataCommand_RequestNewsMessage: stores its stack argument at this+8. |

**UserDataCommand_RequestPlayerAvatars** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151b920` | UserDataCommand_RequestPlayerAvatars::UserDataCommand_Reque… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestPlayerAvatars vtable at +0 and zeroes the int at +0x8. |
| `0151b9a0` | UserDataCommand_RequestPlayerAvatars::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestPlayerAvatars. |
| `0151b9e0` | UserDataCommand_RequestPlayerAvatars::~UserDataCommand_Requ… | confirmed | Destructor body without free: stores the RequestPlayerAvatars vtable into this and calls base destructor body 01528a10. |
| `0151ba30` | UserDataCommand_RequestPlayerAvatars::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151ba40` | UserDataCommand_RequestPlayerAvatars::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151ba60` | UserDataCommand_RequestPlayerAvatars::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151b920 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151bad0` | UserDataCommand_RequestPlayerAvatars::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestPlayerAvatars via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), … |
| `0151bb20` | UserDataCommand_RequestPlayerAvatars::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151bb70` | UserDataCommand_RequestPlayerAvatars::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151bc30` | UserDataCommand_RequestPlayerAvatars::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestPlayerAvatars vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argu… |
| `0151bc90` | UserDataCommand_RequestPlayerAvatars::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request Player Avatars and a newline, calls stub 0140c290, then sprintf of mRequestAccountID from +0x8 appende… |
| `0151bd20` | UserDataCommand_RequestPlayerAvatars::DisplayName | confirmed | Virtual slot [20] display name: appends the 39-character text UserData Command Request Player Avatars to the std::string argument via 01241650. |
| `0151b9b0` | UserDataCommand_RequestPlayerAvatars::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebc8. |
| `0151b9c0` | UserDataCommand_RequestPlayerAvatars::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebc8. |
| `0151bbb0` | UserDataCommand_RequestPlayerAvatars::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151b920 and returns the object, otherwise 0. |
| `0151bc10` | UserDataCommand_RequestPlayerAvatars::RegisterFactory | likely | Pushes factory 0151bbb0 and key 0x22c into the registry: calls 013ce130(0x22c, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestPlayerAwards** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151b500` | UserDataCommand_RequestPlayerAwards::UserDataCommand_Reques… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestPlayerAwards vtable at +0 and zeroes the int at +0x8. |
| `0151b580` | UserDataCommand_RequestPlayerAwards::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestPlayerAwards. |
| `0151b5c0` | UserDataCommand_RequestPlayerAwards::~UserDataCommand_Reque… | confirmed | Destructor body without free: stores the RequestPlayerAwards vtable into this and calls base destructor body 01528a10. |
| `0151b610` | UserDataCommand_RequestPlayerAwards::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151b620` | UserDataCommand_RequestPlayerAwards::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151b640` | UserDataCommand_RequestPlayerAwards::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151b500 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151b6b0` | UserDataCommand_RequestPlayerAwards::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestPlayerAwards via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), b… |
| `0151b700` | UserDataCommand_RequestPlayerAwards::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151b750` | UserDataCommand_RequestPlayerAwards::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151b810` | UserDataCommand_RequestPlayerAwards::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestPlayerAwards vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argum… |
| `0151b870` | UserDataCommand_RequestPlayerAwards::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request Player Awards and a newline, calls stub 0140c290, then sprintf of mRequestAccountID from +0x8 appended… |
| `0151b900` | UserDataCommand_RequestPlayerAwards::DisplayName | confirmed | Virtual slot [20] display name: appends the 38-character text UserData Command Request Player Awards to the std::string argument via 01241650. |
| `0151b590` | UserDataCommand_RequestPlayerAwards::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebc4. |
| `0151b5a0` | UserDataCommand_RequestPlayerAwards::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebc4. |
| `0151b790` | UserDataCommand_RequestPlayerAwards::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151b500 and returns the object, otherwise 0. |
| `0151b7f0` | UserDataCommand_RequestPlayerAwards::RegisterFactory | likely | Pushes factory 0151b790 and key 0x22d into the registry: calls 013ce130(0x22d, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestPlayerHonors** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151b0e0` | UserDataCommand_RequestPlayerHonors::UserDataCommand_Reques… | confirmed | Constructor body: runs base constructor 01528920, stores the RequestPlayerHonors vtable at +0 and zeroes the int at +0x8. |
| `0151b160` | UserDataCommand_RequestPlayerHonors::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RequestPlayerHonors. |
| `0151b1a0` | UserDataCommand_RequestPlayerHonors::~UserDataCommand_Reque… | confirmed | Destructor body without free: stores the RequestPlayerHonors vtable into this and calls base destructor body 01528a10. |
| `0151b1f0` | UserDataCommand_RequestPlayerHonors::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8. |
| `0151b200` | UserDataCommand_RequestPlayerHonors::SetDword08 | confirmed | Virtual slot [23] setter: stores its argument into this+0x8 and returns with ret 4. |
| `0151b220` | UserDataCommand_RequestPlayerHonors::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; on success constructs with 0151b0e0 and returns it, else 0; then calls source slot [18] (+0x48) with t… |
| `0151b290` | UserDataCommand_RequestPlayerHonors::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RequestPlayerHonors via 00d8d382 (assert clone via 012f5a60, line 0x2d on failure), b… |
| `0151b2e0` | UserDataCommand_RequestPlayerHonors::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151b330` | UserDataCommand_RequestPlayerHonors::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151b3f0` | UserDataCommand_RequestPlayerHonors::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestPlayerHonors vtable, calls base destructor 01528a10, and frees this only when bit 0 of the argum… |
| `0151b450` | UserDataCommand_RequestPlayerHonors::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Request Player Honors and a newline, calls stub 0140c290, then sprintf of mRequestAccountID from +0x8 appended… |
| `0151b4e0` | UserDataCommand_RequestPlayerHonors::DisplayName | confirmed | Virtual slot [20] display name: appends the 38-character text UserData Command Request Player Honors to the std::string argument via 01241650. |
| `0151b170` | UserDataCommand_RequestPlayerHonors::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebc0. |
| `0151b180` | UserDataCommand_RequestPlayerHonors::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebc0. |
| `0151b370` | UserDataCommand_RequestPlayerHonors::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151b0e0 and returns the object, otherwise 0. |
| `0151b3d0` | UserDataCommand_RequestPlayerHonors::RegisterFactory | likely | Pushes factory 0151b370 and key 0x22e into the registry: calls 013ce130(0x22e, factory), moves the result into ECX and calls 013ce230. |

**UserDataCommand_RequestPlayerInfo** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151acc0` | UserDataCommand_RequestPlayerInfo::UserDataCommand_RequestP… | confirmed | Constructor body: runs base constructor 01528920 (UserDataCommand base), stores the RequestPlayerInfo vtable at +0 and zeroes the int at +0x8. |
| `0151ad40` | UserDataCommand_RequestPlayerInfo::TypeName | confirmed | Virtual slot [11] returns the literal class name UserDataCommand_RequestPlayerInfo. |
| `0151ad80` | UserDataCommand_RequestPlayerInfo::~UserDataCommand_Request… | confirmed | Destructor body without free: stores the RequestPlayerInfo vtable into this, keeps a local pointer for the SEH frame, and calls base destructor body 01528a10 o… |
| `0151add0` | UserDataCommand_RequestPlayerInfo::GetDword08 | confirmed | Virtual slot [22] getter: returns the dword at this+0x8 (mov eax,[ecx+8]; ret). |
| `0151ade0` | UserDataCommand_RequestPlayerInfo::SetDword08 | confirmed | Virtual slot [23] setter: stores its stack argument into this+0x8 and returns with ret 4. |
| `0151ae00` | UserDataCommand_RequestPlayerInfo::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; if non-null it constructs with 0151acc0 and returns the object, else 0; then calls source vtable slot … |
| `0151ae70` | UserDataCommand_RequestPlayerInfo::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast of dest from NetworkCommand to RequestPlayerInfo via 00d8d382, asserting clone through 012f5a60 (line 0x2d) … |
| `0151aec0` | UserDataCommand_RequestPlayerInfo::Read | confirmed | Virtual slot [9] Read: header read via vtable +0x10 (slot 4), base UserDataCommand::Read 01528b70, one int into this+0x8 via 013c24d0, then body-end read via v… |
| `0151af10` | UserDataCommand_RequestPlayerInfo::Write | confirmed | Virtual slot [10] Write: header write via vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, SerialAppendInteger of this+0x8 via 013c23f0, then body-e… |
| `0151afd0` | UserDataCommand_RequestPlayerInfo::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestPlayerInfo vtable, calls base destructor body 01528a10, and calls _free(this) only when bit 0 of… |
| `0151b030` | UserDataCommand_RequestPlayerInfo::Dump | confirmed | Virtual slot [19] Dump: appends the text UserData Command Request Player Info and a newline, calls stub 0140c290 with the sink, then sprintf of mRequestAccount… |
| `0151b0c0` | UserDataCommand_RequestPlayerInfo::DisplayName | confirmed | Virtual slot [20] display name: appends the 36-character text UserData Command Request Player Info to the std::string argument via 01241650. |
| `0151ad50` | UserDataCommand_RequestPlayerInfo::GetStaticDword | likely | Virtual slot [12] returns the 32-bit class-wide static at DAT_01cbebbc. |
| `0151ad60` | UserDataCommand_RequestPlayerInfo::SetStaticDword | likely | Virtual slot [13] stores its stack argument into the class-wide static DAT_01cbebbc and returns with ret 4. |
| `0151af50` | UserDataCommand_RequestPlayerInfo::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151acc0 and returns the object, otherwise returns 0. |
| `0151afb0` | UserDataCommand_RequestPlayerInfo::RegisterFactory | likely | Pushes factory 0151af50 and key 0x22f into the registry: calls 013ce130(0x22f, factory), moves the returned object into ECX and calls 013ce230 on it. |

**UserDataCommand_RequestPlayerMatches** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151a9e0` | UserDataCommand_RequestPlayerMatches::Clone | confirmed | Virtual slot [17] Clone: operator new(0xc) via 00d833a6; if the pointer is non-null it constructs the object with 0151a8a0 and returns it, otherwise returns 0.… |
| `0151aa50` | UserDataCommand_RequestPlayerMatches::CopyFieldsTo | confirmed | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast of dest from NetworkCommand to RequestPlayerMatches via 00d8d382; when the cast fails it reports the clone a… |
| `0151aaa0` | UserDataCommand_RequestPlayerMatches::Read | confirmed | Virtual slot [9] Read(stream): calls own header read at vtable +0x10 (slot 4) and stops with 0 if it fails; calls base UserDataCommand::Read 01528b70; reads on… |
| `0151aaf0` | UserDataCommand_RequestPlayerMatches::Write | confirmed | Virtual slot [10] Write(stream): calls own header write at vtable +0x8 (slot 2), base UserDataCommand::Write 01528bb0, appends the int at this+0x8 via SerialAp… |
| `0151abb0` | UserDataCommand_RequestPlayerMatches::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: stores the RequestPlayerMatches vtable into this, calls base destructor body 01528a10, and calls _free(this) only … |
| `0151ac10` | UserDataCommand_RequestPlayerMatches::Dump | confirmed | Virtual slot [19] Dump: appends the text UserData Command Request Player Matches and a newline via 01241650, calls the no-op 0140c290 with the trace sink, then… |
| `0151aca0` | UserDataCommand_RequestPlayerMatches::DisplayName | confirmed | Virtual slot [20] display name: appends the 39-character text UserData Command Request Player Matches to the std::string passed in as its argument (01241650 ap… |
| `0151ab30` | UserDataCommand_RequestPlayerMatches::Factory | likely | Allocates 0xc bytes with operator new 00d833a6 under an SEH frame; if the allocation is non-null it constructs the object with 0151a8a0 and returns it, otherwi… |
| `0151ab90` | UserDataCommand_RequestPlayerMatches::RegisterFactory | likely | Pushes the factory address 0151ab30 and key 0x230 into the registry: calls 013ce130(0x230, factory), moves the returned object into ECX and calls 013ce230 on i… |

**UserDataCommand_RespondAdvertisement** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015269a0` | UserDataCommand_RespondAdvertisement::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends four Array fields at this+8, +0x18, +0x28, +0x38 with 012fcdc0, then cal… |
| `01526a00` | UserDataCommand_RespondAdvertisement::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command Respond Advertisement' with a newline; if the four Array fields have equal counts it prints per index m… |
| `01526ec0` | UserDataCommand_RespondAdvertisement::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 38-character literal 'UserData Command Respond Advertisement' with 0124165… |
| `01526ee0` | UserDataCommand_RespondAdvertisement::UserDataCommand_Respo… | confirmed | Constructor body: runs base constructor 01528920, stores the RespondAdvertisement vftable, and zeroes the four Array fields (ends at +0x44). |
| `01526f90` | UserDataCommand_RespondAdvertisement::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_RespondAdvertisement'. |
| `01526fd0` | UserDataCommand_RespondAdvertisement::DestructorBody | confirmed | Stores the RespondAdvertisement vftable, frees the four Array fields (each via 012327d0 then _free, from +0x38 down to +8), then calls the base destructor body… |
| `015271f0` | UserDataCommand_RespondAdvertisement::ScalarDeletingDtor | confirmed | Virtual slot 0: calls the destructor body 01526fd0, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01527210` | UserDataCommand_RespondAdvertisement::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_RespondAdvertisement (00d8d382); on failure reports the clone assertion via 01… |
| `015272d0` | UserDataCommand_RespondAdvertisement::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, then four Array fields into this+8, +0x18, +0x28, +0x38 with 01301640 (each p… |
| `015268f0` | UserDataCommand_RespondAdvertisement::QueueClientEvent | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xef into it with 012f8c70; calls 012fa… |
| `01527100` | UserDataCommand_RespondAdvertisement::Clone | likely | Virtual slot 17: allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01526ee0; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01527170` | UserDataCommand_RespondAdvertisement::Create | likely | Allocates 0x48 bytes with 00d833a6; when non-null constructs the object with 01526ee0 and returns it, otherwise returns 0. |
| `01526fa0` | UserDataCommand_RespondAdvertisement::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec08. |
| `01526fb0` | UserDataCommand_RespondAdvertisement::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec08. |

**UserDataCommand_RespondFeatureRotator** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01525800` | UserDataCommand_RespondFeatureRotator::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, then appends seven Array fields at this+8, +0x18, +0x28, +0x38, +0x48, +0x58, +0… |
| `01525890` | UserDataCommand_RespondFeatureRotator::Dump | confirmed | Virtual slot 19 (debug dump): appends the header 'UserData Command Respond Feature Rotator'; if the seven Array fields have equal element counts it prints per … |
| `01525ea0` | UserDataCommand_RespondFeatureRotator::AppendDisplayName | confirmed | Virtual slot 20: appends the 40-character literal 'UserData Command Respond Feature Rotator' with 01241650 to the string object passed in on the stack; ret 4. |
| `01525ec0` | UserDataCommand_RespondFeatureRotator::UserDataCommand_Resp… | confirmed | Constructor body: runs base constructor 01528920, stores the RespondFeatureRotator vftable, and zeroes the seven Array fields (ends at +0x74). |
| `01525fa0` | UserDataCommand_RespondFeatureRotator::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_RespondFeatureRotator'. |
| `01525fe0` | UserDataCommand_RespondFeatureRotator::DestructorBody | confirmed | Stores the RespondFeatureRotator vftable, frees the seven Array fields (each via 012327d0 then _free, from +0x68 down to +8), then calls the base destructor bo… |
| `015262a0` | UserDataCommand_RespondFeatureRotator::ScalarDeletingDtor | confirmed | Virtual slot 0: calls the destructor body 01525fe0, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `015262c0` | UserDataCommand_RespondFeatureRotator::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_RespondFeatureRotator (00d8d382); on failure reports the clone assertion via 0… |
| `015263d0` | UserDataCommand_RespondFeatureRotator::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, then seven Array fields into this+8 .. +0x68 with 01301640 (each preceded by … |
| `01525730` | UserDataCommand_RespondFeatureRotator::QueueClientEvent | likely | Virtual slot 15: allocates 0x14 bytes with 00d833a6 and, when non-null, initialises the block with 012f9eb0; writes tag 0xf0 into it with 012f8c70; calls 012fa… |
| `015261b0` | UserDataCommand_RespondFeatureRotator::Clone | likely | Virtual slot 17: allocates 0x78 bytes with 00d833a6; when non-null constructs the object with 01525ec0; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01526220` | UserDataCommand_RespondFeatureRotator::Create | likely | Allocates 0x78 bytes with 00d833a6; when non-null constructs the object with 01525ec0 and returns it, otherwise returns 0. |
| `01525fb0` | UserDataCommand_RespondFeatureRotator::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec00. |
| `01525fc0` | UserDataCommand_RespondFeatureRotator::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec00. |

**UserDataCommand_RespondPlayerMatches** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151d400` | UserDataCommand_RespondPlayerMatches::Dump | confirmed | Virtual slot [19] Dump: appends UserData Command Respond Player Matches and a newline, calls stub 0140c290, then, when the per-match vectors agree in size, pri… |
| `0151de30` | UserDataCommand_RespondPlayerMatches::DisplayName | confirmed | Virtual slot [20] display name: appends the 39-character text UserData Command Respond Player Matches via 01241650. |
| `0151e200` | UserDataCommand_RespondPlayerMatches::UserDataCommand_Respo… | confirmed | Constructor body (size 0xfc): runs base constructor 01528920, stores the RespondPlayerMatches vtable at +0, zeroes the int at +0x8 and the begin/end/capacity w… |
| `0151e390` | UserDataCommand_RespondPlayerMatches::TypeName | confirmed | Virtual slot [11] returns the class name literal UserDataCommand_RespondPlayerMatches. |
| `0151e3d0` | UserDataCommand_RespondPlayerMatches::~UserDataCommand_Resp… | confirmed | Destructor body without free: stores the RespondPlayerMatches vtable, builds two temporary pointer lists with 0151e190 pushes from the nested lists at +0xdc an… |
| `0151e950` | UserDataCommand_RespondPlayerMatches::Clone | confirmed | Virtual slot [17] Clone: operator new(0xfc) via 00d833a6; on success constructs with 0151e200 and returns it, else 0; then calls source slot [18] (+0x48) with … |
| `0151e9c0` | UserDataCommand_RespondPlayerMatches::Write | confirmed | Virtual slot [10] Write(stream): calls own header write (vtable +0x8), base UserDataCommand::Write 01528bb0, then writes the 3 vectors at +0xc, +0x1c, +0x2c wi… |
| `0151edf0` | UserDataCommand_RespondPlayerMatches::ScalarDeletingDtor | confirmed | Virtual slot [0] scalar deleting destructor: calls the destructor body 0151e3d0 on this and calls _free(this) only when bit 0 of the argument is set; returns t… |
| `0151e3a0` | UserDataCommand_RespondPlayerMatches::GetStaticDword | likely | Virtual slot [12] returns the class-wide static dword at DAT_01cbebd8. |
| `0151e3b0` | UserDataCommand_RespondPlayerMatches::SetStaticDword | likely | Virtual slot [13] stores its argument into the class-wide static DAT_01cbebd8. |
| `0151ed60` | UserDataCommand_RespondPlayerMatches::Factory | likely | Allocates 0xfc bytes with operator new 00d833a6 under an SEH frame; on success constructs with 0151e200 and returns the object, otherwise 0. |
| `0151edd0` | UserDataCommand_RespondPlayerMatches::RegisterFactory | likely | Pushes factory 0151ed60 and key 0x238 into the registry: calls 013ce130(0x238, factory), moves the result into ECX and calls 013ce230. |
| `0151ee10` | — | unknown | Virtual slot [15] client handler: for each element of the pointer lists at +0xe0 and +0xf0 it calls the element's vtable slot 29 (+0x74), tests the result with… |
| `0151f2c0` | — | unknown | Virtual slot [18] CopyFieldsTo(dest): dynamic_cast from NetworkCommand to RespondPlayerMatches via 00d8d382 (assert clone via 012f5a60, line 0xb5 on failure), … |

**UserDataCommand_Subscribe** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015284a0` | UserDataCommand_Subscribe::UserDataCommand_Subscribe | confirmed | Constructor body: runs base constructor 01528920, stores the Subscribe vftable and zeroes this+8 and this+0xc. |
| `01528520` | UserDataCommand_Subscribe::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_Subscribe'. |
| `01528560` | UserDataCommand_Subscribe::DestructorBody | confirmed | Stores the Subscribe vftable into *this, then calls the base destructor body 01528a10. |
| `01528670` | UserDataCommand_Subscribe::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_Subscribe (00d8d382); on failure reports the clone assertion via 012f5a60; cal… |
| `015286d0` | UserDataCommand_Subscribe::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, then two integers into this+8 and this+0xc (013c24d0), then the leaf body thr… |
| `01528730` | UserDataCommand_Subscribe::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends the integers at this+8 and this+0xc with 013c23f0, then calls this->vtab… |
| `015287f0` | UserDataCommand_Subscribe::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the Subscribe vftable, calls base destructor body 01528a10, then calls _free(this) only when bit 0 of the argument is set; returns this. |
| `01528850` | UserDataCommand_Subscribe::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command Subscribe' with a newline via 01241650, calls stub 0140c290, then formats 'mAccountID: %d' (this+4), 'm… |
| `01528900` | UserDataCommand_Subscribe::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 26-character literal 'UserData Command Subscribe' with 01241650; returns w… |
| `01528600` | UserDataCommand_Subscribe::Clone | likely | Virtual slot 17: allocates 0x10 bytes with 00d833a6; when non-null constructs the object with 015284a0; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `01528770` | UserDataCommand_Subscribe::Create | likely | Allocates 0x10 bytes with 00d833a6; when non-null constructs the object with 015284a0 and returns it, otherwise returns 0. |
| `01528530` | UserDataCommand_Subscribe::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec18. |
| `01528540` | UserDataCommand_Subscribe::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec18. |

**UserDataCommand_Unsubscribe** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01528020` | UserDataCommand_Unsubscribe::UserDataCommand_Unsubscribe | confirmed | Constructor body: runs base constructor 01528920, stores the Unsubscribe vftable, zeroes this+8 and this+0xc. |
| `015280a0` | UserDataCommand_Unsubscribe::GetName | confirmed | Virtual slot 11: returns the address of the literal 'UserDataCommand_Unsubscribe'. |
| `015280e0` | UserDataCommand_Unsubscribe::DestructorBody | confirmed | Stores the Unsubscribe vftable into *this, then calls the base destructor body 01528a10. |
| `015281f0` | UserDataCommand_Unsubscribe::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target from NetworkCommand to UserDataCommand_Unsubscribe (00d8d382); on failure reports the clone assertion via 012f5a60; c… |
| `01528250` | UserDataCommand_Unsubscribe::Read | confirmed | Virtual slot 9: reads the header through this->vtable[4], the base body 01528b70, then two integers into this+8 and this+0xc (013c24d0), then the leaf body thr… |
| `015282b0` | UserDataCommand_Unsubscribe::Write | confirmed | Virtual slot 10: calls this->vtable[2] (header write), the base body 01528bb0, appends the integers at this+8 and this+0xc with 013c23f0, then calls this->vtab… |
| `01528370` | UserDataCommand_Unsubscribe::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the Unsubscribe vftable, calls base destructor body 01528a10, then calls _free(this) only when bit 0 of the argument is set; returns thi… |
| `015283d0` | UserDataCommand_Unsubscribe::Dump | confirmed | Virtual slot 19 (debug dump): appends 'UserData Command Unsubscribe' with a newline, calls stub 0140c290, then formats 'mAccountID: %d' (this+4), 'mUserAccount… |
| `01528480` | UserDataCommand_Unsubscribe::AppendDisplayName | confirmed | Virtual slot 20: moves its stack argument (a string object) into ECX and appends the 28-character literal 'UserData Command Unsubscribe' with 01241650; returns… |
| `01528180` | UserDataCommand_Unsubscribe::Clone | likely | Virtual slot 17: allocates 0x10 bytes with 00d833a6; when non-null constructs the object with 01528020; then calls this->vtable[0x48/4] (CopyFieldsTo) with the… |
| `015282f0` | UserDataCommand_Unsubscribe::Create | likely | Allocates 0x10 bytes with 00d833a6; when non-null constructs the object with 01528020 and returns it, otherwise returns 0. |
| `015280b0` | UserDataCommand_Unsubscribe::GetSharedDword | speculative | Virtual slot 12: returns the global dword DAT_01cbec14. |
| `015280c0` | UserDataCommand_Unsubscribe::SetSharedDword | speculative | Virtual slot 13: stores its stack argument into the global DAT_01cbec14. |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151f730` | — | unknown | Takes 12 stack arguments (param_2..param_13). First pads the 4-byte lists at +0xe0 and +0xf0 with zero entries (direct store or 014e4a30) until each is at leas… |

### `net.userdata_command`

**UserDataCommand_RequestNewsMessage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01525310` | UserDataCommand_RequestNewsMessage::UserDataCommand_Request… | confirmed | Constructor body: calls FUN_01528920 (base constructor), stores the RequestNewsMessage vtable at [this], zeroes this+0x8 and returns this. |
| `01525390` | UserDataCommand_RequestNewsMessage::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RequestNewsMessage (mov eax, imm32; ret). |

**UserDataCommand_RespondDBAvatars** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015243c0` | UserDataCommand_RespondDBAvatars::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond DB Avatars (35 bytes, literal length 0x23) into the sret buffer via FUN_01241650 and returns. |
| `015243e0` | UserDataCommand_RespondDBAvatars::UserDataCommand_RespondDB… | confirmed | Constructor: calls FUN_01528920 (base constructor), stores the DBAvatars vtable at [this], zeroes its member fields and returns this. |
| `01524490` | UserDataCommand_RespondDBAvatars::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondDBAvatars (mov eax, imm32; ret). |
| `015246d0` | UserDataCommand_RespondDBAvatars::~UserDataCommand_RespondD… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_015244d0 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `015246f0` | UserDataCommand_RespondDBAvatars::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondDBAvatars via FUN_00d8d382; on failure calls FUN_012f5a60 with the clone… |
| `015244d0` | UserDataCommand_RespondDBAvatars::DtorBody | likely | Stores UserDataCommand_RespondDBAvatars::vftable at [this]; frees and zeroes the list buffers (guarded by non-null tests); calls FUN_012327d0 and FUN_01528a10 … |
| `015247b0` | UserDataCommand_RespondDBAvatars::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; reads the list members and scalars with list readers and FUN_013c… |
| `015244a0` | UserDataCommand_RespondDBAvatars::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebf4 (mov eax,[0x01cbebf4]; ret). |
| `015244b0` | UserDataCommand_RespondDBAvatars::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebf4 and returns with ret 4. |
| `015246b0` | UserDataCommand_RespondDBAvatars::RegisterFactory | speculative | Calls FUN_013ce130(0x231, FUN_01524650) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `01523e30` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; builds a record object on FUN_012f8c70 and copies its list members with FUN_012fa… |
| `01523ee0` | — | unknown | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8, list writers FUN_012fbec0 and FUN_012fcdc0, then i… |
| `015245e0` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (never assigned); when non-null calls FUN_015243e0 (constructor); calls the virtual at vtable slot [18… |
| `01524650` | — | unknown | Creator callback: tests an unresolved stack local against zero, returns 0 when zero, otherwise calls FUN_015243e0 (constructor) and returns its result. |

**UserDataCommand_RespondDBAwards** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01523950` | UserDataCommand_RespondDBAwards::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond DB Awards (34 bytes, literal length 0x22) into the sret buffer via FUN_01241650 and returns. |
| `01523970` | UserDataCommand_RespondDBAwards::UserDataCommand_RespondDBA… | confirmed | Constructor: calls FUN_01528920 (base constructor), stores the DBAwards vtable at [this], zeroes its member fields and returns this. |
| `01523a20` | UserDataCommand_RespondDBAwards::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondDBAwards (mov eax, imm32; ret). |
| `01523c60` | UserDataCommand_RespondDBAwards::~UserDataCommand_RespondDB… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_01523a60 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `01523c80` | UserDataCommand_RespondDBAwards::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondDBAwards via FUN_00d8d382; on failure calls FUN_012f5a60 with the clone … |
| `01523a60` | UserDataCommand_RespondDBAwards::DtorBody | likely | Stores UserDataCommand_RespondDBAwards::vftable at [this]; frees and zeroes the list buffers (guarded by non-null tests); calls FUN_012327d0 and FUN_01528a10 (… |
| `01523d40` | UserDataCommand_RespondDBAwards::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; reads the list members and scalars with list readers and FUN_013c… |
| `01523a30` | UserDataCommand_RespondDBAwards::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebf0 (mov eax,[0x01cbebf0]; ret). |
| `01523a40` | UserDataCommand_RespondDBAwards::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebf0 and returns with ret 4. |
| `01523c40` | UserDataCommand_RespondDBAwards::RegisterFactory | speculative | Calls FUN_013ce130(0x232, FUN_01523be0) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `015233c0` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; builds a record object on FUN_012f8c70 and copies its list members with FUN_012fa… |
| `01523470` | — | unknown | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8, list writers FUN_012fbec0 and FUN_012fcdc0, then i… |
| `01523b70` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (never assigned); when non-null calls FUN_01523970 (constructor); calls the virtual at vtable slot [18… |
| `01523be0` | — | unknown | Creator callback: tests an unresolved stack local against zero, returns 0 when zero, otherwise calls FUN_01523970 (constructor) and returns its result. |

**UserDataCommand_RespondDBHonors** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01522ee0` | UserDataCommand_RespondDBHonors::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond DB Honors (34 bytes, literal length 0x22) into the sret buffer via FUN_01241650 and returns. |
| `01522f00` | UserDataCommand_RespondDBHonors::UserDataCommand_RespondDBH… | confirmed | Constructor: calls FUN_01528920 (base constructor), stores the DBHonors vtable at [this], zeroes its list and string members and returns this. |
| `01522fb0` | UserDataCommand_RespondDBHonors::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondDBHonors (mov eax, imm32; ret). |
| `015231f0` | UserDataCommand_RespondDBHonors::~UserDataCommand_RespondDB… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_01522ff0 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `01523210` | UserDataCommand_RespondDBHonors::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondDBHonors via FUN_00d8d382; on failure calls FUN_012f5a60 with the clone … |
| `01522ff0` | UserDataCommand_RespondDBHonors::DtorBody | likely | Stores UserDataCommand_RespondDBHonors::vftable at [this]; frees and zeroes the list buffers (guarded by non-null tests); calls FUN_012327d0 and FUN_01528a10 (… |
| `015232d0` | UserDataCommand_RespondDBHonors::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; reads the list members and scalars with the list readers and FUN_… |
| `01522fc0` | UserDataCommand_RespondDBHonors::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebec (mov eax,[0x01cbebec]; ret). |
| `01522fd0` | UserDataCommand_RespondDBHonors::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebec and returns with ret 4. |
| `015231d0` | UserDataCommand_RespondDBHonors::RegisterFactory | speculative | Calls FUN_013ce130(0x233, FUN_01523170) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `01522950` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf4-byte object with FUN_012f8c70(0xf4); copies this+0xc via FUN_012… |
| `01522a00` | — | unknown | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8, list writers FUN_012fbec0 and FUN_012fcdc0 on the … |
| `01523100` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (never assigned); when non-null calls FUN_01522f00 (constructor); calls the virtual at vtable slot [18… |
| `01523170` | — | unknown | Creator callback: tests an unresolved stack local against zero, returns 0 when zero, otherwise calls FUN_01522f00 (constructor) and returns its result. |

**UserDataCommand_RespondNewsMessage** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01524e50` | UserDataCommand_RespondNewsMessage::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond News Message (37 bytes, literal length 0x25) into the sret buffer via FUN_01241650 and retur… |
| `01524f20` | UserDataCommand_RespondNewsMessage::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondNewsMessage (mov eax, imm32; ret). |
| `01525180` | UserDataCommand_RespondNewsMessage::~UserDataCommand_Respon… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_01524f60 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `015251a0` | UserDataCommand_RespondNewsMessage::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondNewsMessage via FUN_00d8d382; on failure calls FUN_012f5a60 with the clo… |
| `01524930` | UserDataCommand_RespondNewsMessage::Write | likely | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), list writer FUN_012fcdc0 on the list member, then inherited slot [3] with th… |
| `01524e70` | UserDataCommand_RespondNewsMessage::UserDataCommand_Respond… | likely | Constructor: calls FUN_01528920 (base constructor), stores the News vtable at [this], zeroes its member fields and returns this. |
| `01524f60` | UserDataCommand_RespondNewsMessage::DtorBody | likely | Stores UserDataCommand_RespondNewsMessage::vftable at [this]; frees and zeroes the list buffers (guarded by non-null tests); calls FUN_012327d0 and FUN_01528a1… |
| `01525260` | UserDataCommand_RespondNewsMessage::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; for each of four fields at this+0x8, +0x18, +0x28, +0x38 calls FU… |
| `01524f30` | UserDataCommand_RespondNewsMessage::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebf8 (mov eax,[0x01cbebf8]; ret). |
| `01524f40` | UserDataCommand_RespondNewsMessage::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebf8 and returns with ret 4. |
| `01525160` | UserDataCommand_RespondNewsMessage::RegisterFactory | speculative | Calls FUN_013ce130(0x228, FUN_01525100) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `01524880` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf1-byte object with FUN_012f8c70(0xf1); copies this+0x8, +0x18, +0x… |
| `01525090` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (never assigned); when non-null calls FUN_01524e70 (constructor); calls the virtual at vtable slot [18… |
| `01525100` | — | unknown | Creator callback: tests an unresolved stack local against zero, returns 0 when zero, otherwise calls FUN_01524e70 (constructor) and returns its result. |

**UserDataCommand_RespondPlayerAvatars** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015220a0` | UserDataCommand_RespondPlayerAvatars::DtorBody | confirmed | Stores UserDataCommand_RespondPlayerAvatars::vftable at [this]; frees and zeroes the four list buffers at this+0x40, +0x30, +0x20 and +0x10 (each guarded by a … |
| `01522190` | UserDataCommand_RespondPlayerAvatars::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondPlayerAvatars (mov eax, imm32; ret). |
| `01522240` | UserDataCommand_RespondPlayerAvatars::~UserDataCommand_Resp… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_015220a0 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `01522600` | UserDataCommand_RespondPlayerAvatars::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond Player Avatars (39 bytes, literal length 0x27) into the sret buffer via FUN_01241650 and ret… |
| `01522620` | UserDataCommand_RespondPlayerAvatars::UserDataCommand_Respo… | confirmed | Constructor: calls FUN_01528920 (base constructor), stores the Avatars vtable at [this] and zeroes this+0x8, +0x10, +0x14, +0x18, +0x20, +0x24, +0x28, +0x30, +… |
| `01522720` | UserDataCommand_RespondPlayerAvatars::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondPlayerAvatars via FUN_00d8d382; on failure calls FUN_012f5a60 with the c… |
| `015226b0` | UserDataCommand_RespondPlayerAvatars::CreateCopy | likely | Virtual slot [17]. Calls FUN_00d833a6 (result tested); when non-zero calls FUN_01522620 (constructor) for the new object; then calls the virtual at vtable slot… |
| `01522870` | UserDataCommand_RespondPlayerAvatars::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; reads four lists at this+0xc, +0x1c, +0x2c, +0x3c with FUN_00420f… |
| `015221a0` | UserDataCommand_RespondPlayerAvatars::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebe8 (mov eax,[0x01cbebe8]; ret). |
| `015221b0` | UserDataCommand_RespondPlayerAvatars::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebe8 and returns with ret 4. |
| `015227a0` | UserDataCommand_RespondPlayerAvatars::CreateInstance | speculative | Creator callback: calls FUN_00d833a6 (result tested); returns 0 on failure, otherwise calls FUN_01522620 (constructor) and returns its result. |
| `01522800` | UserDataCommand_RespondPlayerAvatars::RegisterFactory | speculative | Calls FUN_013ce130(0x234, FUN_015227a0) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `01521fe0` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf5-byte object with FUN_012f8c70(0xf5); copies this+0x8 via FUN_012… |
| `015221d0` | — | unknown | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8, FUN_012fbec0 on the lists, then inherited slot [3]… |

**UserDataCommand_RespondPlayerAwards** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01521a10` | UserDataCommand_RespondPlayerAwards::DtorBody | confirmed | Stores UserDataCommand_RespondPlayerAwards::vftable at [this]; frees the buffer at this+0x10 when non-null and zeroes this+0x10, +0x14, +0x18; calls FUN_01528a… |
| `01521aa0` | UserDataCommand_RespondPlayerAwards::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondPlayerAwards (mov eax, imm32; ret). |
| `01521b30` | UserDataCommand_RespondPlayerAwards::~UserDataCommand_Respo… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_01521a10 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `01521c60` | UserDataCommand_RespondPlayerAwards::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond DB Awards (34 bytes, literal length 0x22) into the sret buffer via FUN_01241650 and returns. |
| `01521c80` | UserDataCommand_RespondPlayerAwards::UserDataCommand_Respon… | confirmed | Constructor: calls FUN_01528920 (base constructor), stores the Awards vtable at [this], zeroes this+0x8, +0x10, +0x14 and +0x18, and returns this. |
| `01521e60` | UserDataCommand_RespondPlayerAwards::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondPlayerAwards via FUN_00d8d382; on failure calls FUN_012f5a60 with the cl… |
| `01521ae0` | UserDataCommand_RespondPlayerAwards::Write | likely | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_012fbec0 on the list at this+0xc, FUN_013c23f0 on this+0x8, then inherit… |
| `01521f50` | UserDataCommand_RespondPlayerAwards::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; FUN_00420fb6 then FUN_01300540 on the list at this+0xc; FUN_013c2… |
| `01521ab0` | UserDataCommand_RespondPlayerAwards::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebe4 (mov eax,[0x01cbebe4]; ret). |
| `01521ac0` | UserDataCommand_RespondPlayerAwards::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebe4 and returns with ret 4. |
| `01521ec0` | UserDataCommand_RespondPlayerAwards::CreateInstance | speculative | Creator callback: calls FUN_00d833a6 (result tested), and returns 0 on failure, otherwise calls FUN_01521c80 (constructor) and returns its result. |
| `01521f20` | UserDataCommand_RespondPlayerAwards::RegisterFactory | speculative | Calls FUN_013ce130(0x235, FUN_01521ec0) and passes its result as the receiver of FUN_013ce230; 23-byte body. |
| `01521970` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf6-byte object with FUN_012f8c70(0xf6); copies this+0xc via FUN_012… |
| `01521df0` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (pvVar1, never assigned); when non-null calls FUN_01521c80 (constructor); calls the virtual at vtable … |

**UserDataCommand_RespondPlayerHonors** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01521110` | UserDataCommand_RespondPlayerHonors::DtorBody | confirmed | Stores UserDataCommand_RespondPlayerHonors::vftable at [this]; frees and zeroes the four list buffers at this+0x40, +0x30, +0x20 and +0x10 (each guarded by a n… |
| `01521200` | UserDataCommand_RespondPlayerHonors::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondPlayerHonors (mov eax, imm32; ret). |
| `015212b0` | UserDataCommand_RespondPlayerHonors::~UserDataCommand_Respo… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_01521110 on this, tests bit 0 of the flag argument, frees this when set; returns this. |
| `01521470` | UserDataCommand_RespondPlayerHonors::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond Player Honors (38 bytes, literal length 0x26) into the sret buffer by calling FUN_01241650 a… |
| `01521490` | UserDataCommand_RespondPlayerHonors::UserDataCommand_Respon… | confirmed | Constructor: calls FUN_01528920 (base constructor, cookie-guarded), stores the Honors vtable at [this] and zeroes this+0x8, +0x10, +0x14, +0x18, +0x20, +0x24, … |
| `01521590` | UserDataCommand_RespondPlayerHonors::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondPlayerHonors via FUN_00d8d382; on failure calls FUN_012f5a60 with the cl… |
| `01521240` | UserDataCommand_RespondPlayerHonors::Write | likely | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8, FUN_012fbec0 on the four lists at this+0xc, +0x1c,… |
| `01521210` | UserDataCommand_RespondPlayerHonors::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebe0 (mov eax,[0x01cbebe0]; ret). |
| `01521220` | UserDataCommand_RespondPlayerHonors::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebe0 and returns with ret 4. |
| `01521670` | UserDataCommand_RespondPlayerHonors::RegisterFactory | speculative | Calls FUN_013ce130(0x236, FUN_01521610) and passes its result as the receiver of FUN_013ce230; 23-byte body, no return value used. |
| `01520e90` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf7-byte object with FUN_012f8c70(0xf7); copies this+0x8 via FUN_012… |
| `01521520` | — | unknown | Virtual slot [17]. Declares an unresolved result pointer (pvVar1, never assigned); when non-null calls FUN_01521490 (constructor); calls the virtual at vtable … |
| `01521610` | — | unknown | Creator callback with no stack arguments: tests an unresolved stack local (Ghidra local_10) against zero, returns 0 when zero, otherwise calls FUN_01521490 (co… |
| `01521730` | — | unknown | Virtual slot [9] (__thiscall, param_2 = version or stream selector). Reads the four lists at +0xc..+0x3c with FUN_0041f5e6 on the stored counts; calls inherite… |

**UserDataCommand_RespondPlayerInfo** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015204b0` | UserDataCommand_RespondPlayerInfo::DtorBody | confirmed | Stores UserDataCommand_RespondPlayerInfo::vftable at [this]; frees the string buffer at this+0x38 when capacity (this+0x4c) exceeds 0xf, resets size and capaci… |
| `01520570` | UserDataCommand_RespondPlayerInfo::GetTypeName | confirmed | Virtual slot [11]. Returns the constant char* UserDataCommand_RespondPlayerInfo (mov eax, imm32; ret). |
| `01520640` | UserDataCommand_RespondPlayerInfo::~UserDataCommand_Respond… | confirmed | Scalar deleting destructor (slot [0]): calls the body FUN_015204b0 on this, tests bit 0 of the flag argument, and calls the free thunk on this when the bit is … |
| `01520a40` | UserDataCommand_RespondPlayerInfo::GetDisplayName | confirmed | Virtual slot [20]. Builds the std::string UserData Command Respond Player Info (36 bytes) into the sret buffer by calling FUN_01241650(literal, 0x24) and retur… |
| `01520a60` | UserDataCommand_RespondPlayerInfo::UserDataCommand_RespondP… | confirmed | Constructor body: calls FUN_01528920 (base constructor), stores the RespondPlayerInfo vtable at [this], zeroes this+0x8, +0x28, +0x2c, +0x30, sets capacity thi… |
| `01520c60` | UserDataCommand_RespondPlayerInfo::CloneInto | confirmed | Virtual slot [18]. Dynamic-casts the NetworkCommand argument to UserDataCommand_RespondPlayerInfo via FUN_00d8d382; if the cast fails calls FUN_012f5a60 with t… |
| `015205b0` | UserDataCommand_RespondPlayerInfo::Write | likely | Virtual slot [10]. Calls inherited slot [2] with the stream, FUN_01528bb0(stream), FUN_013c23f0 on this+0x8 and on the six dwords at +0xc..+0x20, FUN_012fbec0 … |
| `01520d80` | UserDataCommand_RespondPlayerInfo::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream). Calls inherited slot [4]; FUN_01528b70 gate; FUN_013c24d0 reads into this+0xc..+0x20 (six dwords) and this+0x8… |
| `01520580` | UserDataCommand_RespondPlayerInfo::GetStaticSlot | speculative | Virtual slot [12]. Returns the dword at global DAT_01cbebdc (mov eax,[0x01cbebdc]; ret). No arguments. |
| `01520590` | UserDataCommand_RespondPlayerInfo::SetStaticSlot | speculative | Virtual slot [13]. Stores its stack argument into global DAT_01cbebdc and returns with ret 4. |
| `01520d50` | UserDataCommand_RespondPlayerInfo::RegisterFactory | speculative | Calls FUN_013ce130(0x237, FUN_01520cf0) and passes its result as the receiver of FUN_013ce230; no return value is used. Body is 23 bytes. |
| `015203d0` | — | unknown | Virtual slot [15]. Calls FUN_00d833a6(0x14) and, when non-zero, FUN_012f9eb0; allocates a 0xf8-byte object with FUN_012f8c70(0xf8); copies fields this+0x8, +0x… |
| `01520bf0` | — | unknown | Virtual slot [17]. Declares a result pointer that the decompiler lost (pvVar1, never assigned in the body); when it is non-null calls FUN_01520a60 (constructor… |
| `01520cf0` | — | unknown | Creator callback with no stack arguments: tests an unresolved stack local (Ghidra local_10) against zero, returns 0 when it is zero, otherwise calls FUN_01520a… |

**UserDataCommand_RespondPlayerMatches** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151fcf0` | UserDataCommand_RespondPlayerMatches::Read | likely | Virtual slot [9] (__thiscall, param_2 = stream, version tested as param_2 < 2). Calls inherited slot [4] to read the header; gates on FUN_01528b70; reads with … |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01520660` | CopyString_Off34 | speculative | Assigns the std::string at this+0x34 into the caller-supplied std::string (sret, param_2) through basic_string::assign(src, 0, npos) and returns param_2. |
| `01520ae0` | ReleaseMember24 | speculative | Calls FUN_005f15f0 on the member at this+0x24 and returns param_2 unchanged (ret 4 shape). |
| `01521ce0` | — | unknown | Calls FUN_005f15f0 on the member at this+0xc and returns param_2 unchanged. |

## Source shards

[[S0450|S0450]] (45), [[S0531|S0531]] (57), [[S0532|S0532]] (64), [[S0533|S0533]] (58), [[S0535|S0535]] (59), [[S0536|S0536]] (41), [[S0537|S0537]] (20), [[S0538|S0538]] (83), [[S0539 00f87df0-00f8c800|S0539]] (36), [[S0540|S0540]] (10), [[S0792|S0792]] (136), [[S0793|S0793]] (118), [[S0794|S0794]] (125)
