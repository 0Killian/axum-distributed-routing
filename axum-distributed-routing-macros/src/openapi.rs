/*use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Type, parse_macro_input};

struct DocComments {
    description: Option<String>,
}

impl DocComments {
    fn from_attrs(attrs: &[syn::Attribute]) -> Self {
        let mut description = None;
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Ok(meta) = attr.meta.require_name_value() {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            let doc_comment = lit_str.value().trim().to_string();
                            if !doc_comment.is_empty() {
                                description = Some(doc_comment);
                                break;
                            }
                        }
                    }
                }
            }
        }
        Self { description }
    }
}

#[proc_macro_derive(OpenApiSchema)]
pub fn openapi_schema_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let schema_title = name.to_string();
    let doc_comments = DocComments::from_attrs(&input.attrs);
    let description = doc_comments.description.unwrap_or_default();

    let (kind, properties_map, enum_variants, items_type) = match input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut properties = HashMap::new();
            let required_fields = Vec::new();

            for field in fields.iter() {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = &field.ty;

                let field_doc_comments = DocComments::from_attrs(&field.attrs);
                let field_description = field_doc_comments.description;

                // TODO: serde attributes (ex: #[serde(rename = "custom_name")])

                let is_option = is_option_type(field_type);
                if !is_option {
                    required_fields.push(field_name.clone());
                }

                let field_schema_tokens = generate_field_schema(field_type);
                properties.insert(field_name.to_string(), field_schema_tokens)
            }

            let properties_quote = if properties.is_empty() {
                quote! { std::collections::HashMap::new() }
            } else {
                let keys = properties.keys();
                let values = properties.values();
                quote! {
                    std::collections::HashMap::from([
                        #( (#keys.to_string(), #values) ),*
                    ])
                }
            };

            (
                quote! { "object" },
                properties_quote,
                quote! { None },
                quote! { None },
            )
        }
        syn::Data::Enum(data_enum) => {
            let variants = data_enum
                .variants
                .iter()
                .map(|v| v.ident.to_string())
                .collect::<Vec<_>>();
            ("enum", vec![], Some(variants), None)
        }
        syn::Data::Union(_) => panic!("Unions are not supported"),
    };
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }
    false
}
*/
