# Class: DelayedAssetLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae8100` | `DelayedAssetLoader::OnFileCreateJobComplete` | ✅ | virtual | Finds the tracked request by job ID, logs a failed save when the job flag is false, notifies the status handler with success or failure, parses a successful list result when marked as a list, and removes the tracked req… |
| `00ae8360` | `DelayedAssetLoader::Ctor` | ✅ | ctor | Installs the asset-delivery and job-queue base vtables, initializes flags and counters, constructs the pending deque, pool, tracked-asset list, and hash table, then returns the destination. |
| `00ae8450` | `DelayedAssetLoader::AdjustThisMinus8ThenScalarDelete` | ✅ | virtual | Subtracts 8 from ECX and jumps to the scalar deleting destructor at 00ae85b0. |
| `00ae8460` | `DelayedAssetLoader::Destroy` | ✅ | dtor | Restores the loader vtables, resets active state, frees the auxiliary buffer, unregisters the callback, clears the hash table and string, destroys the pending deque and tracked-asset map, then destroys the job-queue bas… |
| `00ae85b0` | `DelayedAssetLoader::ScalarDeletingDtor` | ✅ | dtor | Calls the DelayedAssetLoader destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |
