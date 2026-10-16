# System: engine.overhead

45 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0096f540` | `OverHeadTextElementCtor` | OverHeadTextElement | Constructs a text-bearing overhead element used by proxied objects. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `0096f620` | `OverHeadTextElementDtor` | OverHeadTextElement | Destroys the derived overhead text element and runs base cleanup. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `0096f680` | `OverHeadTextElementScalarDeletingDtor` | OverHeadTextElement | Implements scalar deleting destruction for OverHeadTextElement. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00a4c330` | `OverHeadNameIconElement::OverHeadNameIconElement` | OverHeadNameIconElement | Constructor of the overhead element that draws the player-name icon above a character (extends OverHeadBitmapElement with one byte of state). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4c430` | `OverHeadNameIconElement::ScalarDeletingDtor` | OverHeadNameIconElement | Standard scalar deleting destructor of the name-icon overhead element. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4c490` | `OverHead_ChatBubbleResourcesInit` |  | Loader for the overhead/chat-bubble assets: it reserves texture slots 42000..42025 for the nine chat-bubble borders, nine fills, three inner shadows and five HP-bar strips, so the overhead draw code can reference them b… | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00bf0350` | `OverHeadChatBubbleElement::Ctor` | OverHeadChatBubbleElement | Creates an overhead chat bubble display element. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0450` | `OverHeadChatBubbleElement::Dtor` | OverHeadChatBubbleElement | Releases chat bubble-owned strings and base state. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf04d0` | `OverHeadChatBubbleElement::ScalarDeletingDtor` | OverHeadChatBubbleElement | Implements scalar deleting destruction for overhead chat bubbles. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00957900` | `OverHeadElement::GetOffset18` | OverHeadElement | Shared inherited accessor for OverHeadElement and its derived overhead classes; returns the object pointer plus 0x18. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957910` | `OverHeadElement::GetOffset40` | OverHeadElement | Shared inherited accessor for OverHeadElement and its derived overhead classes; returns the object pointer plus 0x40. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00962ef0` | `SetDisplayTransformAndRefresh` |  | The refresh callee creates an overhead name-icon element when its definition resolves. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a4c3e0` | `OverHeadNameIconElement::~OverHeadNameIconElement` | OverHeadNameIconElement | Destructor of the name-icon overhead element; twin of the scalar deleting destructor at 00a4c430 that occupies vtable slot 0. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4cdf0` | `OverHeadElementSet_Init` |  | Neighbour-chain inference: created by the overhead-set factory 00a4cea0 and holding eight nested OverHeadElement* arrays, this is the constructor of the OverHeadElementSet class named by the Array<OverHeadElementSet*> R… | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4cea0` | `OverHeadManager_CreateElementSet` |  | Creates and registers a new overhead element set in the manager's set list; callers 00bf00e0 and 00cad100 add sets for new content. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4d2c0` | `OverHead_AppendRow9` |  | Configuration-row builder for the overhead system: 9-int rows stored in the manager table, added during client UI startup (caller 0092b170). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4d3e0` | `OverHead_AppendRow3_A` |  | Configuration-row builder for a second overhead table (3-int rows), added during client UI startup (caller 0092b170). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4d480` | `OverHead_AppendRow3_B` |  | Configuration-row builder for a third overhead table (3-int rows), added during client UI startup (caller 0092b170). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00b6a6e0` | `BuildOverheadTextEntry → FishingProcessor::SetProxiedObjectOverheadText` |  | Creates or updates an overhead-text element for an entry processed by the fishing path. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a8c0` | `RefreshOverheadTextEntries → FishingProcessor::RefreshSchoolOverheadText` |  | Updates overhead text for a collection of entries. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |

## 🟡 speculative (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00957850` | `OverHeadElementCtor ?` | OverHeadElement | Installs OverHeadElement vtable, initializes several scalar fields, copies supplied vectors/values into offsets 0x14 through 0x4c, and stores static defaults. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009578e0` | `OverHeadElementDtor ?` | OverHeadElement | Writes OverHeadElement::vftable to the object and returns. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009578f0` | `OverHeadElement::GetOffset14 ?` | OverHeadElement | Returns this pointer plus 0x14. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957920` | `OverHeadElement::SetField40 ?` | OverHeadElement | Copies one dword from the input pointer to this+0x40. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957930` | `OverHeadElement::GetOffset30 ?` | OverHeadElement | Returns this pointer plus 0x30. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957940` | `OverHeadElement::SetField30Block ?` | OverHeadElement | Copies four dwords from the input into object offsets 0x30, 0x34, 0x38, and 0x3c. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957960` | `OverHeadElement::GetOffset44 ?` | OverHeadElement | Returns this pointer plus 0x44. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957970` | `OverHeadElement::SetFloat44 ?` | OverHeadElement | Loads and stores one floating-point value at this+0x44. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957980` | `OverHeadElement::GetOffset48 ?` | OverHeadElement | Returns this pointer plus 0x48. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957990` | `OverHeadElement::SetFloat48 ?` | OverHeadElement | Loads and stores a floating-point value at this+0x48. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009579a0` | `OverHeadElement::SetByte4c ?` | OverHeadElement | Copies one byte from input to this+0x4c. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009579b0` | `OverHeadElement::GetOffset4c ?` | OverHeadElement | Returns this pointer plus 0x4c. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009579c0` | `OverHeadElement::GetOffset4d ?` | OverHeadElement | Returns this pointer plus 0x4d. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009579d0` | `OverHeadElement::SetField20Block ?` | OverHeadElement | Copies four dwords from input offsets 0,4,8,0xc into object offsets 0x20,0x24,0x28,0x2c. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009579f0` | `OverHeadElement::SetFloat34 ?` | OverHeadElement | Moves one scalar float into this+0x34. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957a00` | `OverHeadElement::GetFloat34 ?` | OverHeadElement | Returns the float stored at this+0x34. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957a10` | `OverHeadElement::ScalarDeletingDtor ?` | OverHeadElement | Installs the OverHeadElement vtable, then frees the object when deletion flag bit 0 is set; returns the object otherwise. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957a40` | `OverHeadHpBarElementCtor ?` | OverHeadHpBarElement | Calls the OverHeadElement base constructor, installs the HP-bar vtable, stores one supplied value, and initializes two fields including capacity 0x10. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957af0` | `OverHeadHpBarElementDtor ?` | OverHeadHpBarElement | Restores the derived HP-bar vtable, then the OverHeadElement base vtable. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957b60` | `OverHeadHpBarElement::ScalarDeletingDtor ?` | OverHeadHpBarElement | Restores derived then base vtables, and frees the object when deletion flag bit 0 is set. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009622f0` | `— ⚠reject` |  | The constructor anchor identifies the created object as an overhead name-icon element; table-key meanings are not established. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `009625d0` | `— ⚠reject` |  | The caller context is a display-related scan, but the gate's semantic meaning is not recovered. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a4bc10` | `— ⚠reject` |  | Neighbour-chain inference: the destroyed member arrays are exactly the OverHead element-set storage types built by the overhead resource loader (00a4c490), so this is the teardown of the owning overhead/manager object; … | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4c260` | `— ⚠reject` | OverHeadBitmapElement | Constructor of the overhead element that draws a bitmap (icon/bar) above a character; the 12-byte record pairs a texture/resource id with an amount and flags for the overhead render pass. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4c780` | `— ⚠reject` |  | Visibility/sorting pass for the overhead display: it selects the element sets inside the camera cone and orders them by projected distance so the draw pass can render the nearest name-tags/chat-bubbles on top. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |

