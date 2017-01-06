# dynaccess
This crate provides `#[derive(Dynaccess)]` that implements dynamic accessor
methods for the struct using `FieldAccessors` trait in `dynaccess_traits`.
A module containing unit structs of which each represent one field in the
struct. These structs are passed to the methods of `FieldAccessors` to get
and modify the corresponding fields of the struct.

## Example

```rust
#[feature(proc_macro)]
#[macro_use]
extern create dynaccess_macros;
extern crate dynaccess_traits;

#[derive(Dynaccess)]
pub struct Person {
    pub age: u32,
    pub names: Vec<String>,
}

fn main() {
    let someone = Person {
        age: 19,
        names: vec!["John".to_string()],
    };

    assert_eq!(someone.get(field::Age), 19);
    someone.set(field::Age, 20);
    assert_eq!(someone.get(field::Age), 20);

    someone.get_mut(field::Names).push("Smith");
    assert_eq!(someone.get(field::Names).join(" "), "John Smith");
}
```
