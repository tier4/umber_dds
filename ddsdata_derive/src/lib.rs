use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, LitStr};

/// Derives the `DdsData` trait for a struct, providing DDS key generation and type naming.
///
/// This macro automatically implements the `gen_key`, `type_name`, and `is_with_key`
/// functions required for DDS communication. Note that these methods are primarily
/// intended for internal use by the library.
///
/// ## ⚠️ Prerequisites and Dependencies
/// To use this macro, you must have the following in scope and in your `Cargo.toml`:
/// * **`use umber_dds::KeyHash;`**: Required for returning the generated key.
/// * **`use speedy::Writable;`**: Required for serializing the key fields **(only required if one or more `#[key]` attributes are specified)**.
/// * **`md5` crate**: Required in `Cargo.toml`. Used to compute hashes for serialized keys exceeding 16 bytes **(only required if one or more `#[key]` attributes are specified)**.
///
/// ## Field Attributes
/// * `#[key]`: Marks a field as part of the DDS Key. Can be applied to multiple fields.
/// * `#[dds_data(type_name = "CustomName")]`: (Optional) Overrides the default type name.
///   If omitted, the struct's exact Rust identifier is used.
///
/// ## Key Generation Logic (`gen_key`)
/// When calculating the `KeyHash`, the macro extracts fields marked with `#[key]`,
/// serializes them in Big Endian format, and applies the following rules:
/// 1. **Length <= 16 bytes:** Padded with trailing zeros (`0`) to exactly 16 bytes.
/// 2. **Length > 16 bytes:** Computes an MD5 hash of the bytes and uses the 16-byte digest.
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
                            keys.push((ident, &field.ty));
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

    let is_with_key_val = keys_count != 0;

    let gen_key_body = if keys_count == 0 {
        quote! {
            None
        }
    } else {
        // create wrapper struct for gen_key().
        let wrapper_name = syn::Ident::new(&format!("{}KeyWrapper", name), name.span());

        // definition of wrapper struct field
        let wrapper_fields = keys.iter().map(|(ident, ty)| {
            quote! { #ident: #ty }
        });

        // initialization of wrapper struct field
        let wrapper_init = keys.iter().map(|(ident, _ty)| {
            quote! { #ident: self.#ident.clone() }
        });

        let write_stmts = keys.iter().map(|(ident, ty)| {
            let write_stmt = gen_write_stmt(ty, quote!(self.#ident));
            quote! { #write_stmt }
        });

        quote! {
            #[derive(Clone)]
            struct #wrapper_name {
                #(#wrapper_fields),*
            }

            impl<C: speedy::Context> speedy::Writable<C> for #wrapper_name {
                #[inline]
                fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
                    let mut __cdr_offset = 0usize;
                    #(#write_stmts)*
                    Ok(())
                }
            }

            let wrapper = #wrapper_name {
                #(#wrapper_init),*
            };

            let mut result = wrapper.write_to_vec_with_ctx(speedy::Endianness::BigEndian).unwrap();

            let rlen = result.len();
            if rlen <= 16 {
                for _ in 0..(16-rlen) {
                    result.push(0);
                }
                Some(KeyHash::new(&result[0 .. 16]))
            } else {
                let md5 = md5::compute(result);
                Some(KeyHash::new(&md5.0))
            }
        }
    };

    let expanded = quote! {
        impl DdsData for #name {
            fn gen_key(&self) -> Option<KeyHash> {
                #gen_key_body
            }

            fn type_name() -> String {
                #final_type_name.to_string()
            }

            fn is_with_key() -> bool {
                #is_with_key_val
            }
        }
    };

    TokenStream::from(expanded)
}

/// Vec<T> -> T
fn get_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// [T; N] -> T, N
fn get_array_info(ty: &syn::Type) -> Option<(&syn::Type, usize)> {
    if let syn::Type::Array(arr) = ty {
        let elem_ty = &*arr.elem;
        if let syn::Expr::Lit(expr_lit) = &arr.len {
            if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                if let Ok(len) = lit_int.base10_parse::<usize>() {
                    return Some((elem_ty, len));
                }
            }
        }
    }
    None
}

/// HashMap/BTreeMap<K, V> ->  <"HachMap"/"BTreeMap">, K, V
fn get_map_info(ty: &syn::Type) -> Option<(String, &syn::Type, &syn::Type)> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            let ident_str = segment.ident.to_string();
            // HashMap または BTreeMap かどうかを判定
            if ident_str == "HashMap" || ident_str == "BTreeMap" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    let mut types = args.args.iter().filter_map(|arg| {
                        if let syn::GenericArgument::Type(ty) = arg {
                            Some(ty)
                        } else {
                            None
                        }
                    });
                    if let (Some(k), Some(v)) = (types.next(), types.next()) {
                        return Some((ident_str, k, v));
                    }
                }
            }
        }
    }
    None
}

fn gen_read_expr(ty: &syn::Type) -> TokenStream2 {
    let type_string = quote!(#ty).to_string().replace(" ", "");

    if let Some(inner_ty) = get_vec_inner_type(ty) {
        let read_inner = gen_read_expr(inner_ty);
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                if pad > 0 { reader.skip_bytes(pad)?; __cdr_offset += pad; }

                let vec_len = reader.read_i32()?;
                __cdr_offset += 4usize;

                let mut vec = Vec::with_capacity(vec_len as usize);
                for _ in 0..vec_len {
                    vec.push(#read_inner);
                }
                vec
            }
        };
    }

    if let Some((inner_ty, len)) = get_array_info(ty) {
        let read_inner = gen_read_expr(inner_ty);
        return quote! {
            {
                let mut temp = Vec::with_capacity(#len);
                for _ in 0..#len {
                    temp.push(#read_inner);
                }
                temp.try_into().unwrap_or_else(|_| unreachable!("Array conversion failed"))
            }
        };
    }

    if let Some((map_type_str, key_ty, val_ty)) = get_map_info(ty) {
        let read_key = gen_read_expr(key_ty);
        let read_val = gen_read_expr(val_ty);
        let map_init = if map_type_str == "BTreeMap" {
            quote! { std::collections::BTreeMap::new() }
        } else {
            quote! { std::collections::HashMap::new() }
        };
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                if pad > 0 { reader.skip_bytes(pad)?; __cdr_offset += pad; }

                let map_len = reader.read_i32()?;
                __cdr_offset += 4usize;

                let mut map = #map_init;
                for _ in 0..map_len {
                    let k = #read_key;
                    let v = #read_val;
                    map.insert(k, v);
                }
                map
            }
        };
    }

    if type_string == "String" {
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                if pad > 0 { reader.skip_bytes(pad)?; __cdr_offset += pad; }

                let cdr_name_len = reader.read_i32()?;
                __cdr_offset += 4usize;

                let c = reader.read_string((cdr_name_len - 1) as usize)?;
                reader.read_u8()?; // null char

                __cdr_offset += cdr_name_len as usize;
                c
            }
        };
    }

    let align_and_size = match type_string.as_str() {
        "u8" | "i8" | "bool" | "char" => Some((1, 1)),
        "u16" | "i16" => Some((2, 2)),
        "u32" | "i32" | "f32" => Some((4, 4)),
        "u64" | "i64" | "f64" => Some((8, 8)),
        _ => None,
    };

    if let Some((align, size)) = align_and_size {
        let read_call = match type_string.as_str() {
            "u8" => quote!(reader.read_u8()?),
            "i8" => quote!(reader.read_i8()?),
            "bool" => quote!(reader.read_u8()? != 0),
            "char" => quote!(reader.read_u8()? as char),
            "u16" => quote!(reader.read_u16()?),
            "i16" => quote!(reader.read_i16()?),
            "u32" => quote!(reader.read_u32()?),
            "i32" => quote!(reader.read_i32()?),
            "f32" => quote!(reader.read_f32()?),
            "u64" => quote!(reader.read_u64()?),
            "i64" => quote!(reader.read_i64()?),
            "f64" => quote!(reader.read_f64()?),
            _ => unreachable!(),
        };

        return quote! {
            {
                let pad = ((#align as usize) - (__cdr_offset % (#align as usize))) % (#align as usize);
                if pad > 0 { reader.skip_bytes(pad)?; __cdr_offset += pad; }

                let val = #read_call;
                __cdr_offset += (#size as usize);
                val
            }
        };
    }

    quote! {
        {
            <#ty as speedy::Readable<'a, C>>::read_from(reader)?
        }
    }
}

fn gen_write_stmt(ty: &syn::Type, val_expr: TokenStream2) -> TokenStream2 {
    let type_string = quote!(#ty).to_string().replace(" ", "");

    if let Some(inner_ty) = get_vec_inner_type(ty) {
        let write_inner = gen_write_stmt(inner_ty, quote!(*item));
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                const ZEROS: [u8; 3] = [0;3];
                writer.write_bytes(&ZEROS[..pad])?;
                __cdr_offset += pad;

                let vec_len = (#val_expr).len();
                writer.write_i32(vec_len as i32)?;
                __cdr_offset += 4usize;

                for item in &(#val_expr) {
                    #write_inner
                }
            }
        };
    }

    if let Some((inner_ty, _len)) = get_array_info(ty) {
        let write_inner = gen_write_stmt(inner_ty, quote!(*item));
        return quote! {
            {
                for item in &(#val_expr) {
                    #write_inner
                }
            }
        };
    }

    if let Some((_map_type_str, key_ty, val_ty)) = get_map_info(ty) {
        let write_key = gen_write_stmt(key_ty, quote!(*k));
        let write_val = gen_write_stmt(val_ty, quote!(*v));
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                const ZEROS: [u8; 3] = [0;3];
                writer.write_bytes(&ZEROS[..pad])?;
                __cdr_offset += pad;

                let map_len = (#val_expr).len();
                writer.write_i32(map_len as i32)?;
                __cdr_offset += 4usize;

                for (k, v) in &(#val_expr) {
                    #write_key
                    #write_val
                }
            }
        };
    }

    if type_string == "String" {
        return quote! {
            {
                let pad = (4usize - (__cdr_offset % 4usize)) % 4usize;
                const ZEROS: [u8; 3] = [0;3];
                writer.write_bytes(&ZEROS[..pad])?;
                __cdr_offset += pad;

                let cdr_name_len = (#val_expr).len() + 1;
                writer.write_i32(cdr_name_len as i32)?;
                __cdr_offset += 4usize;

                writer.write_bytes((#val_expr).as_bytes())?;
                writer.write_u8(0)?; // null char
                __cdr_offset += cdr_name_len;
            }
        };
    }

    let align_and_size = match type_string.as_str() {
        "u8" | "i8" | "bool" | "char" => Some((1, 1)),
        "u16" | "i16" => Some((2, 2)),
        "u32" | "i32" | "f32" => Some((4, 4)),
        "u64" | "i64" | "f64" => Some((8, 8)),
        _ => None,
    };

    if let Some((align, size)) = align_and_size {
        let write_call = match type_string.as_str() {
            "u8" => quote!(writer.write_u8(#val_expr)?),
            "i8" => quote!(writer.write_i8(#val_expr)?),
            "bool" => quote!(writer.write_u8(if #val_expr { 1 } else { 0 })?),
            "char" => quote!(writer.write_u8((#val_expr) as u8)?),
            "u16" => quote!(writer.write_u16(#val_expr)?),
            "i16" => quote!(writer.write_i16(#val_expr)?),
            "u32" => quote!(writer.write_u32(#val_expr)?),
            "i32" => quote!(writer.write_i32(#val_expr)?),
            "f32" => quote!(writer.write_f32(#val_expr)?),
            "u64" => quote!(writer.write_u64(#val_expr)?),
            "i64" => quote!(writer.write_i64(#val_expr)?),
            "f64" => quote!(writer.write_f64(#val_expr)?),
            _ => unreachable!(),
        };

        let pad_max = (align - 1) as usize;

        return quote! {
            {
                let pad = ((#align as usize) - (__cdr_offset % (#align as usize))) % (#align as usize);
                const ZEROS: [u8; #pad_max] = [0; #pad_max];
                writer.write_bytes(&ZEROS[..pad])?;
                __cdr_offset += pad;

                #write_call;
                __cdr_offset += (#size as usize);
            }
        };
    }

    quote! {
        {
            (#val_expr).write_to(writer)?;
        }
    }
}

#[proc_macro_derive(DdsDeserialize)]
/// Derives the `speedy::Readable` trait with strict CDR (Common Data Representation) alignment.
///
/// This macro automatically generates deserialization logic that complies with DDS CDR
/// padding and memory alignment rules (e.g., 4-byte alignment for `i32`, 8-byte alignment for `f64`).
///
/// ## ⚠️ Prerequisites
/// * **`speedy` crate**: Required in `Cargo.toml`.
///
/// ## Supported Data Types
/// * **Primitives:** `u8`, `i8`, `bool`, `u16`, `i16`, `u32`, `i32`, `f32`, `u64`, `i64`, `f64`.
/// * **Characters (`char`):** Interpreted as a 1-byte C-style `char` (equivalent to `u8`) during deserialization, not as a 4-byte Rust Unicode scalar value.
/// * **Strings:** standard Rust `String` (handles null-termination and length prefixes).
/// * **Collections:** `Vec<T>`, Fixed-size arrays `[T; N]`, `HashMap<K, V>`, `BTreeMap<K, V>`.
/// * **Nested Types:** Any other struct that implements the `speedy::Readable` trait.
///
/// *(Note: Ensure `speedy::Readable` is in scope if you are deserializing nested custom types.)*
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
        let read_expr = gen_read_expr(ftype);

        quote! {
            let #fname = #read_expr;
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
                let mut __cdr_offset = 0usize;

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
/// Derives the `speedy::Writable` trait with strict CDR (Common Data Representation) alignment.
///
/// This macro automatically generates serialization logic that complies with DDS CDR
/// padding and memory alignment rules.
///
/// ## ⚠️ Prerequisites
/// * **`speedy` crate**: Required in `Cargo.toml`.
///
/// ## Supported Data Types
/// * **Primitives:** `u8`, `i8`, `bool`, `u16`, `i16`, `u32`, `i32`, `f32`, `u64`, `i64`, `f64`.
/// * **Characters (`char`):** Interpreted as a 1-byte C-style `char` (equivalent to `u8`) during serialization, not as a 4-byte Rust Unicode scalar value.
/// * **Strings:** standard Rust `String` (handles null-termination and length prefixes).
/// * **Collections:** `Vec<T>`, Fixed-size arrays `[T; N]`, `HashMap<K, V>`, `BTreeMap<K, V>`.
/// * **Nested Types:** Any other struct that implements the `speedy::Writable` trait.
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
        let write_stmt = gen_write_stmt(ftype, quote!(self.#fname));

        quote! {
            #write_stmt
        }
    });

    let gen = quote! {
        impl<C: speedy::Context> speedy::Writable<C> for #name {
            #[inline]
            fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
                let mut __cdr_offset = 0usize;

                #(#write_fields)*

                Ok(())
            }
        }
    };

    gen.into()
}
