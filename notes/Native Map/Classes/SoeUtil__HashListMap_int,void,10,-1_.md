# Class: SoeUtil::HashListMap<int,void,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3c1a0` | `HashListMap10_InsertNode` | 🟢 | method | Inserts a caller-allocated node into a 10-bucket hash map: links it at the tail (prev at node+8 from map+8, next link written through +0xc, map tail updated at +8), stores the key at node+4, pushes the node onto the buc… |
| `00a3d170` | `— ⚠reject` | ⚪ | method | Erase of the node held at container+4 from the 10-bucket map: selects the bucket by (node key at +4) % 10 within the bucket heads at container+0x10, walks the +0 chain and splices the node out, splices it out of the ord… |
