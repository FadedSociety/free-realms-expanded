# Class: SoeUtil::Array<SoeGems::TextIndex<int>::InvertedInstance,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8c670` | `— ⚠reject` | ⚪ | virtual | Returns current storage when capacity suffices, otherwise adjusts capacity, reports it, and allocates eight bytes per element. |
| `00b8c730` | `InvertedInstanceArray::SwapStorage` | ✅ | virtual | On the ordinary branch swaps buffer and capacity-related fields with the other array object. |
| `00b8dd40` | `InvertedInstanceArray::Dtor` | ✅ | dtor | Restores the array vtable, zeros size, frees external storage, and clears the data pointer. |
| `00b8dd70` | `InvertedInstanceArray::ScalarDeletingDtor` | ✅ | dtor | Destroys array storage and frees the object when scalar-delete flag bit zero is set. |
| `00b8e900` | `InvertedInstanceArray::DtorBody` | ✅ | dtor | Restores array vtable, clears size, frees the data buffer, and clears the data pointer. |
| `00b8edc0` | `InvertedInstanceArray::Ctor` | ✅ | ctor | Installs the array vtable and initializes its buffer, size and capacity fields to zero. |
