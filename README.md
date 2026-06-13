# Cap'n Proto Schema

**A Rust library for parsing and modeling Cap'n Proto schema definitions** — provides a programmatic representation of Cap'n Proto structs, fields, and type metadata for code generation, introspection, and wire-format analysis.

## Why It Matters

Cap'n Proto is a ridiculously fast data interchange format and RPC system. Unlike Protocol Buffers (which encode/decode through a separate serialization step), Cap'n Proto data is structured in memory exactly as it will appear on the wire — **zero-copy serialization/deserialization**. There is no encoding step; the in-memory representation IS the on-wire representation. This makes it 3–4 orders of magnitude faster than Protobuf for large messages.

Cap'n Proto is used in:

- **Sandstorm.io** — secure web application platform (Cap'n Proto was originally developed for this)
- **Cloudflare Workers internal** — high-performance inter-service communication
- **Apache Arrow** — columnar in-memory analytics (uses Cap'n Proto for IPC in some configurations)
- **Machine learning pipelines** — tensor serialization with zero-copy

Cap'n Proto schemas define structs with fields that have **explicit ordinals** (slot numbers), enabling forward/backward compatibility. Adding a field with a new ordinal doesn't break older readers. Removing a field is safe if the ordinal isn't reused. This is the same evolutionary schema design Protocol Buffers popularized, but applied to a zero-copy wire format.

This library provides a Rust model of the schema's struct definitions — field names, types, ordinals, data word counts, and pointer counts — enabling tooling that introspects or generates code from `.capnp` files.

## How It Works

### Struct Layout: Data Section + Pointer Section

A Cap'n Proto struct in memory consists of two contiguous sections:

```
┌─────────────────────────────┐
│       Data Section           │  data_word_count × 8 bytes
│   (primitive fields: int,    │
│    float, bool, enum)        │
├─────────────────────────────┤
│     Pointer Section          │  pointer_count × 8 bytes
│   (references to: text,      │
│    data, lists, other        │
│    structs)                  │
└─────────────────────────────┘
```

**Total size in bytes:**

```
size = (data_word_count + pointer_count) × 8
```

The struct is packed for cache-line alignment. Primitives (integers, floats, bools) live in the data section, packed by alignment. References to variable-size data (strings, byte arrays, lists, nested structs) live in the pointer section as 8-byte pointers into a separate "capabilities" arena.

### Field Ordinals

Each field has an **ordinal** — a unique position number assigned at definition time. Ordinals are immutable: once published, a field's ordinal can never change (it would break wire compatibility). This is the schema evolution contract:

| Change | Safe? | Why |
|--------|-------|-----|
| Add field with new ordinal | ✅ | Old readers ignore unknown fields |
| Remove field | ✅ | Old writers leave gap; new readers see default |
| Rename field | ✅ | Only the name changes, ordinal is the same |
| Reuse a removed ordinal | ❌ | Confuses readers about the field type |
| Change field type at same ordinal | ❌ | Wire format mismatch |

### Schema ID Parsing

Each Cap'n Proto file has a globally unique 64-bit ID:

```
@0x8518ea84c2fbc4ed
```

The `parse_capnp_id()` function extracts this from a `@0x...` line:

```rust
let id = parse_capnp_id("@0x8518ea84c2fbc4ed");
assert_eq!(id, Some(0x8518ea84c2fbc4ed));
```

The ID is derived from a SHA-512 hash of the schema file content at creation time, truncated to 64 bits. It's used for message type identification on the wire.

### Memory Layout Mathematics

For a struct with data_word_count = D and pointer_count = P:

```
struct_size = (D + P) × 8 bytes
data_section_offset = 0
pointer_section_offset = D × 8
```

Field positions within each section are determined by alignment:

- `Bool` fields: 1 bit each, packed 64 per data word
- `Int32`/`Float32`: 4-byte aligned within data section
- `Int64`/`Float64`: 8-byte aligned, occupies one data word
- `Text`/`Data`/`List`/`Struct`: one pointer word (8 bytes) in pointer section

**Complexity:**

| Operation | Time | Space |
|-----------|------|-------|
| Parse struct definition | O(F) where F = fields | O(F) |
| Compute total size | O(1) | O(1) |
| Parse schema ID | O(L) where L = line length | O(1) |
| Field lookup by ordinal | O(F) linear / O(1) with index | O(F) |

## Quick Start

```rust
use capnp_schema::{CapnpStruct, CapnpField, parse_capnp_id};

// Define a Cap'n Proto struct model
let mut person = CapnpStruct::new("Person");
person.data_word_count = 2;   // 16 bytes for primitives
person.pointer_count = 1;     // 8 bytes for pointers
person.fields.push(CapnpField {
    name: "id".into(),
    slot_type: "Int64".into(),
    ordinal: 0,
});
person.fields.push(CapnpField {
    name: "name".into(),
    slot_type: "Text".into(),
    ordinal: 0,  // pointer section, ordinal independent
});

assert_eq!(person.total_size_bytes(), 24);  // (2 + 1) × 8

// Parse a schema ID
let id = parse_capnp_id("@0x8518ea84c2fbc4ed");
assert_eq!(id, Some(0x8518ea84c2fbc4ed));
```

## API

| Type | Fields/Methods | Description |
|------|---------------|-------------|
| `CapnpField` | `name: String`, `slot_type: String`, `ordinal: u16` | A struct field definition |
| `CapnpStruct` | `name`, `data_word_count`, `pointer_count`, `fields` | A Cap'n Proto struct |
| `CapnpStruct::new` | `(name: &str) → Self` | Create a new struct definition |
| `CapnpStruct::total_size_bytes` | `(&self) → usize` | `(data_words + pointer_words) × 8` |
| `parse_capnp_id` | `(&str) → Option<u64>` | Extract the `@0x...` ID from a schema line |

## Architecture Notes

Part of the SuperInstance serialization toolchain. Pairs with `archive-writer` for packaging and `ast-diff` for schema evolution detection.

Within γ + η = C, the Cap'n Proto struct layout instantiates the conservation law as **wire-format conservation**: the total struct size is strictly determined by data_word_count + pointer_count, and the field layout within those words is immutable once published. Agent contributions (γ) add fields with new ordinals; the environment (η) must maintain backward compatibility with older readers. The conserved quantity (C) is the wire representation — it can grow but never break existing consumers.

See the [architecture overview](https://github.com/casey-digennaro/capnp-schema/blob/main/ARCHITECTURE.md).

## References

1. Varda, K. (2013). "Cap'n Proto: Cap'n Proto Encoding Spec." *capnproto.org*. (Authoritative spec by the Protobuf author)
2. Varda, K. (2012). "Cap'n Proto vs. Protocol Buffers: A Benchmark." *capnproto.org*.
3. Kleppmann, M. (2017). *Designing Data-Intensive Applications*. O'Reilly. Chapter 4. (Schema evolution comparison)
4. Apache Arrow. "Arrow Memory Layout." *arrow.apache.org*. (Related columnar zero-copy format)

## License

MIT
