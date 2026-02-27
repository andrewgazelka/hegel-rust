use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, FnArg, Ident, ItemFn, Pat, Token, Type};

#[derive(Clone)]
enum GivenArg {
    Infer,
    Explicit(Expr),
}

struct GivenArgs {
    args: Vec<GivenArg>,
}

impl Parse for GivenArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "#[given] requires at least one argument.",
            ));
        }

        let mut args = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![_]) {
                let _: Token![_] = input.parse()?;
                args.push(GivenArg::Infer);
            } else {
                let expr: Expr = input.parse()?;
                args.push(GivenArg::Explicit(expr));
            }

            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(GivenArgs { args })
    }
}

/// A single setting in a #[settings(...)] expression.
struct Setting {
    key: Ident,
    value: Expr,
}

/// Parsed result of a #[settings(...)] expression.
struct SettingsArgs {
    settings: Vec<Setting>,
}

impl Parse for SettingsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut settings = Vec::new();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            let value: Expr = input.parse()?;
            settings.push(Setting { key, value });
            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }
        Ok(SettingsArgs { settings })
    }
}


/// Extract parameter name and type from a function argument.
fn extract_fn_param(arg: &FnArg) -> syn::Result<(&Ident, &Type)> {
    match arg {
        FnArg::Typed(pat_type) => {
            let name = match pat_type.pat.as_ref() {
                Pat::Ident(pat_ident) => &pat_ident.ident,
                Pat::Tuple(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support tuple destructuring in parameters",
                    ))
                }
                Pat::Struct(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support struct destructuring in parameters",
                    ))
                }
                Pat::TupleStruct(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support tuple struct destructuring in parameters",
                    ))
                }
                Pat::Slice(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support slice destructuring in parameters",
                    ))
                }
                Pat::Reference(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support reference patterns in parameters",
                    ))
                }
                Pat::Wild(_) => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] does not support wildcard (_) parameters — \
                        each parameter needs a name",
                    ))
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[given] requires simple named parameters",
                    ))
                }
            };
            Ok((name, pat_type.ty.as_ref()))
        }
        FnArg::Receiver(r) => Err(syn::Error::new_spanned(
            r,
            "#[given] does not currently support annotating instance methods. \
            This is not a fundamental limitation, but requires design thought. \
            Please open an issue.",
        )),
    }
}

fn maybe_parse_settings(func: &ItemFn) -> syn::Result<Option<SettingsArgs>> {
    for attr in &func.attrs {
        let path = attr.path();
        let is_settings = path.is_ident("settings")
            || path
                .segments
                .last()
                .map(|s| s.ident == "settings")
                .unwrap_or(false);
        if is_settings {
            let args: SettingsArgs = attr.parse_args()?;
            return Ok(Some(args));
        }
    }
    Ok(None)
}

fn maybe_parse_given(func: &ItemFn) -> syn::Result<Option<GivenArgs>> {
    for attr in &func.attrs {
        let path = attr.path();
        let is_given = path.is_ident("given")
            || path
                .segments
                .last()
                .map(|s| s.ident == "given")
                .unwrap_or(false);
        if is_given {
            let args: GivenArgs = attr.parse_args()?;
            return Ok(Some(args));
        }
    }
    Ok(None)
}

pub fn expand_given(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> TokenStream {
    let given_args: GivenArgs = match syn::parse2(attr) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error(),
    };

    let mut func: ItemFn = match syn::parse2(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let settings_args = match maybe_parse_settings(&func) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error(),
    };

    // Extract parameters
    let params: Vec<_> = func.sig.inputs.iter().collect();
    let param_info: Vec<(&Ident, &Type)> = match params
        .iter()
        .map(|arg| extract_fn_param(arg))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(info) => info,
        Err(e) => return e.to_compile_error(),
    };

    // A single `_` means "infer all parameters"
    let args = if given_args.args.len() == 1 && matches!(given_args.args[0], GivenArg::Infer) {
        vec![GivenArg::Infer; param_info.len()]
    } else {
        if given_args.args.len() != param_info.len() {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "#[given] has {} arguments but function has {} parameters",
                    given_args.args.len(),
                    param_info.len()
                ),
            )
            .to_compile_error();
        }
        given_args.args
    };

    // Generate let bindings ordered by parameter position
    let draw_bindings: Vec<TokenStream> = args
        .iter()
        .zip(param_info.iter())
        .map(|(arg, (name, ty))| {
            let gen_expr = match arg {
                GivenArg::Infer => quote! { hegel::generators::from_type::<#ty>() },
                GivenArg::Explicit(expr) => quote! { #expr },
            };
            let name_str = name.to_string();
            quote! {
                let #name: #ty = hegel::draw(&#gen_expr.__given_param(#name_str));
            }
        })
        .collect();

    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let body = &func.block;

    let settings_chain: Vec<TokenStream> = match &settings_args {
        Some(args) => args
            .settings
            .iter()
            .map(|s| {
                let key = &s.key;
                let value = &s.value;
                quote! { .#key(#value) }
            })
            .collect(),
        None => vec![],
    };

    let new_body: TokenStream = quote! {
        {
            use hegel::generators::Generate as _;
            hegel::Hegel::new(|| {
                #(#draw_bindings)*
                (|| #body)()
            })
            .__test_name(#fn_name_str)
            #(#settings_chain)*
            .run();
        }
    };

    let new_block: syn::Block = syn::parse2(new_body).expect("failed to parse generated body");

    func.sig.inputs.clear();
    func.block = Box::new(new_block);

    // Remove #[given] and #[settings] attributes (they've been consumed)
    func.attrs.retain(|attr| {
        let path = attr.path();
        !path.is_ident("given")
            && !path.is_ident("settings")
            && !path
                .segments
                .last()
                .map(|s| s.ident == "given" || s.ident == "settings")
                .unwrap_or(false)
    });

    quote! { #func }
}

pub fn expand_settings(
    _attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> TokenStream {
    let func: ItemFn = match syn::parse2(item.clone()) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let given_args = match maybe_parse_given(&func) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error(),
    };

    // Don't actually do anything inside #[settings]. #[given] will handle expanding any shared settings.
    match given_args {
        Some(_) => {
            item
        }
        None => {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[settings] must be set together with #[given], but was set alone here.",
            )
            .to_compile_error()
        }
    }
}
