# Class: NotificationQueue

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb7640` | `NotificationQueue::GiveTime` | ✅ | method | Fastcall on receiver p: calls 00769710 and returns when the object it yields has byte +4 +0x959 nonzero; computes elapsed = now (007027f0 result) minus the stamp at p+0x1a8/0x1ac, clamped to 0x7fffffff. If p+0x1b0 is cl… |
