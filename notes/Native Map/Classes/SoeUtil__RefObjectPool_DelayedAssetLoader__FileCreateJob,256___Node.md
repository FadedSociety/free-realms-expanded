# Class: SoeUtil::RefObjectPool<DelayedAssetLoader::FileCreateJob,256>::Node

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae5d60` | `SoeUtil::RefObjectPool::FileCreateJobNode::Ctor` | ✅ | ctor | Constructs the embedded FileCreateJob, installs the pool-node vtable, stores the node value at +0x9e, and returns the destination. |
| `00ae6320` | `SoeUtil::RefObjectPool::FileCreateJobNode::ScalarDeletingDtor` | ✅ | dtor | Calls the embedded FileCreateJob destructor, frees the node when flag bit 0 is set, and returns the original pointer otherwise. |
| `00ae6990` | `SoeUtil::RefObjectPool::FileCreateJobNode::ReleaseToPool` | ✅ | virtual | Calls the embedded object vtable slot 0 with value 0, then appends this node to the pool deque at +0x278. |
