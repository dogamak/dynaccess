#![feature(proc_macro, proc_macro_lib, custom_attribute)]
#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate syn;
extern crate regex;
#[macro_use] extern crate quote;
#[macro_use] extern crate lazy_static;

use proc_macro::TokenStream;
use regex::{Regex, Captures};
use syn::{MetaItem, Ident, Lit, Body, VariantData};


fn create_field_items(input: &syn::MacroInput, field: &syn::Field) -> quote::Tokens {
    lazy_static! {
        static ref REGEX_SNAKE_CASE: Regex = Regex::new("(?:^|_)(.)").unwrap();
    }
    let struct_name = input.ident.clone();
    let field_name = field.ident.clone().unwrap().to_string();
    let field_ident = field.ident.clone();
    let field_type = field.ty.clone();

    let field_name_camel_case = REGEX_SNAKE_CASE
        .replace(field_name.as_str(),
                 |captures: &Captures| {
                     captures.get(1).unwrap().as_str()
                         .to_string().to_uppercase()
                 });
    
    let field_struct_name = Ident::new(format!("_{}", field_name_camel_case));
    let field_const_name = Ident::new(field_name_camel_case);
    
    quote!(
        pub struct #field_struct_name;

        pub const #field_const_name: #field_struct_name = #field_struct_name;
        
        impl ::dynaccess_traits::Field<#struct_name> for #field_struct_name {
            type Type = #field_type;
            
            fn get(s: &#struct_name) -> &#field_type {
                &s.#field_ident
            }

            fn get_mut(s: &mut #struct_name) -> &mut #field_type {
                &mut s.#field_ident
            }

            fn set(s: &mut #struct_name, v: #field_type) {
                s.#field_ident = v
            }
        }
    )
}

#[proc_macro_derive(FieldModule)]
pub fn field_module(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).expect("failed to parse macro input");

    let mut mod_name = "field".to_string();

    for attr in ast.attrs.iter() {
        if let MetaItem::NameValue(ref name, Lit::Str(ref value, _)) = attr.value {
            if name.to_string() == "field_module".to_string() {
                mod_name = value.clone();
            }
        }
    }

    let mod_name = Ident::new(mod_name);
    
    let field_gens = if let Body::Struct(VariantData::Struct(ref fields)) = ast.body {
        fields.iter().map(|field| create_field_items(&ast, field)).collect::<Vec<_>>()
    } else {
        panic!("#[derive(FieldModule)] is only defined for structs");
    };

    let struct_name = ast.ident;
    
    let gen = quote!(
        mod #mod_name {
            use super::#struct_name;
            
            impl ::dynaccess_traits::FieldAccessors for #struct_name {
                fn set<F, V>(&mut self, field: F, value: V)
                    where F: ::dynaccess_traits::Field<#struct_name, Type=V>,
                {
                    F::set(self, value)
                }

                fn get<F,V>(&self, field: F) -> &V
                    where F: ::dynaccess_traits::Field<#struct_name, Type=V>,
                {
                    F::get(self)
                }

                fn get_mut<F,V>(&mut self, field: F) -> &mut V
                    where F: ::dynaccess_traits::Field<#struct_name, Type=V>,
                {
                    F::get_mut(self)
                }
            }
            
            #( #field_gens )*
        }
    );
    
    println!("{:?}", gen);

    gen.parse().expect("failed to stringify the syntax tree")
}
