# Class: ProjectileParameters

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d00e0` | `ProjectileParameters::ProjectileParameters` | ✅ | ctor | Constructor of the ProjectileParameters record: installs ProjectileParameters::vftable, zeroes the field block +0x10..+0x1c, stores the global float uVar1 at +0x20/+0x24, sets +0x28 = 1, installs two embedded Target::vf… |
| `008d0240` | `ProjectileParameters::~ProjectileParameters` | ✅ | dtor | Destructor of the ProjectileParameters record: rewrites ProjectileParameters::vftable, releases the two embedded SoeUtil::IString members at +0x94 and +0x84 (capacity checks + locked refcount decrement + virtual slot +8… |
| `008d0310` | `ProjectileParameters::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the ProjectileParameters destructor body FUN_008d0240 and frees the object with _free when bit 0 of the flag argument is set. |
| `008eb0a0` | `ProjectileParametersCopyCtor` | ✅ | ctor | Copies projectile scalars, initializes two Target members, copies two strings and remaining parameter fields. |
