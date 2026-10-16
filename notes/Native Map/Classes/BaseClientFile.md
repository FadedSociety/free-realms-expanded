# Class: BaseClientFile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae2e00` | `BaseClientFileCtor` | ✅ | ctor | Constructs the FileRep base, installs BaseClientFile vtable, zeros cursor/size/status fields, and initializes the embedded helper at the object tail. |
| `00ae2e70` | `BaseClientFile::SetStatus5` | ✅ | virtual | Stores integer 5 at this+0x10 and returns byte zero. |
| `00ae2e80` | `— ⚠reject` | ⚪ | virtual | Calls readiness helper 00702f50; when true, invokes backend helper 00702b20 and returns. |
| `00ae2ea0` | `BaseClientFile::GetStatus` | ✅ | virtual | Loads and returns the dword at this+0x10. |
| `00ae2f40` | `— ⚠reject` | ⚪ | virtual | If a shared representation exists, returns its 64-bit field at +8; otherwise queries the backend when ready, or returns zero. |
| `00ae2f70` | `— ⚠reject` | ⚪ | virtual | Returns the shared representation's 64-bit size when present; otherwise asks the ready backend for its 64-bit size or returns zero. |
| `00ae2fa0` | `BaseClientFile::IsOpen` | ✅ | virtual | Returns false when no shared representation exists and backend is not ready; otherwise returns true. |
| `00ae3070` | `— ⚠reject` | ⚪ | virtual | Decrements and clears the shared representation, resets status and cursor fields, then calls backend close when readiness helper succeeds. |
| `00ae30c0` | `BaseClientFile::Read` | ✅ | virtual | Copies up to the available byte count from shared representation into the caller buffer, advances the 64-bit cursor and optional read count, or delegates to the backend; sets status 6 if backend is unavailable. |
| `00ae3190` | `BaseClientFile::Seek` | ✅ | virtual | Delegates to backend seek when no shared representation exists; otherwise clamps the requested 64-bit position to file size and stores it as the cursor. |
| `00ae31f0` | `BaseClientFileDtor` | 🟢 | dtor | Destroys the embedded helper, decrements and clears the shared representation, and invokes the FileRep base destructor. |
| `00ae3280` | `BaseClientFile::ScalarDeletingDtor` | ✅ | dtor | Calls BaseClientFile destructor body, optionally frees this on flag bit 0, and returns this otherwise. |
| `00ae3310` | `BaseClientFile::Open` | ✅ | virtual | Resets cursor fields, gets the active opener, normalizes the path separator position, requests a shared representation, replaces any old representation with refcount updates, and falls back to backend open on failure. |
