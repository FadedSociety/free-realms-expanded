# Class: UIProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86fa0` | `UIProcessorCreateObject0x34` | 🟢 | virtual | UIProcessor vtable slot [3] factory: allocates 0x34 bytes through 00d833a6 (the call pushes only the size); on success sets ECX to the allocated object and calls 00d355b0 with stack arguments param_2, the pointer comput… |
| `00a8d500` | `UIProcessorUpdateHashedEntry` | ✅ | virtual | Looks up param_3 in a 128-bucket table, retains and assigns the supplied object on a match, notifies the processor for noncurrent keys, and inserts keys above 999 after subtracting 1000; returns 1. |
| `00a97450` | `UIProcessor::~UIProcessor` | ✅ | dtor | Destructor body: stores UIProcessor::vftable, releases a refcounted list at this+0x4fc (per-node refcount decrement, node free via 00a8ce60), walks a second list at this+0x583 calling 00a88a10, destroys members via 00a8… |
| `00a98020` | `UIProcessor::UIProcessor` | ✅ | ctor | Constructor body: stores AssetManagement::AssetRequestInterface::AssetRequester::vftable, then UIProcessor::vftable, initialises fields at this+0x8..0x2c and 0x2c..0x54 (counters set to 0 or -1), allocates a 0x264-byte … |
| `00a98250` | `UIProcessor::ScalarDeletingDtor` | 🟢 | dtor | Calls UIProcessor destructor body 00a97450 on this, then if the low bit of the flag argument is set calls _free on this; returns this. |
