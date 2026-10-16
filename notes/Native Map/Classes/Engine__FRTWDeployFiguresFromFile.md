# Class: Engine::FRTWDeployFiguresFromFile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012521c0` | `Engine::FRTWDeployFiguresFromFile::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWDeployFiguresFromFile::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01252230` | `Engine::FRTWDeployFiguresFromFile::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::FRTWDeployFiguresFromFile::vftable. Bytes b8 imm32 c3: returns the constant C string 'DeployFiguresFromFile'. |
| `01252250` | `Engine::FRTWDeployFiguresFromFile::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'DeployFiguresFromFile : 0 : ..\engine\game\frtwgamehandlers.cc:1970'. |
| `01252260` | `Engine::FRTWDeployFiguresFromFile::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125e010` | `— ⚠reject` | ⚪ | virtual | Reads the string argument named FileName (012eb250), assigns it to a local std::string, and passes that string to 013210e0; returns no value. It does not create a ScaleformAction. |
