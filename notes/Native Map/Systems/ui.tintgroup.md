# System: ui.tintgroup

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a72780` | `TintGroupDataSource::ColumnTitle` | TintGroupDataSource | Column header text for the tint-group data source. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a73610` | `TintGroupDataSource::DtorBody` | TintGroupDataSource | Destructor body of TintGroupDataSource: releases its two node maps and the holder object. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a736c0` | `TintGroupDataSource::Ctor` | TintGroupDataSource | Constructor of TintGroupDataSource: builds its tint-group maps and marks them dirty so the first access rebuilds them. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a73760` | `TintGroupDataSource::ScalarDeletingDtor` | TintGroupDataSource | Scalar deleting destructor of TintGroupDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a733e0` | `— ⚠reject` | TintGroupDataSource | Virtual slot [2]. If the byte at this+0x228 is zero it calls 00a732e0 with ECX = this-0x30. Then it sets ECX = this+4 and tail-jumps to 010c8f30 (jmp rel32), so the callee result is returned to the caller. Does not set … | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a73400` | `— ⚠reject` | TintGroupDataSource | Cell accessor for the tint-group data source (row, column, out). | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

