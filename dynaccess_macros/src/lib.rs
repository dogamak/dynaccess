#![feature(proc_macro, proc_macro_lib)]
#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate syn;
extern crate regex;
#[macro_use] extern crate quote;
#[macro_use] extern crate lazy_static;

use proc_macro::TokenStream;
use regex::{Regex, Captures};
use syn::{MetaItem, NestedMetaItem, Ident, Lit, Body, VariantData, Attribute};


fn create_field_items(input: &syn::MacroInput,
                      field_attrs: &Vec<Attribute>,
                      field: &syn::Field)
                      -> quote::Tokens
{
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
        #( #field_attrs )*
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

#[proc_macro_derive(Dynaccess)]
pub fn dynaccess(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).expect("failed to parse macro input");

    let attrs = filter_dynaccess_attrs(ast.attrs.iter());

    let mut mod_name = "field".to_string();
    let mut field_attrs = vec![];
    
    for attr in attrs {
        match attr {
            NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, Lit::Str(ref value, _)))
                if name.to_string() == "module".to_string() =>
            {
                mod_name = value.to_string();
            },
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

                field_attrs.append(&mut new_attrs);
            },
            _ => ()
        }
    }
    
    let mod_name = Ident::new(mod_name);
    
    let field_gens = if let Body::Struct(VariantData::Struct(ref fields)) = ast.body {
        fields.iter().map(|field| {
            create_field_items(&ast, &field_attrs, field)
        }).collect::<Vec<_>>()
    } else {
        panic!("#[derive(FieldModule)] is only defined for structs");
    };

    let struct_name = ast.ident;
    
    let gen = quote!(
        #[allow(unused_variables, non_upper_case_globals)]
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

    gen.parse().expect("failed to stringify the syntax tree")
}
