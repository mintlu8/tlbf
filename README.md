# tlbf

Type level bitflags.

## Example

```rust
tlbf!(
    pub Color: u64 {
        Red, 
        Green, 
        Blue,
    }
);
```
