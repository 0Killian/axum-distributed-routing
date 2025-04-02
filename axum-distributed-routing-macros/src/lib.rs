use std::{collections::HashMap, str::FromStr};

use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, Attribute, Block, Ident, LitStr, PatType, Token, Type
};

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
    query_params: Option<Type>,
    body_params: Option<Type>,
    parameters: Punctuated<PatType, Token![,]>,
    name: Ident,
    group: Type,
    return_type: Type,
    method: Method,
    handler_attributes: Vec<Attribute>,
    handler: Block,
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
        let mut method = None;
        let mut group = None;
        let mut handler_attributes = Vec::new();

        while !input.is_empty() {
            if input.peek(Token![#]) || input.peek(Token![async]) {
                if handler.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "Handler is already defined",
                    ));
                }

                handler_attributes = input.call(Attribute::parse_outer)?;
                input.parse::<Token![async]>()?;

                name = Some(input.parse()?);
                
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
            } else {
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
                        let (path_, path_params_) = Self::parse_path(path_str)?;
                        path = Some(path_);
                        path_params = path_params_;
                    }
                    "query" => {
                        // Expects equal sign
                        input.parse::<syn::Token![=]>()?;

                        query_params = Some(input.parse()?);
                    }
                    "body" => {
                        // Expects equal sign
                        input.parse::<syn::Token![=]>()?;

                        body_params = Some(input.parse()?);
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "Unknown attribute '{}'. Allowed attributes are: 'method', 'group', 'path', 'query', 'body'.",
                                ident
                            ),
                        ));
                    }
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
            group: group.unwrap(),
            method: method.unwrap(),
            handler_attributes,
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
    fn parse_path(literal: LitStr) -> syn::Result<(String, HashMap<Ident, Type>)> {
        let path = literal.value();
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
                            literal.span(),
                            "Expected one of character, `:` or `}`, found `{`",
                        ));
                    }
                }
                '}' => {
                    if state == ParsePathState::PathParamType {
                        let param_name = proc_macro2::TokenStream::from_str(&current_name)
                            .map_err(|_| {
                                syn::Error::new(literal.span(), "Invalid path parameter name")
                            })?;
                        let param_type = proc_macro2::TokenStream::from_str(&current_type)
                            .map_err(|_| {
                                syn::Error::new(proc_macro2::Span::call_site(), "Invalid path parameter type")
                            })?;
                        path_params.insert(syn::parse2(param_name)?, syn::parse2(param_type)?);

                        real_path.push(':');
                        real_path.push_str(&current_name);

                        current_name = String::new();
                        current_type = String::new();
                        state = ParsePathState::Path;
                    } else if state == ParsePathState::PathParamName {
                        return Err(syn::Error::new(
                            literal.span(),
                            "Expected one of character or `:`, found `}`",
                        ));
                    } else {
                        return Err(syn::Error::new(
                            literal.span(),
                            "Expected one of character or `{`, found `}`",
                        ));
                    }
                }
                ':' => {
                    if state == ParsePathState::PathParamName {
                        state = ParsePathState::PathParamType;
                    } else if state != ParsePathState::Path {
                        return Err(syn::Error::new(
                            literal.span(),
                            "Expected one of character or `{`, found `:`",
                        ));
                    } else {
                        return Err(syn::Error::new(
                            literal.span(),
                            "Expected one of character or `}`, found `:`",
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
                literal.span(),
                "Expected one of character or `}`, found EOF",
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

    let query_params = if let Some(q) = args.query_params {
        quote::quote! { axum::extract::Query(query): axum::extract::Query<#q>, }
    } else {
        quote::quote! {}
    };

    let body_params = if let Some(b) = args.body_params {
        quote::quote! { body: #b, }
    } else {
        quote::quote! {}
    };

    let route_name = Ident::new(
        &format!(
            "ROUTE_{}",
            stringcase::macro_case(args.name.to_string().as_str())
        ),
        proc_macro2::Span::call_site(),
    );
    let name = args.name;
    let path = args.path;
    let parameters = args.parameters;
    let return_type = args.return_type;
    let block = args.handler;
    let group = args.group;
    let handler_attributes = args.handler_attributes;

    let handler_def = quote::quote! {
        #(#handler_attributes)*
        async fn #name(#path_params #query_params #body_params #parameters) -> #return_type #block
    };

    let handler = match args.method {
        Method::Get => quote::quote! { axum::routing::get(#name) },
        Method::Post => quote::quote! { axum::routing::post(#name) },
        Method::Put => quote::quote! { axum::routing::put(#name) },
        Method::Patch => quote::quote! { axum::routing::patch(#name) },
        Method::Delete => quote::quote! { axum::routing::delete(#name) },
        Method::Head => quote::quote! { axum::routing::head(#name) },
        Method::Options => quote::quote! { axum::routing::options(#name) },
        Method::Trace => quote::quote! { axum::routing::trace(#name) },
        Method::Connect => quote::quote! { axum::routing::connect(#name) },
    };

    let result = quote::quote! {
        #handler_def

        pub static #route_name: #group = #group::new(#path, |r, _| r.route(#path, #handler));

        axum_distributed_routing::inventory::submit! {
            #route_name
        }
    };

    result.into()
}
