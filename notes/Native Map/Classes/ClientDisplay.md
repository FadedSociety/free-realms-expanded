# Class: ClientDisplay

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b16e10` | `ClientDisplay::ClientDisplay` | ✅ | ctor | ClientDisplay constructor: installs ClientDisplay::vftable into *this, stores the owner argument at +8, zeroes the loading-screen cache pointer at +0xc, and calls the display base constructor 00ae8360 with argument 0. |
| `00b16ea0` | `ClientDisplay::ShowLoadingScreen ⚠downgrade` | 🟢 | method | Creates or refreshes the loading screen cached at this+0xc: when a cached object exists and its +0x14/+0x10 fields differ from the two size arguments it is destroyed through 00caacc0 and freed, otherwise 00cab450 is inv… |
| `00b16f90` | `ClientDisplay::DropLoadingScreen` | 🟢 | method | Drops the loading screen: when the owner object at this+8 has a non-null field at +0x24c it calls 009f7900 with that object and the float constant at 0x01762a34 (which walks a list applying 00c67730 to each element), th… |
| `00b16fe0` | `— ⚠reject` | ⚪ | dtor | ClientDisplay destructor: installs ClientDisplay::vftable, destroys and frees the cached loading-screen object at +0xc through 00caacc0 followed by _free, then calls the base destructor 00ae8460. |
| `00b17060` | `ClientDisplay::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of ClientDisplay::vftable: calls the destructor body 00b16fe0 and frees the object with _free when bit 0 of the flag argument is set, otherwise returns the pointer. |
