# Class: SoeUtil::Array<float,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00887e20` | `Array::~Array` | ✅ | dtor | Fastcall (ECX = object). EH frame. Stores SoeUtil::Array<float,0,1> vtable at [this+0x20], zeroes the count at +0x28, frees the buffer at +0x24 with FUN_00401c00(ptr,4) and zeroes the buffer, then calls FUN_008785c0 (ba… |
| `009d3fe0` | `FloatArrayCopyCtor` | ✅ | ctor | Initializes an empty Array<float,0,1>, reserves for the source count, copies the source's used floats into the new buffer, updates the destination count, and returns the destination pointer. |
