# Class: SnsIntegrationCommon

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107fc80` | `ReleaseMember4IfNonNull ? ⚠rename` | 🟡 | virtual | Virtual slot 5 (shared by SnsIntegrationProcessor and SnsIntegrationCommon): if the pointer at this+0x4 is non-null it tail-jumps to 00d31890 with that pointer in ECX, else returns (bytes: mov ecx,[ecx+4]; test; je; jmp… |
| `01080ce0` | `SnsBuildMultipartPost` | ✅ | virtual | Virtual slot 6 implementation shared with SnsIntegrationProcessor. When the flag at this+0x10 is set it builds a multipart/form-data body (fields pact=xml, requestString, time as %llu, signature, closing boundary; bound… |
| `01081c60` | `SnsIntegrationCommonDtorBody` | ✅ | dtor | Destructor body of SnsIntegrationCommon: stores the SnsIntegrationCommon vftable, drains the two MessageTrackingNode lists at this+0x24 and this+0x138 through 00bb8000, calls 00d2ee60(0), frees the buffer at +0x4 throug… |
| `01081e10` | `— ⚠reject` | ⚪ | ctor | Constructor body: stores TinyHttp::HttpManagerHandler then SnsIntegrationCommon vftables at this+0x0, installs the IString<char> vftable at +0x8 with the static empty buffer, sets +0x10, +0x14 and +0x18 to 0, 0 and 1, c… |
| `01081f70` | `SnsIntegrationCommonScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 01081c60 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
