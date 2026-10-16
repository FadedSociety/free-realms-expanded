# Class: YouTube::FRYouTubeUploaderHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2e0e0` | `OnVideoUploadAuthenticate` | ✅ | virtual | Virtual slot 0 of FRYouTubeUploaderHandler: when the function pointer at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadAuthenticate' and the third stack argument, then returns (ret 0xC). |
| `00d2e100` | `OnVideoUploadPost` | ✅ | virtual | Virtual slot 1 of FRYouTubeUploaderHandler: when the callback at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadPost' and the third stack argument, then returns (ret 0xC). |
| `00d2e130` | `OnVideoUploadProgress` | ✅ | virtual | Virtual slot 3 of FRYouTubeUploaderHandler: when the callback at this+4 is non-null, calls it with the string 'VideoHandler:OnVideoUploadProgress' and the second stack argument (param_3), then returns (ret 8). |
| `00d2e170` | `Ctor` | ✅ | ctor | Stores the FRYouTubeUploaderHandler vptr at +0 and the stack argument at +4, then returns this in EAX (ret 4). |
| `00d2e190` | `DeleteIfFlagBit0 ?` | 🟡 | virtual | Virtual slot 4 of FRYouTubeUploaderHandler: stores the vptr at +0, frees this through _free when bit 0 of the byte argument is set, and returns this (ret 4). It does not call a separate dtor body. |
