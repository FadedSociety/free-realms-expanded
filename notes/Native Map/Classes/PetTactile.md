# Class: PetTactile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cce490` | `PetTactile::ReleaseSpotEffects` | 🟢 | method | For each grooming-map node, releases the two associated effect handles, then clears the grooming spot map. |
| `00cce620` | `PetTactile::Dtor ⚠downgrade` | 🟢 | dtor | Restores PetTactile vtable, clears the shared pet panel state when active, then restores and clears its embedded grooming map. |
| `00cce850` | `PetTactile::Ctor` | ✅ | ctor | Stores two constructor arguments, installs PetTactile vtable, clears tactile fields and initializes an embedded grooming spot hash-list map. |
| `00cce940` | `PetTactile::ScalarDeletingDtor` | ✅ | dtor | Destroys the PetTactile object and frees its allocation when the deleting flag is set. |
| `00cce960` | `PetTactile::RefreshSpots` | 🟢 | method | Clears the spot map, resolves current pet tactile data, enumerates configured spots, inserts their ids and positions, and updates the activity distance state. |
