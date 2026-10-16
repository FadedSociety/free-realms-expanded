# Class: AssetManagement::AssetDeliveryDirect::FileLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0109dfa0` | `AssetDeliveryDirect::FileLoader::FileLoader` | ✅ | ctor | Constructor: calls 00fc4cc0, installs the FileLoader vtable, zeroes the list links at +0xc4 and +0xc8, stores the owner queue pointer param_2 at +0xcc and increments the owner's refcount at +4 with a lock prefix; return… |
| `0109e030` | `AssetDeliveryDirect::FileLoader::~FileLoader` | ✅ | dtor | Destructor body of FileLoader: installs the FileLoader vtable and runs the base teardown 00fc4c30. |
| `0109e080` | `AssetDeliveryDirect::FileLoader::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: installs the FileLoader vtable, runs the base teardown 00fc4c30, and calls _free on the object when bit 0 of the stack argument is set; returns the object pointer. |
