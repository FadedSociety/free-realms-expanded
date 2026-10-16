# Class: T4StringBuffer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00e73af0` | `T4StringBuffer::T4StringBuffer` | ✅ | ctor | thiscall, C string in stack arg 1 and length in arg 2 (-1 = strlen): stores T4BaseStream::vftable, zeroes +4 and +8, computes the length (strlen+0 or the given value) into +0xc, stores 100 at +0x10 and zeroes the bytes … |
| `00e73c20` | `T4StringBuffer::T4StringBuffer` | ✅ | ctor | thiscall, source object pointer in stack arg 1: stores T4BaseStream::vftable, zeroes +4, copies the source's dwords at +8, +0xc, +0x10 and bytes at +0x14, +0x15, +0x16. When the source's data pointer (+4) is non-zero it… |
| `00e746f0` | `T4StringBuffer::ReserveBuffer ? ⚠rename` | 🟡 | method | thiscall, minimum capacity in stack arg 1 (values below 0x40 become 0x3f): reads the flags of the virtual-base sub-object. Flag +0x14 set: grows the owned buffer through SoeBuffer::Ensure (00e70db0) when its size is bel… |
