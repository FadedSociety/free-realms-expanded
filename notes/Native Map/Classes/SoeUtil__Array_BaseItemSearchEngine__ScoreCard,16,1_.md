# Class: SoeUtil::Array<BaseItemSearchEngine::ScoreCard,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e1d0` | `InlineArrayFreeBuffer` | ✅ | method | Aligns this+0x13 down to four bytes and frees the argument only when it differs from that inline buffer address. |
| `00c2e9f0` | `— ⚠reject` | ⚪ | dtor | Frees external storage if not the aligned inline buffer, clears the pointer/count, switches to the zero-inline base vtable, then frees its backing pointer. |
| `00c2eab0` | `InlineArrayResizeAllocator` | ✅ | method | Delegates requests above 16 to the base allocator; otherwise reports capacity 16 and returns aligned inline storage at this+0x13. |
| `00c2eee0` | `ScoreCardArrayCleanup → ScoreCardInlineArrayUnwindCleanup` | 🟢 | dtor | Sets up an exception-unwind frame, calls the inline ScoreCard array destructor on a temporary, and restores the exception-list link. |
| `00c2f1c0` | `ScoreCardInlineArrayCtor` | ✅ | ctor | Zeros the base array pointer/size/capacity fields and installs the inline-capacity-16 specialization vtable. |
| `00c2f220` | `ScoreCardInlineArrayScalarDeletingDtor` | 🟢 | dtor | Calls the inline-array destructor, then calls _free(this) when flag bit 0 is set; otherwise returns this. |
| `00c2f2f0` | `ScoreCardArrayCleanup → ScoreCardInlineArrayUnwindCleanup` | 🟢 | dtor | Sets an EH cleanup frame, invokes the inline ScoreCard destructor on a temporary, and restores the prior exception-list link. |
| `00c2f730` | `— ⚠reject` | ⚪ | method | Copies source active records to a temporary and clears source count, copies destination active records to another temporary and clears it, then destroys temporary vectors and buffers. |
| `00c2f890` | `— ⚠reject` | ⚪ | dtor | Runs inline ScoreCard array teardown, then frees the object when flag bit 0 is set. |
