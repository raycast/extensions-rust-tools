use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, FnArg, Pat, Type};
use std::sync::atomic::{AtomicBool, Ordering};

// Global flag to track if main has been generated
static MAIN_GENERATED: AtomicBool = AtomicBool::new(false);

/// Attribute macro that marks functions for export to TypeScript
///
/// Usage:
/// ```rust
/// #[raycast]
/// fn greeting(name: String, is_formal: bool) -> String {
///     format!("Hello {}{name}!", if is_formal { "Mr/Ms " } else { "" })
/// }
/// ```
#[proc_macro_attribute]
pub fn raycast(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let expanded = expand_raycast_function(input_fn);

    TokenStream::from(expanded)
}

fn expand_raycast_function(input_fn: ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;
    let fn_sig = &input_fn.sig;

    // Validate function signature
    validate_function_signature(&input_fn.sig);

    // Extract parameter information
    let param_names: Vec<_> = input_fn.sig.inputs.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                    Some(&ident.ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let param_types: Vec<_> = input_fn.sig.inputs.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type.ty.as_ref())
            } else {
                None
            }
        })
        .collect();

    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();

    // Check if function returns Result
    let returns_result = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => is_result_type(ty),
        ReturnType::Default => false,
    };

    // Generate the registry entry
    let registry_entry = generate_registry_entry(&fn_name_str, &param_names, &param_types, is_async, returns_result);

    // Check if we should generate main function
    let should_generate_main = !MAIN_GENERATED.swap(true, Ordering::SeqCst);

    if should_generate_main {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig #fn_block

            #registry_entry

            // Auto-generate main function (only once)
            #[tokio::main]
            async fn main() -> Result<(), Box<dyn std::error::Error>> {
                raycast_rust_runtime::RaycastExecutor::run_cli().await?;
                Ok(())
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig #fn_block

            #registry_entry
        }
    }
}

fn validate_function_signature(sig: &syn::Signature) {
    // Check for unsupported features
    if sig.variadic.is_some() {
        panic!("Variadic functions are not supported with #[raycast]");
    }

    // Check for self parameters
    for input in &sig.inputs {
        if let FnArg::Receiver(_) = input {
            panic!("Methods with self parameters are not supported with #[raycast]. Use free functions instead.");
        }
    }
}

fn generate_param_parsing(param_names: &[&syn::Ident], param_types: &[&Type]) -> proc_macro2::TokenStream {
    let param_count = param_names.len();

    let parsing_code = param_names.iter().zip(param_types.iter()).enumerate().map(|(i, (name, ty))| {
        let json_var = syn::Ident::new(&format!("{}_json", name), name.span());
        quote! {
            let #json_var = args.get(#i)
                .ok_or_else(|| raycast_rust_runtime::RaycastError::MissingArgument {
                    function: _function_name.to_string(),
                    parameter: stringify!(#name).to_string(),
                    position: #i,
                })?
                .clone();
            let #name: #ty = serde_json::from_value(#json_var)
                .map_err(|e| raycast_rust_runtime::RaycastError::DecodingError {
                    function: _function_name.to_string(),
                    parameter: stringify!(#name).to_string(),
                    position: #i,
                    error: e.to_string(),
                })?;
        }
    });

    quote! {
        if args.len() != #param_count {
            return Err(raycast_rust_runtime::RaycastError::ArgumentCountMismatch {
                function: _function_name.to_string(),
                expected: #param_count,
                actual: args.len(),
            });
        }

        #(#parsing_code)*
    }
}


fn generate_registry_entry(fn_name_str: &str, param_names: &[&syn::Ident], param_types: &[&Type], is_async: bool, returns_result: bool) -> proc_macro2::TokenStream {
    let fn_ident = syn::Ident::new(&fn_name_str, proc_macro2::Span::call_site());
    let param_parsing = generate_param_parsing(param_names, param_types);

    let function_call = quote! { #fn_ident(#(#param_names),*) };

    let result_handling = if returns_result {
        quote! {
            raycast_rust_runtime::serialize_result_to_json(result)
        }
    } else {
        quote! {
            raycast_rust_runtime::serialize_to_json(result)
        }
    };

    let execute_fn = if is_async {
        quote! {
            |_function_name: String, args: Vec<serde_json::Value>| {
                Box::pin(async move {
                    #param_parsing
                    let result = #function_call.await;
                    #result_handling
                })
            }
        }
    } else {
        quote! {
            |_function_name: String, args: Vec<serde_json::Value>| {
                Box::pin(async move {
                    #param_parsing
                    let result = #function_call;
                    #result_handling
                })
            }
        }
    };

    quote! {
        raycast_rust_runtime::inventory::submit! {
            raycast_rust_runtime::RaycastFunction {
                name: #fn_name_str,
                execute: #execute_fn,
            }
        }
    }
}

fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}
