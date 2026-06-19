use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, LitStr};

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
            // TODO: keyがStringだったときにCDR形式からはずれてしまう。
            let serialize_data = self.#key.write_to_vec_with_ctx(Endianness::BigEndian).unwrap();
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

#[proc_macro_derive(DdsDeserialize)]
pub fn derive_dds_deserialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("DdsDeserialize can only be used on structs with named fields"),
    };

    let read_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        let ftype = &f.ty;
        let type_string = quote!(#ftype).to_string();

        if type_string.contains("String") {
            quote! {
                let #fname = {
                    let cdr_str_len = reader.read_i32()?;
                    let c = reader.read_string((cdr_str_len - 1) as usize)?;
                    reader.read_u8()?; // null char
                    reader.skip_bytes((4 - cdr_str_len as usize % 4) % 4)?;
                    c
                };
            }
        } else if type_string == "u32" {
            quote! {
                let #fname = reader.read_u32()?;
            }
        } else {
            quote! {
                let #fname = <#ftype as speedy::Readable<'a, C>>::read_from(reader)?;
            }
        }
    });

    let init_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        quote! { #fname }
    });

    let gen = quote! {
        impl<'a, C: speedy::Context> speedy::Readable<'a, C> for #name {
            #[inline]
            fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
                #(#read_fields)*
                Ok(Self {
                    #(#init_fields),*
                })
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(DdsSerialize)]
pub fn derive_dds_serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("DdsSerialize can only be used on structs with named fields"),
    };

    let write_fields = fields.iter().map(|f| {
        let fname = &f.ident;
        let ftype = &f.ty;
        let type_string = quote!(#ftype).to_string();

        if type_string.contains("String") {
            quote! {
                let cdr_str_len = self.#fname.len() + 1;
                writer.write_i32(cdr_str_len as i32)?;
                writer.write_bytes(self.#fname.as_bytes())?;
                writer.write_u8(0)?; // null char

                // padding
                const ZEROS: [u8; 3] = [0; 3];
                writer.write_bytes(&ZEROS[..((4 - cdr_str_len % 4) % 4)])?;
            }
        } else if type_string == "u32" {
            quote! {
                writer.write_u32(self.#fname)?;
            }
        } else {
            quote! {
                self.#fname.write_to(writer)?;
            }
        }
    });

    let gen = quote! {
        impl<C: speedy::Context> speedy::Writable<C> for #name {
            #[inline]
            fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
                #(#write_fields)*
                Ok(())
            }
        }
    };

    gen.into()
}
