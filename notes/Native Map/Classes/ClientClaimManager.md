# Class: ClientClaimManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff860` | `ClaimSystemLog_Level5` | 🟢 | virtual | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 5 and the file name 'ClaimSystem.log'. |
| `009ff880` | `ClaimSystemLog_Level4` | 🟢 | virtual | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 4 and the file name 'ClaimSystem.log'. |
| `009ff8a0` | `ClaimSystemLog_Level3` | 🟢 | virtual | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 3 and the file name 'ClaimSystem.log'. |
| `009ff8c0` | `ClaimSystemLog_Level2` | 🟢 | virtual | Logging leaf: forwards a varargs format string to the shared logger 00700f90 with level 2 and the file name 'ClaimSystem.log'. |
| `00a03490` | `— ⚠reject` | ⚪ | dtor | Destructor body: stores ClientClaimManager::vftable, frees the 8-byte member at +0x98 through 00c694d0, clears that member and the byte at +8, then tears down the tracker map (00a03420), the claim item array (00a01400) … |
| `00a03910` | `ClientClaimManager::ClientClaimManager` | ✅ | ctor | Constructor: stores ClientClaimManager::vftable, stores the argument at +4 (the client/owner), clears the bytes at +8/+9 and the dword at +0xc, builds the callback list (00a02fd0), allocates 8 bytes and constructs the m… |
