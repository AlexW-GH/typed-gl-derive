#![recursion_limit = "128"]
extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use proc_macro2::TokenStream;
use syn::Data;
use syn::Fields;
use syn::FieldsNamed;
use syn::Type;
use syn::Expr;
use proc_macro2::Ident;


#[proc_macro_derive(IsVertex)]
pub fn is_vertex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let size_fn = create_size_fn(&input.data);
    let type_fn = create_type_fn(&input.data);
    let ptr_fn = create_ptr_fn(&input.data);
    let field_pos_fn = create_field_pos_fn(&input.data);
    let name = input.ident;
    let expanded = quote! {
        use typedgl::gl_obj::VertexElementType;
        impl IsVertex for #name {
            fn element_size(&self, index: usize) -> i32 {
                #size_fn
            }

            fn element_type(&self, index: usize) -> VertexElementType {
                #type_fn
            }

            fn element_stride(&self) -> i32 {
                std::mem::size_of_val(self) as i32
            }

            fn element_pointer(&self, index: usize) ->  *const std::os::raw::c_void {
                #ptr_fn
            }

            fn field_position(&self, field_name: &str) -> usize {
                #field_pos_fn
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn create_size_fn(data: &Data) -> TokenStream{
    let field_lengths = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref field) => {
                    retrieve_array_lengths(field)
                }, _ => Vec::new()
            }
        }, _ => Vec::new()
    };
    let match_arms = map_to_length_match_arms(&field_lengths);
    quote! {
        match index {
            #(#match_arms)*
            _ => panic!("Invalid Index")
        }
    }
}

fn create_type_fn(data: &Data) -> TokenStream{
    let type_names = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    retrieve_field_types(fields)
                }, _ => Vec::new()
            }
        }, _ => Vec::new()
    };
    let match_arms = map_to_types_match_arms(&type_names);
    quote! {
        match index {
            #(#match_arms)*
            _ => panic!("Invalid Index")
        }
    }
}

fn create_ptr_fn(data: &Data) -> TokenStream{
    let field_names = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    retrieve_field_names(fields)
                }, _ => Vec::new()
            }
        }, _ => Vec::new()
    };
    let match_arms = map_to_ptr_match_arms(&field_names);
    quote! {
        match index {
            #(#match_arms)*
            _ => panic!("Invalid Index")
        }
    }
}

fn create_field_pos_fn(data: &Data) -> TokenStream{
    let field_names = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    retrieve_field_names(fields)
                }, _ => Vec::new()
            }
        }, _ => Vec::new()
    };
    let match_arms = map_to_field_pos_match_arms(&field_names);
    quote! {
        match field_name {
            #(#match_arms)*
            _ => panic!("Invalid Index")
        }
    }
}

fn retrieve_field_types(fields: &FieldsNamed) -> Vec<String>{
    use quote::ToTokens;
    fields.named.iter().map(|field| {
        let array_string: String = field.into_token_stream().into_iter()
            .filter(|tree| {
                match tree {
                    proc_macro2::TokenTree::Group(_) => {
                        true
                    },
                    _ => false
                }
            })
            .map(|outer_tree| {
                match outer_tree {
                    proc_macro2::TokenTree::Group(inner) => format!("{}", inner).to_owned(),
                    _ => String::new()
                }
            }).collect();
        let left_side: &str = array_string.split(";").collect::<Vec<&str>>().get(0).unwrap();
        let type_string: &str = left_side.split("[").collect::<Vec<&str>>().get(1).unwrap().trim();
        type_string.to_string()
    }).collect()
}

fn retrieve_array_lengths(fields: &FieldsNamed) -> Vec<Expr>{
    fields.named.iter().map(|named|{
        match &named.ty {
            Type::Array(type_array) => {
                type_array.len.clone()
            }
            _ => panic!("Only arrays are supported")
        }
    }).collect()
}

fn retrieve_field_names(fields: &FieldsNamed) -> Vec<Ident>{
    fields.named.iter().map(|named|{
        match &named.ident {
            Some(ident) => ident.clone(),
            None => panic!("Unnamed fields not allowed"),
        }
    }).collect()
}

fn map_to_types_match_arms(field_names: &[String]) -> Vec<TokenStream>{
    field_names.iter()
        .map(|name| {
            match name.as_str() {
                "u8" => quote! { VertexElementType::UnsignedByte },
                "i8" => quote! { VertexElementType::Byte },
                "u16" => quote! {VertexElementType::UnsignedShort },
                "i16" => quote! { VertexElementType::Short },
                "u32" => panic!("u32 not supported"),
                "i32" => panic!("i32 not supported"),
                "u64" => panic!("u64 not supported"),
                "i64" => panic!("i64 not supported"),
                "usize" => panic!("u32 not supported"),
                "isize" => panic!("u32 not supported"),
                "f32" => quote! { VertexElementType::Float },
                "f64" => panic!("u32 not supported by"),
                other =>  panic!("{} not supported", other),
            }})
        .enumerate()
        .map(|(index, name)| {
            quote!{
                #index => #name,
            }})
        .collect()
}

fn map_to_length_match_arms(field_lengths: &[Expr]) -> Vec<TokenStream>{
    field_lengths.iter()
        .enumerate()
        .map(|(index, len)| {
            quote!{
                #index => #len,
               }})
        .collect()
}

fn map_to_ptr_match_arms(field_names: &[Ident]) -> Vec<TokenStream>{
    let mem_sizes: Vec<TokenStream> = field_names.iter()
        .map(|field_name| {
            quote!{std::mem::size_of_val(&self.#field_name)}
        }).collect();
    let mut match_expressions: Vec<TokenStream> = Vec::new();
    for (index, _) in mem_sizes.iter().enumerate() {
        if index == 0 {
            match_expressions.push(quote!{ std::ptr::null::<std::os::raw::c_void>() })
        } else {
            let left_expression = match_expressions.get(index-1).unwrap();
            let right_expression = mem_sizes.get(index-1).unwrap();
            if index == 1 {
                match_expressions.push(
                    quote!{ #right_expression }
                );
            } else {
                match_expressions.push(
                    quote!{ ( #left_expression + #right_expression ) }
                );
            }

        }
    }
    match_expressions.iter()
        .enumerate()
        .map(|(index, _)| {
            let expression = match_expressions[index].clone();
            quote!{
                    #index => unsafe{std::mem::transmute( #expression )},
            }})
        .collect()
}

fn map_to_field_pos_match_arms(field_names: &[Ident]) -> Vec<TokenStream>{
    field_names.iter()
        .map(|field_name| field_name.to_string())
        .enumerate()
        .map(|(index, name)| {
            quote!{
                    #name => #index,
            }})
        .collect()
}