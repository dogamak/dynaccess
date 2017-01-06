#![feature(proc_macro, custom_attribute)]
#[macro_use]
extern crate dynaccess_macros;
extern crate dynaccess_traits;

#[cfg(test)]
mod tests {
    use dynaccess_traits::FieldAccessors;
    
    #[derive(FieldModule)]
    struct Struct {
        pub field: bool,
        pub name: String
    }
    
    #[test]
    fn it_works() {
        let mut s = Struct {
            field: false,
            name: "Hello".to_string()
        };

        use tests::field;
        
        s.set(field::Field, true);
        s.get_mut(field::Name).push_str(" World!");

        assert!(s.field);
        assert_eq!(s.name, "Hello World!");
    }
}
