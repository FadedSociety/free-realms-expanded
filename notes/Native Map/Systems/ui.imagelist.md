# System: ui.imagelist

13 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c13060` | `CheckImageListArg` |  | Argument accessor for the ImageList module methods. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c131d0` | `RegisterImageListLuaModule` |  | Creates the ImageList scripting interface used by UI screens that need runtime-loaded images. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c132d0` | `ImageArrayResize` | SoeUtil::Array<GuiDsImageList::Image,0,… | Capacity management for the GuiDsImageList image array (powering ImageList entries); the caller copies the elements into the returned buffer. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13370` | `ImageArrayFreeBuffer` | SoeUtil::Array<GuiDsImageList::Image,0,… | Buffer release hook of the GuiDsImageList image array. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13390` | `ImageArrayMoveAssign` | SoeUtil::Array<GuiDsImageList::Image,0,… | Move-assignment/swap of the GuiDsImageList image array. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c134c0` | `ImageArrayScalarDeletingDtor` | SoeUtil::Array<GuiDsImageList::Image,0,… | MSVC scalar deleting destructor of the GuiDsImageList image array. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c137a0` | `GuiDsImageList::ScalarDeletingDtor` | GuiDsImageList | MSVC scalar deleting destructor of the engine image-list object created by ImageList.Create. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c130c0` | `ImageListLoadImage` |  | Loads an image file into an ImageList entry from a script (the list then feeds GUI bitmaps). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13400` | `ImageArrayPopBack` | SoeUtil::Array<GuiDsImageList::Image,0,… | Element teardown used when the array shrinks (destructors call it with the full size to release everything). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13490` | `ImageListArrayDtor` | SoeUtil::Array<GuiDsImageList::Image,0,… | Cleanup of the SoeUtil::Array instance embedded in GuiDsImageList; called from two unwind funclets of the image-list code. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13570` | `GuiDsImageList::~GuiDsImageList` | GuiDsImageList | Tears down an ImageList object; called by its scalar deleting destructor 00c137a0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c135f0` | `ImageListCreate ⚠downgrade` |  | Factory entry of the ImageList scripting class used by UI screens to create runtime image lists. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c13500` | `— ⚠reject` | GuiDsImageList | Builds the engine-side image list object behind the ImageList scripting class; called by the Lua ImageList.Create implementation 00c135f0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

