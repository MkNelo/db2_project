extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, punctuated::Punctuated, Token, Type};

#[proc_macro]
pub fn params(ts: TokenStream) -> TokenStream {
    let parser = Punctuated::<Type, Token![,]>::parse_terminated;
    let types = parser
        .parse(ts.clone())
        .expect("Expected a comma separated list of types")
        .into_iter()
        .collect::<Vec<Type>>();

    let count = types.iter().count();
    TokenStream::from(quote! {
        &{
            use serde_json::Value;
            use serde_json::Value::Array;
            use serde_json::from_value;
            use tokio_postgres::types::ToSql;
            fn query_for_types(value: &Value) -> Option<Vec<Box<(dyn ToSql + Send + Sync)>>> {
                if let Array(ref vec) = value {
                    let mut iter = vec.iter();
                    let mut returning: Vec<Box<(dyn ToSql + Send + Sync)>> = Vec::with_capacity(#count);
                    #(
                        let transaction = iter
                        .next()
                        .and_then(|obj| from_value::<#types>(obj.clone()).ok());

                        if let Some(obj) = transaction {
                            returning.push(Box::new(obj));
                        } else {
                            return None;
                        }
                    )*
                    Some(returning)
                }
                else {
                    None
                }
            }

            query_for_types
        }
    })
}

#[cfg(test)]
mod tests {}
