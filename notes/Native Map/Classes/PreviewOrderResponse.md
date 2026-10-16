# Class: PreviewOrderResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00f894a0` | `— ⚠reject` | ⚪ | dtor | Destructor body (does not free this) for the PreviewOrderResponse layout: releases nine IString-shaped members at this+0x00 to +0x80 (step 0x10), the subscription-product array at +0x90 (destroyed with 00f66990, count a… |
| `00f8b780` | `PreviewOrderResponse::Ctor` | ✅ | ctor | Constructor for the PreviewOrderResponse fields: nine IString-shaped members at this+0x00 to +0x80 set to empty, SoeUtil::Array<Uram::PreviewOrderResponse::SubscriptionProduct> vtable at this+0x90 and SoeUtil::Array<Ura… |
| `00f8c2f0` | `PreviewOrderResponse::Parse` | ✅ | method | XML reply parser (same template): loads the document (00d549d0) and root (00d50ac0); no root reports 'PreviewOrderResponse::Parse, failed with invalid xml'. Requires root purchasepreviewresponse (string 01877d2c), so th… |
