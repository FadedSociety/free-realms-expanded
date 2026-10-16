# Class: Logging::LoggingApi

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00897050` | `Logging::LoggingApiCtor` | ✅ | ctor | Constructs the Logging::LoggingApi singleton: registers with the SOE logging shim FUN_010d7650("LOGAPI_100",...), installs the LogHandler base vtable at +0xbe8 and the LoggingApi vtables at +0, +0xc0, +0xc4, then initia… |
| `008973e0` | `Logging::LoggingApi::~LoggingApi` | ✅ | dtor | Destructor: reinstalls the LoggingApi vtables at +0/+0xc0/+0xc4/+0xbe8, releases the owned log-file manager object at +0xd270 through FUN_0089a630 plus _free, tears down the Hash<int,int,512,-1> at +0xd3bc and the Strin… |
| `008977c0` | `Logging::LoggingApiScalarDtor` | 🟢 | dtor | Scalar deleting destructor of Logging::LoggingApi: runs the full destructor FUN_008973e0 and releases the object with _free when bit 0 of param_2 is set. |
