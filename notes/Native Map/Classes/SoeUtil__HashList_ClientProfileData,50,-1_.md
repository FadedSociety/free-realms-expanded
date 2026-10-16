# Class: SoeUtil::HashList<ClientProfileData,50,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3a760` | `HashList_ClientProfileData50_Construct` | ✅ | ctor | Initializes the shared 50-bucket list layout: zeros fields at +4 through +0x10, clears 200 bytes of bucket heads at +0x14, then leaves the HashList<ClientProfileData,50,-1> vtable installed at +0. |
| `00a3a7e0` | `HashList_ClientProfileData50_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the list-side vtable: calls the teardown body 00a3a6a0 (ECX=this) and then calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
