use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, GenericArgument, ItemFn, PathArguments, ReturnType, Type};

fn extract_ok_type(return_type: &ReturnType) -> Result<&Type, TokenStream> {
    if let ReturnType::Type(_, ty) = return_type {
        if let Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Result" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(ok_type)) = args.args.first() {
                            return Ok(ok_type);
                        }
                    }
                }
            }
        }
        Err(
            Error::new_spanned(ty, "Expected a Result<T, anyhow::Error> return type")
                .to_compile_error()
                .into(),
        )
    } else {
        Err(
            Error::new_spanned(return_type, "Expected a Result return type")
                .to_compile_error()
                .into(),
        )
    }
}

#[proc_macro_attribute]
/// This macro wraps a function that returns a Result with an `anyhow::Result`.
///
/// If the function returns `anyhow::Result<OkType>`, it will be converted to `windows::core::Result<OkType>`.
///
///
/// ```rust
/// #[macros::anyhow]
/// fn some_func() -> anyhow::Result<Sometype> {
///     Ok(Sometype)
/// }
///
/// // will be converted to
///
/// fn some_func() -> windows::core::Result<Sometype> {
///   let result: anyhow::Result<Sometype> = (|| Ok(Sometype))();
///   match result {
///     Ok(v) => Ok(v),
///     Err(e) => {
///       log::error!("Error: {:?}", e);
///       Err(windows::core::Error::from(windows::Win32::Foundation::E_FAIL))
///     }
///   }
/// }
/// ```
pub fn anyhow(_: TokenStream, input: TokenStream) -> TokenStream {
    // parse the input function
    let input_fn = parse_macro_input!(input as ItemFn);

    // get the function name, inputs, and body
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;

    // check if the function has a return type
    let output = match &input_fn.sig.output {
        ReturnType::Type(_, _ty) => {
            let result = extract_ok_type(&input_fn.sig.output);

            match result {
                Ok(ok_type) => ok_type,
                Err(err) => return err,
            }
        }
        _ => {
            return Error::new_spanned(&input_fn.sig, "Expected a Result return type")
                .to_compile_error()
                .into();
        }
    };

    // generate the new function
    let generated = quote! {
        fn #fn_name(#fn_inputs) -> windows::core::Result<#output> {
            let result: Result<#output> = (|| #fn_body)();

            match result {
                Ok(v) => Ok(v),
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    Err(windows::core::Error::from(windows::Win32::Foundation::E_FAIL))
                }
            }
        }
    };

    generated.into()
}
