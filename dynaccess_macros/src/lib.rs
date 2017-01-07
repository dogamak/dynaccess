//! This crate provides `#[derive(Dynaccess)]` that implements dynamic accessor
//! methods for the struct using `FieldAccessors` trait in `dynaccess_traits`.
//! A module containing unit structs of which each represent one field in the
//! struct. These structs are passed to the methods of `FieldAccessors` to get
//! and modify the corresponding fields of the struct.
//!
//! # Example
//!
//! ```rust
//! #[feature(proc_macro)]
//! #[macro_use]
//! extern create dynaccess_macros;
//! extern crate dynaccess_traits;
//!
//! #[derive(Dynaccess)]
//! pub struct Person {
//!     pub age: u32,
//!     pub names: Vec<String>,
//! }
//!
//! fn main() {
//!     let someone = Person {
//!         age: 19,
//!         names: vec!["John".to_string()],
//!     };
//!
//!     assert_eq!(someone.get(field::Age), 19);
//!     someone.set(field::Age, 20);
//!     assert_eq!(someone.get(field::Age), 20);
//!
//!     someone.get_mut(field::Names).push("Smith");
//!     assert_eq!(someone.get(field::Names).join(" "), "John Smith");
//! }
//! ```

//#![feature(proc_macro, proc_macro_lib)]
#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate syn;
extern crate regex;
#[macro_use] extern crate quote;
#[macro_use] extern crate lazy_static;

use proc_macro::TokenStream;
use regex::{Regex, Captures};
use syn::{
    Attribute,
    Body,
    Field,
    Ident,
    IntTy,
    Lit,
    MacroInput,
    MetaItem,
    NestedMetaItem,
    Ty,
    VariantData,
};

// Generates AST for struct field
fn create_field_items(config: &MacroConfig,
                      field: &FieldInfo)
                      -> quote::Tokens
{
    let struct_ident = config.struct_ident.clone();
    let field_struct_ident = field.struct_ident.clone();
    let field_const_ident = field.const_ident.clone();
    let field_type = field.ty.clone();
    let field_ident = field.ident.clone();
    
    let mut field_attrs = config.global_attrs.clone();
    let mut field_specific_attrs = field.attrs.clone();
    field_attrs.append(&mut field_specific_attrs);
    
    quote!(
        #( #field_attrs )*
        pub struct #field_struct_ident;

        pub const #field_const_ident: #field_struct_ident = #field_struct_ident;
        
        impl ::dynaccess_traits::Field<#struct_ident> for #field_struct_ident {
            type Type = #field_type;
            
            fn get(s: &#struct_ident) -> &#field_type {
                &s.#field_ident
            }

            fn get_mut(s: &mut #struct_ident) -> &mut #field_type {
                &mut s.#field_ident
            }

            fn set(s: &mut #struct_ident, v: #field_type) {
                s.#field_ident = v
            }
        }
    )
}

// Searches attributes for `#[dynaccess(...)]` 
fn filter_dynaccess_attrs<'a, I>(attrs: I) -> Box<Iterator<Item=syn::NestedMetaItem> + 'a>
    where I: Iterator<Item=&'a Attribute> + 'a,
{
    Box::new(attrs.filter_map(|attr| {
        if let MetaItem::List(ref ident, ref attrs) = attr.value {
                if ident.to_string() == "dynaccess".to_string() {
                    return Some(attrs.clone());
                }
            }
        None
    }).flat_map(|a| a))
}

#[derive(Clone)]
struct FieldInfo {
    attrs: Vec<Attribute>,
    const_ident: Ident,
    ident: Ident,
    struct_ident: Ident,
    ty: Ty,
}

struct MacroConfig {
    fields: Vec<FieldInfo>,
    global_attrs: Vec<Attribute>,
    module_ident: Ident,
    struct_ident: Ident,
}

// Extracts information of a struct field from AST
// Performs ident case conversion
fn parse_field(_: &MacroConfig, field: &Field) -> FieldInfo {
    lazy_static! {
        static ref REGEX_SNAKE_CASE: Regex = Regex::new("(?:^|_)(.)").unwrap();
    }

    let field_name = field.ident.clone().unwrap().to_string();
    let camel_case = REGEX_SNAKE_CASE
        .replace_all(field_name.as_str(),
                     |captures: &Captures| {
                         captures.get(1).unwrap().as_str()
                             .to_string().to_uppercase()
                     });

    let struct_ident = Ident::from(format!("_{}", camel_case));
    let const_ident = Ident::from(camel_case);
    let ty = field.ty.clone();

    let iter = filter_dynaccess_attrs(field.attrs.iter())
        .filter_map(|item| match item {
            NestedMetaItem::MetaItem(item) => Some(item),
            _ => None,
        });

    let mut field_attrs = vec![];
    
    for attr in iter {
        match attr {
            MetaItem::List(ref name, ref attrs)
                if name.to_string() == "field_attrs".to_string() =>
            {
                let iter = attrs.iter().filter_map(|item| match item {
                    &NestedMetaItem::MetaItem(ref item) => Some(item),
                    _ => None,
                });
                
                for attr in iter {
                    field_attrs.push(Attribute {
                        style: syn::AttrStyle::Outer,
                        is_sugared_doc: false,
                        value: attr.clone()
                    });
                }
            },
            _ => ()
        }
    }
    
    FieldInfo {
        attrs: field_attrs,
        const_ident: const_ident,
        ident: field.ident.clone().unwrap(),
        struct_ident: struct_ident,
        ty: ty,
    }
}

// Creates `MacroConfig` from `MacroInput`.
// Extracts options and field information from the AST
fn parse_macro_config(input: &MacroInput) -> MacroConfig {
    let mut module_name = "field".to_string();
    let mut global_attrs = vec![];

    // Get only attributes under #[dynaccess(...)]
    let attrs = filter_dynaccess_attrs(input.attrs.iter());
    
    // Parse attributes
    for attr in attrs {
        match attr {
            // #[dynaccess(module = ...)]
            NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, Lit::Str(ref value, _)))
                if name.to_string() == "module".to_string() =>
            {
                module_name = value.to_string();
            },

            // #[dynaccess(field_attrs(...))]
            NestedMetaItem::MetaItem(MetaItem::List(ref name, ref attrs))
                if name.to_string() == "field_attrs".to_string() =>
            {
                let mut new_attrs = attrs.iter().filter_map(|item| {
                    match item {
                        &NestedMetaItem::MetaItem(ref item) => Some(item),
                        _ => None
                    }
                }).map(|item| {
                    Attribute {
                        style: syn::AttrStyle::Outer,
                        is_sugared_doc: false,
                        value: item.clone()
                    }
                }).collect();

                global_attrs.append(&mut new_attrs);
            },
            _ => ()
        }
    }

    let mut config = MacroConfig {
        module_ident: Ident::new(module_name),
        struct_ident: input.ident.clone(),
        global_attrs: global_attrs,
        fields: vec![]
    };

    // Parse fields
    if let Body::Struct(VariantData::Struct(ref fields)) = input.body {
        config.fields = fields.iter()
            .map(|field| parse_field(&config, field))
            .collect();
    } else {
        panic!("#[derive(Dynaccess)] is only defined for structs")
    }
    
    config
}

// The macro.
#[proc_macro_derive(Dynaccess)]
pub fn dynaccess(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).expect("failed to parse macro input");

    let config = parse_macro_config(&ast);

    let module_ident = config.module_ident.clone();
    let struct_ident = config.struct_ident.clone();

    let field_count = Lit::Int(config.fields.len() as u64, IntTy::Usize);
    let field_struct_idents = config.fields.iter()
        .map(|field| field.struct_ident.clone());

    let field_items = config.fields.iter()
        .map(|field| create_field_items(&config, field));
    
    let gen = quote!(
        #[allow(unused_variables, non_upper_case_globals)]
        mod #module_ident {
            use super::#struct_ident;
            
            impl ::dynaccess_traits::FieldAccessors for #struct_ident {
               fn set<F, V>(&mut self, field: F, value: V)
                    where F: ::dynaccess_traits::Field<#struct_ident, Type=V>,
                {
                    F::set(self, value)
                }

                fn get<F,V>(&self, field: F) -> &V
                    where F: ::dynaccess_traits::Field<#struct_ident, Type=V>,
                {
                    F::get(self)
                }

                fn get_mut<F,V>(&mut self, field: F) -> &mut V
                    where F: ::dynaccess_traits::Field<#struct_ident, Type=V>,
                {
                    F::get_mut(self)
                }
            }
            
            #( #field_items )*
        }
    );

    gen.parse().expect("failed to stringify the syntax tree")
}
