use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(DdsData, attributes(key, dds_data))]
pub fn derive_ddsdata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut user_type_name = None;
    for attr in &input.attrs {
        if attr.path().is_ident("dds_data") {
            let _ = attr.parse_nested_meta(|meta| {
                // #[keyed(type_name = "...")]
                if meta.path.is_ident("type_name") {
                    let expr = meta.value()?;
                    let s: LitStr = expr.parse()?;
                    user_type_name = Some(s.value());
                }
                Ok(())
            });
        }
    }

    let mut keys = Vec::new();
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                for attr in &field.attrs {
                    if attr.path().is_ident("key") {
                        if let Some(ident) = &field.ident {
                            keys.push(ident);
                        }
                    }
                }
            }
        }
    }

    let default_type_name = name.to_string();
    let final_type_name = match user_type_name {
        Some(custom) => custom,
        None => default_type_name,
    };

    let keys_count = keys.len();

    let constract_key = keys.iter().map(|key| {
        quote! {
            let mut serialize_data = cdr::serialize::<_,_, CdrBe>(&self.#key, Infinite).expect("");
            let _ = serialize_data.drain(0..=3);
            for b in serialize_data {
                result.push(b)
            }
        }
    });

    let expanded = quote! {
        impl DdsData for #name {
            fn gen_key(&self) -> KeyHash {
                let mut cdr_size = 0;
                let mut result: Vec<u8> = Vec::new();
                #(#constract_key)*
                let rlen = result.len();
                if rlen <= 16 {
                    for _ in 0..(16-rlen) {
                        result.push(0);
                    }
                    KeyHash::new(&result[0 .. 16])
                } else {
                    let md5 = compute(result);
                    KeyHash::new(&md5.0)
                }
            }
            fn type_name() -> String {
                #final_type_name.to_string()
            }
            fn is_with_key() -> bool {
                #keys_count != 0
            }
        }
    };

    TokenStream::from(expanded)
}
