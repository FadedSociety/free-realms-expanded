# Class: CaptureCommon::JobQueueCallbackHandlerT<ImageCapture::JobRec>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d0b3a0` | `ImageCaptureJobQueueCallbackHandler_Ctor` | ✅ | ctor | Installs JobQueueHandler then capture callback-handler vtables, stores callback value, and configures handler with bounds 0 and 0x7fffffff. |
| `00d0b410` | `JobQueueCallbackHandler_DtorBody` | ✅ | dtor | Restores callback-handler vtable, calls cleanup helper, then restores base JobQueueHandler vtable. |
| `00d0b470` | `AdjustorThunk_00fc4db0` | ✅ | virtual | The 16-byte x86 body writes constant 1 as stack argument, adjusts ECX by 8, and jumps to 00fc4db0. |
| `00d0b490` | `JobQueueCallbackHandler_ScalarDeletingDtor` | ✅ | virtual | Calls callback-handler destructor, frees object when requested, and otherwise returns original pointer. |
| `00d0be00` | `JobQueueCallbackHandler_OnJob` | ✅ | virtual | Forwards callback arguments to routine that appends the job pointer to the queue record. |
