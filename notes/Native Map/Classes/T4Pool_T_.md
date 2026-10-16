# Class: T4Pool<T>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00e76000` | `— ⚠reject` | ⚪ | method | thiscall, value in stack arg 1: allocates an 8-byte node through 00e611f0 (line 0x8a for the first node, 0x94 otherwise), sets its value and next link to 0. Empty pool (head at [this] is NULL): the node becomes head, ta… |
| `00e760b0` | `— ⚠reject` | ⚪ | method | thiscall, value in stack arg 1: byte-identical body to 00e76000 (T4Pool<T> push-back: tracked 8-byte node allocation with lines 0x8a/0x94, head/tail/end/count updates). |
