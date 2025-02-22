use proc_macro::TokenStream;
use quote::*;
use std::{env, fs::File, io::Write, ops::Deref, path::Path};
use syn::{
    parse_macro_input, parse_quote, FnArg, GenericParam, ImplItem, ItemImpl, ReturnType, Type,
};
use syn::{Arm, PathArguments};

///
///  Distribute the user-defined functions in the call function as a mapping
///

pub fn rpcs(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut parsed = parse_macro_input!(input as ItemImpl);

    let struct_name = parsed.self_ty.clone();
    let name = match struct_name.as_ref() {
        Type::Path(path) => path.path.segments[0].ident.clone().to_string(),
        _ => "Error".to_string(),
    };

    let mut fn_names = vec![];
    let mut param_names = vec![];
    let mut param_idents = vec![];
    let mut fn_idents = vec![];
    let mut return_names = vec![];
    let mut is_empty_impl = true;
    let mut rpc_call_match_arms = vec![];

    parsed.items.iter().for_each(|item| match item {
        ImplItem::Method(data) => {
            let fn_ident = data.sig.ident.clone();
            let fn_name = fn_ident.to_string();

            fn_names.push(fn_name.clone());
            fn_idents.push(fn_ident.clone());

            match &data.sig.output {
                ReturnType::Default => return_names.push(String::from("Result<()>")),
                ReturnType::Type(_, t) => {
                    return_names.push(format!("Result<{}>", t.to_token_stream()))
                }
            };

            // TODO: Replace by RPC types
            // return_names.push("Result<Value>");

            data.sig.inputs.iter().for_each(|input| match input {
                FnArg::Receiver(_) => {}
                FnArg::Typed(typed) => match typed.ty.deref() {
                    Type::Path(p) => {
                        p.path.segments.iter().for_each(|seg| {
                            let param_ident = seg.ident.clone();
                            param_names.push(param_ident.to_string());
                            param_idents.push(param_ident.clone());

                            let rcma:Arm = parse_quote! {
                                #fn_name => {
                                    let param = serde_json::from_value::<#param_ident>(params)?;

                                    let response = self.#fn_ident(ctx, param).await;

                                    if response.code != 0 {
                                        Err(abcf::Error::new_rpc_error(response.code, response.message))
                                    } else if let Some(v) = response.data {
                                        Ok(Some(serde_json::to_value(v)?))
                                    } else {
                                        Ok(None)
                                    }
                                }
                            };

                            rpc_call_match_arms.push(rcma);
                        });
                    }
                    _ => {}
                },
            });
            is_empty_impl = false;
        }
        _ => {}
    });

    let out_dir_str = env::var("OUT_DIR").expect("please create build.rs");
    let out_dir = Path::new(&out_dir_str).join(name.to_lowercase() + ".rs");
    let mut f = File::create(&out_dir).expect("create file error");
    let module_name_mod_name = format!("__abcf_storage_{}", name.to_lowercase());

    let dependency = format!(
        r#"
use abcf_sdk::jsonrpc::endpoint;
use abcf_sdk::error::*;
use abcf_sdk::providers::Provider;
use super::*;
      "#
    );

    f.write_all(dependency.as_bytes()).expect("write error");

    fn_names.iter().zip(param_names).zip(return_names).for_each(
        |((fn_name, param_name), return_name)| {
            let s = format!(
                r#"
pub async fn {}<P: Provider>(p: P, param: {}) -> {} {{
    let mut p = p;

    let data = serde_json::to_string(&param)?;
    let abci_query_req = endpoint::abci_query::Request {{
        path: format!("rpc/{{}}/{}",{}::MODULE_NAME),
        data,
        height:Some("0".to_string()),
        prove: false,
    }};

    let result: Option<endpoint::abci_query::Response> = p.request("abci_query", &abci_query_req).await?;
    if result.is_none() {{
      return Err(Error::ErrorString("result is none".to_string()));
    }}
    let result = result.unwrap();
    abcf::log::debug!("Recv RPC response {{:?}}", result);

    if result.response.code == 0 {{
        let res = serde_json::from_slice(&result.response.value)?;
        Ok(RPCResponse::new(res))
    }} else {{
        Err(Error::ReturnError(endpoint::Response::AbciQuery(result)))
    }}
}}
            "#,
                fn_name, param_name, return_name, fn_name, module_name_mod_name
            );
            f.write_all(s.as_bytes()).expect("write error");
        },
    );

    let trait_name = if let Some(t) = &parsed.trait_ {
        t.1.clone()
    } else {
        parse_quote!(abcf::RPCs)
    };

    let param_s: GenericParam = parse_quote!(S: abcf::bs3::Store);
    parsed.generics.params.push(param_s);

    let mut generics_names = Vec::new();
    let mut lifetime_names = Vec::new();

    for x in &parsed.generics.params {
        if let GenericParam::Type(t) = x {
            generics_names.push(t.ident.clone());
        } else if let GenericParam::Lifetime(l) = x {
            lifetime_names.push(l.lifetime.clone());
        }
    }

    let mut pre_rpc: ItemImpl = if is_empty_impl {
        parse_quote! {
            #[async_trait::async_trait]
            impl #trait_name<abcf::Stateless<Self>, abcf::Stateful<Self>> for #struct_name {
                async fn call(
                    &mut self,
                    ctx: &mut abcf::manager::RContext<abcf::Stateless<Self>, abcf::Stateful<Self>>,
                    method: &str,
                    params: serde_json::Value)
                -> abcf::Result<Option<serde_json::Value>> {
                    Ok(None)
                }
            }
        }
    } else {
        parse_quote! {
            #[async_trait::async_trait]
            impl #trait_name<abcf::Stateless<Self>, abcf::Stateful<Self>> for #struct_name {
                async fn call(
                    &mut self,
                    ctx: &mut abcf::manager::RContext<abcf::Stateless<Self>, abcf::Stateful<Self>>,
                    method: &str,
                    params: serde_json::Value)
                -> abcf::Result<Option<serde_json::Value>> {
                    match method {
                        #(#rpc_call_match_arms)*
                        _ => {Err(abcf::Error::TempOnlySupportRPC)}
                    }
                }
            }
        }
    };

    if let Type::Path(p) = parsed.self_ty.as_mut() {
        let segments = &mut p.path.segments;
        let arguments = &mut segments.last_mut().unwrap().arguments;
        if let PathArguments::AngleBracketed(a) = arguments {
            a.args.push(parse_quote!(S));
        } else {
            *arguments = PathArguments::AngleBracketed(parse_quote!(<S>));
        }
    }

    pre_rpc.generics = parsed.generics.clone();
    pre_rpc.self_ty = parsed.self_ty.clone();

    let expanded = quote! {
        #parsed

        #pre_rpc
    };

    TokenStream::from(expanded)
}
