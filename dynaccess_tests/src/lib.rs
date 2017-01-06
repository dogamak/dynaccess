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

    #[derive(Dynaccess)]
    #[dynaccess(module = "dog_field")]
    struct Dog {
        pub name: String
    }

    #[test]
    fn test_module_name() {
        let d = Dog {
            name: "doge".to_string()
        };

        use tests::dog_field;

        assert_eq!(d.get(dog_field::Name), "doge");
    }

    #[derive(Dynaccess)]
    #[dynaccess(field_attrs(derive(Clone)), module="sheep_field")]
    struct Sheep {
        pub name: String
    }

    #[test]
    pub fn test_field_attrs() {
        let s = Sheep {
            name: "Dolly".to_string()
        };

        assert_eq!(s.get(sheep_field::Name.clone()), &"Dolly".to_string());
    }

    #[derive(Dynaccess)]
    #[dynaccess(module="snowflake_field")]
    struct Snowflake {
        #[dynaccess(field_attrs(derive(Clone)))]
        pub id: usize,
        #[dynaccess(field_attrs(derive(Debug)))]
        pub is_unique: bool
    }

    #[test]
    pub fn test_individual_field_attrs() {
        let snow = Snowflake {
            id: 23956532,
            is_unique: true
        };

        snowflake_field::Id.clone();
        println!("{:?}", snowflake_field::IsUnique);
    }
}
