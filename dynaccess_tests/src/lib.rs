#![feature(proc_macro, custom_attribute)]
#[macro_use]
extern crate dynaccess_macros;
extern crate dynaccess_traits;

#[cfg(test)]
mod tests {
    use dynaccess_traits::FieldAccessors;
    
    #[derive(Dynaccess)]
    struct Struct {
        pub field: bool,
        pub name: String
    }

    #[derive(Dynaccess)]
    #[dynaccess(module = "dog_field")]
    struct Dog {
        pub name: String
    }

    #[test]
    fn test_basic() {
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

    #[test]
    fn test_module_name() {
        let d = Dog {
            name: "doge".to_string()
        };

        use tests::dog_field;

        assert_eq!(d.get(dog_field::Name), "doge");
    }
}
