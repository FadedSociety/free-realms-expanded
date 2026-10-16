# Class: SoeUtil::RefPointer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a87440` | `— ⚠reject` | ⚪ | method | Reference-counted pointer assignment: if the destination already holds an object it first runs the object's vtable slot [0x28] with argument 0 and then 0081ad80 (the release path), stores param_2 as the new pointer, and… |
| `00a874d0` | `— ⚠reject` | ⚪ | method | Second identical instantiation of the reference-counted pointer assignment (release old via vtable slot 0x28 plus 0081ad80, store param_2, addref with 0081ce10, drop the temporary reference through the count word at [pt… |
