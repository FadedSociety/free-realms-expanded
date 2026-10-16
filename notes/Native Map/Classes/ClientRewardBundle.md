# Class: ClientRewardBundle

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8a390` | `ClientRewardBundle::ClientRewardBundle` | ✅ | ctor | Constructor of the reward bundle object: runs the base constructor FUN_0091c110(0,0,0), installs ClientRewardBundle::vftable and stores param_2 at +0xb8. |
| `00b8a400` | `ClientRewardBundle::ScalarDeletingDtor` | ✅ | dtor | Deleting destructor variant of the reward bundle: destroys the bundle through FUN_0103d010 under an SEH frame and frees it through _free when bit 0 of param_2 is set. |
| `00b8a460` | `ClientRewardBundle_CopyCtor` | ✅ | ctor | Copy constructor of the reward bundle: runs the base copy FUN_009b9a70(param_2), installs ClientRewardBundle::vftable and copies the field at source+0xb8 into this+0xb8. |
