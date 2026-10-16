# Class: BackgroundMusicTrackManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f6b40` | `BackgroundMusicTrackManager::BackgroundMusicTrackManager` | ✅ | ctor | Thiscall with one stack argument (ret 4), SEH-framed. Calls 00c67ad0 on this with the argument, installs BackgroundMusicTrackManager::vftable into [this], zeroes the dword at this+0x38, and returns this. |
| `009f7060` | `BackgroundMusicTrackManager::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot [0] of BackgroundMusicTrackManager::vftable: installs an SEH frame, calls 00c67b70 with this in ECX and no arguments (bytes checked: ECX is still the incoming this), then calls _free(thi… |
