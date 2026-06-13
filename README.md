# Cap'n Proto Schema

**A Rust library for parsing and modeling Cap'n Proto schema definitions** — provides a programmatic representation of Cap'n Proto structs, fields, and type metadata for code generation and introspection.

## Why It Matters

Cap'n Proto is a ridiculously fast data interchange format and RPC system. Unlike Protocol Buffers (which need a separate encoding/decoding step), Cap'n Proto data is structured in memory exactly as it will appear on the wire — zero-copy serialization/deserialization. It's used in Sandstorm.io, Cloudflare Workers (internal), and many high-performance distributed systems.

Cap'n Proto schemas define structs with fields that have explicit ordinals (slot numbers), enabling forward/backward compatibility. The schema language also supports enums, interfaces (RPC methods), and annotations.

This library provides a Rust model of the schema's struct definitions — field names, types, ordinals, data word counts, and pointer counts — enabling tooling that introspects or generates code from `.capnp` files.

## How It Works

**`CapnpStruct`**: Models a Cap'n Proto struct with:
- **data_word_count**: Number of 8-byte data words (holds primitive fields)
- **pointer_count**: Number of pointer words (holds text, list, and struct references)
- **fields**: Each field has a name, slot type, and ordinal (position)

The total size of a struct in bytes is `(data_word_count + pointer_count) × 8`. Cap'n Proto groups fields into data section (primitives packed by alignment) and pointer section (references to variable-size data).

**Schema ID parsing**: Cap'n Proto assigns each struct a unique 64-bit ID (e.g., `@0xdeadbeefcafebabe`) for binary compatibility. `parse_capnp_id()` extracts this from a schema declaration line.

## Quick Start

```rust
use capnp_schema::{CapnpStruct, parse_capnp_id};

// Model a simple struct
let mut person = CapnpStruct::new("Person");
person.data_word_count = 2;   // 16 bytes of primitives (age: u32, id: u64)
person.pointer_count = 2;     // 16 bytes of pointers (name: Text, email: Text)
person.fields.push(capnp_schema::CapnpField {
    name: "name".into(),
    slot_type: "Text".into(),
    ordinal: 0,
});

println!("Person struct size: {} bytes", person.total_size_bytes());

// Parse a schema declaration
let id = parse_capnp_id("struct Person @0xa1b2c3d4e5f6a7b8 {");
assert_eq!(id, Some(0xa1b2c3d4e5f6a7b8));
```

## API

- **`CapnpField`** — name, slot_type, ordinal
- **`CapnpStruct`** — Struct model: `new(name)`, `total_size_bytes()`
- **`parse_capnp_id(line)` → `Option<u64>`** — Extract schema ID from declaration text

## Architecture Notes

Provides the schema introspection layer for SuperInstance's Cap'n Proto code generation. Used in build-time tooling to analyze `.capnp` schema files and generate optimized Rust bindings. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
