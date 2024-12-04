use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Keyed, attributes(key))]
pub fn derive_keyed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

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
        impl Keyed for #name {
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
        }
    };

    TokenStream::from(expanded)
}
