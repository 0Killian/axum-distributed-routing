use std::{collections::HashMap, str::FromStr};

use syn::{
    Block, Field, Ident, LitStr, PatType, Token, Type,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Either a block with members or a type name
enum TypeNameOrDef {
    Type(Type),
    Def(Punctuated<Field, Token![,]>),
}

enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
}

struct Args {
    path: String,
    path_params: HashMap<Ident, Type>,
    query_params: Option<TypeNameOrDef>,
    body_params: Option<TypeNameOrDef>,
    parameters: Punctuated<PatType, Token![,]>,
    name: Ident,
    group: Type,
    return_type: Type,
    method: Method,
    is_async: bool,
    handler: Block,
}

impl Parse for TypeNameOrDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Brace) {
            let content;
            let _ = syn::braced!(content in input);
            Ok(TypeNameOrDef::Def(Punctuated::parse_terminated_with(
                &content,
                Field::parse_named,
            )?))
        } else {
            Ok(TypeNameOrDef::Type(input.parse()?))
        }
    }
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut path_params = HashMap::new();
        let mut query_params = None;
        let mut body_params = None;
        let mut parameters: Punctuated<PatType, Token![,]> = Punctuated::new();
        let mut name = None;
        let mut return_type = None;
        let mut handler = None;
        let mut is_async = false;
        let mut method = None;
        let mut group = None;

        while !input.is_empty() {
            let ident: Ident = input.call(Ident::parse_any)?;

            match ident.to_string().as_str() {
                "method" => {
                    // Expects equal sign
                    input.parse::<syn::Token![=]>()?;

                    let method_ident = input.parse::<Ident>()?;
                    match method_ident.to_string().as_str() {
                        "GET" => method = Some(Method::Get),
                        "POST" => method = Some(Method::Post),
                        "PUT" => method = Some(Method::Put),
                        "PATCH" => method = Some(Method::Patch),
                        "DELETE" => method = Some(Method::Delete),
                        "HEAD" => method = Some(Method::Head),
                        "OPTIONS" => method = Some(Method::Options),
                        "TRACE" => method = Some(Method::Trace),
                        "CONNECT" => method = Some(Method::Connect),
                        m => {
                            return Err(syn::Error::new(
                                proc_macro2::Span::call_site(),
                                format!("Unknown method {}", m),
                            ));
                        }
                    }
                }
                "group" => {
                    // Expects equal sign
                    input.parse::<syn::Token![=]>()?;

                    group = Some(input.parse()?);
                }
                "path" => {
                    // Expects equal sign
                    input.parse::<syn::Token![=]>()?;

                    let path_str: LitStr = input.parse()?;
                    let (path_, path_params_) = Self::parse_path(path_str.value())?;
                    path = Some(path_);
                    path_params = path_params_;
                }
                "query" => {
                    // Expects equal sign
                    input.parse::<syn::Token![=]>()?;

                    query_params = Some(input.parse::<TypeNameOrDef>()?);
                }
                "body" => {
                    // Expects equal sign
                    input.parse::<syn::Token![=]>()?;

                    body_params = Some(input.parse::<TypeNameOrDef>()?);
                }
                _ => {
                    if ident.to_string().as_str() == "async" {
                        is_async = true;
                        name = Some(input.parse()?);
                    } else {
                        name = Some(ident);
                    }

                    if input.peek(syn::token::Paren) {
                        let content;
                        let _ = parenthesized!(content in input);
                        parameters = Punctuated::parse_terminated(&content)?;
                    }

                    if input.peek(Token![->]) {
                        input.parse::<Token![->]>()?;
                        return_type = Some(input.parse()?);
                    }
                    handler = Some(input.parse()?);
                }
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        if path.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing path",
            ));
        }

        if name.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing name",
            ));
        }

        if return_type.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing return type",
            ));
        }

        if handler.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing handler",
            ));
        }

        if method.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing method",
            ));
        }

        if group.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing group",
            ));
        }

        Ok(Args {
            name: name.unwrap(),
            return_type: return_type.unwrap(),
            is_async,
            group: group.unwrap(),
            method: method.unwrap(),
            handler: handler.unwrap(),
            path: path.unwrap(),
            path_params,
            query_params,
            body_params,
            parameters,
        })
    }
}

#[derive(PartialEq)]
enum ParsePathState {
    Path,
    PathParamName,
    PathParamType,
}

impl Args {
    fn parse_path(path: String) -> syn::Result<(String, HashMap<Ident, Type>)> {
        let mut real_path = String::new();
        let mut path_params = HashMap::new();
        let mut state = ParsePathState::Path;
        let mut current_name = String::new();
        let mut current_type = String::new();

        for c in path.chars() {
            match c {
                '{' => {
                    if state == ParsePathState::Path {
                        state = ParsePathState::PathParamName;
                    } else {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "Invalid path",
                        ));
                    }
                }
                '}' => {
                    if state == ParsePathState::PathParamType {
                        let param_name = proc_macro2::TokenStream::from_str(&current_name)
                            .map_err(|_| {
                                syn::Error::new(proc_macro2::Span::call_site(), "Invalid path")
                            })?;
                        let param_type = proc_macro2::TokenStream::from_str(&current_type)
                            .map_err(|_| {
                                syn::Error::new(proc_macro2::Span::call_site(), "Invalid path")
                            })?;
                        path_params.insert(syn::parse2(param_name)?, syn::parse2(param_type)?);

                        real_path.push(':');
                        real_path.push_str(&current_name);

                        current_name = String::new();
                        current_type = String::new();
                        state = ParsePathState::Path;
                    } else {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "Invalid path",
                        ));
                    }
                }
                ':' => {
                    if state == ParsePathState::PathParamName {
                        state = ParsePathState::PathParamType;
                    } else {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "Invalid path",
                        ));
                    }
                }
                _ => match state {
                    ParsePathState::Path => {
                        real_path.push(c);
                    }
                    ParsePathState::PathParamName => {
                        current_name.push(c);
                    }
                    ParsePathState::PathParamType => {
                        current_type.push(c);
                    }
                },
            }
        }

        if state != ParsePathState::Path {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Invalid path",
            ));
        }

        Ok((path, path_params))
    }
}

/// Creates a route and add it to the group
///
/// # Example
/// ```
/// route!(
///     group = Routes,
///     path = "/echo/{str:String}",
///     method = GET,
///     async test_fn -> String { str }
/// );
/// ```
#[proc_macro]
pub fn route(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO: cleanup
    let args = parse_macro_input!(attr as Args);

    let path_params = args.path_params;
    let path_idents = path_params.keys().collect::<Vec<_>>();
    let path_types = path_params.values().collect::<Vec<_>>();

    let path_params = if path_params.is_empty() {
        quote::quote! {}
    } else {
        quote::quote! {
            axum::extract::Path((#(#path_idents),*)): axum::extract::Path<(#(#path_types),*)>,
        }
    };

    let (query_params_def, query_params) = if let Some(q) = args.query_params {
        match q {
            TypeNameOrDef::Type(t) => (
                quote::quote! {},
                quote::quote! { axum::extract::Query(query): axum::extract::Query<#t>, },
            ),
            TypeNameOrDef::Def(d) => {
                let def_name = Ident::new(
                    format!(
                        "{}QueryParams",
                        stringcase::pascal_case(args.name.to_string().as_str())
                    )
                    .as_str(),
                    proc_macro2::Span::call_site(),
                );
                (
                    quote::quote! {
                        #[derive(serde::Deserialize)]
                        struct #def_name #d
                    },
                    quote::quote! { axum::extract::Query(query): axum::extract::Query<#def_name>, },
                )
            }
        }
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let (body_params_def, body_params) = if let Some(b) = args.body_params {
        match b {
            TypeNameOrDef::Type(t) => (
                quote::quote! {},
                quote::quote! { axum::extract::Form(body): axum::extract::Form<#t>, },
            ),
            TypeNameOrDef::Def(d) => {
                let def_name = Ident::new(
                    format!(
                        "{}BodyParams",
                        stringcase::pascal_case(args.name.to_string().as_str())
                    )
                    .as_str(),
                    proc_macro2::Span::call_site(),
                );
                (
                    quote::quote! {
                        #[derive(serde::Deserialize)]
                        struct #def_name #d
                    },
                    quote::quote! { axum::extract::Form(body): axum::extract::Form<#def_name>, },
                )
            }
        }
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let route_name = Ident::new(
        &format!(
            "ROUTE_{}",
            stringcase::macro_case(args.name.to_string().as_str())
        ),
        proc_macro2::Span::call_site(),
    );
    let path = args.path;
    let parameters = args.parameters;
    let return_type = args.return_type;
    let block = args.handler;
    let group = args.group;

    let async_keyword = if args.is_async {
        quote::quote! { async }
    } else {
        quote::quote! {}
    };

    let handler = quote::quote! {
        #async_keyword |#path_params #query_params #body_params #parameters| -> #return_type #block
    };

    let handler = match args.method {
        Method::Get => quote::quote! { axum::routing::get(#handler) },
        Method::Post => quote::quote! { axum::routing::post(#handler) },
        Method::Put => quote::quote! { axum::routing::put(#handler) },
        Method::Patch => quote::quote! { axum::routing::patch(#handler) },
        Method::Delete => quote::quote! { axum::routing::delete(#handler) },
        Method::Head => quote::quote! { axum::routing::head(#handler) },
        Method::Options => quote::quote! { axum::routing::options(#handler) },
        Method::Trace => quote::quote! { axum::routing::trace(#handler) },
        Method::Connect => quote::quote! { axum::routing::connect(#handler) },
    };

    let result = quote::quote! {
        #query_params_def
        #body_params_def

        pub static #route_name: #group = #group::new(#path, |r, _| r.route(#path, #handler));

        axum_distributed_routing::inventory::submit! {
            #route_name
        }
    };

    result.into()
}
