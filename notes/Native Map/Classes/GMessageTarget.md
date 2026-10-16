# Class: GMessageTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00428cbb` | `GMessageTarget::GMessageTarget` | ✅ | ctor | Base constructor of the GFC message-target class: runs the GObjectBase base constructor 004d99d1, stores the GMessageTarget interface vtable into both +0 and +4, and zeroes the three dwords at +0x18, +0x1C and +0x20; re… |
| `00428d3a` | `GMessageTarget::GMessageTargetB` | ✅ | ctor | Second constructor overload of the GFC message-target class: runs the GObjectBase base constructor 004d99d1, installs the GMessageTarget vtable at +0 and +4, zeroes +0x18/+0x1C/+0x20, and then attaches the source argume… |
| `00429a8a` | `CtorVtables175D4C8B` | ✅ | ctor | Message-target constructor of the 0x175d4c8 control class: runs the GObjectBase base constructor 004d99d1, installs the GMessageTarget vtable at +0/+4, zeroes the fields at +0x18..+0x20, attaches the source through 004d… |
| `0042a9bb` | `CtorVtables175D9E8B ⚠downgrade` | 🟢 | ctor | Message-target constructor of the 0x175d9e8 control class: installs the GMessageTarget vtable at +0/+4, zeroes the link fields, attaches the source through 004d9a26, overwrites +0/+4 with 0x175d9e8/0x175d9e4, copies sou… |
