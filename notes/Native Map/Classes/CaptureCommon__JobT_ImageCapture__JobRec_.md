# Class: CaptureCommon::JobT<ImageCapture::JobRec>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d0be50` | `ImageCaptureJob_Construct` | ✅ | ctor | Initializes base job with argument 0, installs ImageCapture JobT vtable, and move-constructs its record reference. |
| `00d0bec0` | `— ⚠reject` | ⚪ | virtual | Dispatches through referenced record object's virtual slot +8, passing stored value at job offset +0x60 and reference at +0x6c. |
| `00d0bee0` | `ImageCaptureJob_DtorBody` | ✅ | dtor | Installs derived job vtable, releases referenced object through IString cleanup, and restores Job and RefCounted base vtables. |
| `00d0bf50` | `ImageCaptureJob_ScalarDeletingDtor` | ✅ | virtual | Calls JobT destructor, frees object when requested, and otherwise returns its original pointer. |
