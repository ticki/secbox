# SecBox -- Sensitive data container.

I found myself reimplementing this piece in different projects, so I decided to
make it a library.

`secbox` provides a primitive which tries to harden protection of the inner
data, preventing certain attack vectors. It can be used as building block to
primitives like `SecStr`.

The docs detail the methods used.

This is useful for storing sensitive data like passwords and private keys.

## Cargo.toml

```toml
secbox = "0.1.0"
```

## Example

```rust
// We box the vector, despite only being a container. Techinically, this is
// unnecessary, but it improves the security slightly.
let mut pass = SecBox::new(Vec::new());

for i in ::std::io::stdin().chars() {
    match i {
        // Stop on enter.
        '\n' => break,
        // We SecBox it and the push it. SecBoxing it here will protect the
        // data through a variety of measures.
        i => pass.push(SecBox::new(i)),
    }
}
```
