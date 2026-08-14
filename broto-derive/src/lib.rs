use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Variant, parse_macro_input};

#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("broto-derive features `async` and `sync` are mutually exclusive");
#[cfg(not(any(feature = "async", feature = "sync")))]
compile_error!("enable exactly one of `broto-derive`'s `sync` or `async` features");

#[proc_macro_derive(Encode, attributes(tag))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let krate = broto_crate();

    #[cfg(feature = "async")]
    let is_async = true;
    #[cfg(feature = "sync")]
    let is_async = false;

    build_encode_impl(&input, &krate, is_async).into()
}

#[proc_macro_derive(Decode, attributes(tag))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let krate = broto_crate();

    #[cfg(feature = "async")]
    let is_async = true;
    #[cfg(feature = "sync")]
    let is_async = false;

    build_decode_impl(&input, &krate, is_async).into()
}

/// Reads `#[tag(N)]` off a variant if present, falling back to
/// `default_idx` (its declaration position) otherwise. `N` accepts any
/// valid Rust integer literal syntax (decimal, `0x`, `0o`, `0b`) — syn
/// normalizes them all before parsing. Panics with a clear message if the
/// attribute is malformed, or the value doesn't fit in a `u8` (the wire
/// discriminant type).
fn variant_discriminant(variant: &Variant, default_idx: usize) -> u8 {
    for attr in &variant.attrs {
        if attr.path().is_ident("tag") {
            let lit: syn::LitInt = attr.parse_args().unwrap_or_else(|e| {
                panic!("invalid #[tag(...)] on variant '{}': {e}", variant.ident)
            });
            return lit.base10_parse::<u8>().unwrap_or_else(|e| {
                panic!(
                    "#[tag(...)] on variant '{}' must fit in a u8 (0..=255): {e}",
                    variant.ident
                )
            });
        }
    }

    u8::try_from(default_idx).unwrap_or_else(|_| {
        panic!(
            "enum has more than 256 variants; discriminants must fit in a u8 \
             (tag variant '{}' explicitly with #[tag(...)] to work around this)",
            variant.ident
        )
    })
}

/// Panics if any two variants ended up with the same discriminant — whether
/// from two explicit `#[tag(...)]`s, or an explicit tag colliding with
/// another variant's default positional index.
fn check_unique_discriminants(enum_name: &syn::Ident, variants: &[&Variant], discriminants: &[u8]) {
    let mut seen: std::collections::HashMap<u8, &syn::Ident> = std::collections::HashMap::new();
    for (variant, &discriminant) in variants.iter().zip(discriminants) {
        if let Some(prev) = seen.insert(discriminant, &variant.ident) {
            panic!(
                "enum '{enum_name}': variants '{prev}' and '{}' both use discriminant {discriminant} \
                 — give one of them an explicit #[tag(...)] that doesn't collide",
                variant.ident
            );
        }
    }
}

fn build_encode_impl(input: &DeriveInput, krate: &TokenStream2, is_async: bool) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(s) => {
            let stmts = encode::struct_encode_stmts(&s.fields, krate, is_async);
            quote! {
                let mut total = 0usize;
                #(#stmts)*
                Ok(total)
            }
        }
        Data::Enum(e) => {
            let variants: Vec<&Variant> = e.variants.iter().collect();
            let discriminants: Vec<u8> = variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| variant_discriminant(variant, idx))
                .collect();

            check_unique_discriminants(name, &variants, &discriminants);

            let arms: Vec<_> = variants
                .iter()
                .zip(&discriminants)
                .map(|(variant, &discriminant)| {
                    encode::enum_encode_arm(
                        name,
                        &variant.ident,
                        &variant.fields,
                        discriminant,
                        krate,
                        is_async,
                    )
                })
                .collect();

            quote! {
                let mut total = 0usize;
                match self {
                    #(#arms),*
                }
                Ok(total)
            }
        }
        Data::Union(_) => panic!("Encode derive does not support unions"),
    };

    if is_async {
        quote! {
            impl #impl_generics #krate::Encode for #name #ty_generics #where_clause {
                async fn encode<W>(&self, writer: &mut W) -> #krate::Result<usize>
                where
                    W: #krate::AsyncWrite + Unpin,
                {
                    #body
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics #krate::Encode for #name #ty_generics #where_clause {
                fn encode<W>(&self, writer: &mut W) -> #krate::Result<usize>
                where
                    W: ::std::io::Write,
                {
                    #body
                }
            }
        }
    }
}

fn build_decode_impl(input: &DeriveInput, krate: &TokenStream2, is_async: bool) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let await_tok = await_tok(is_async);

    let body = match &input.data {
        Data::Struct(s) => {
            let (stmts, construction) = decode::struct_decode_stmts(&s.fields, krate, is_async);
            quote! {
                #(#stmts)*
                Ok(#name #construction)
            }
        }
        Data::Enum(e) => {
            let variants: Vec<&Variant> = e.variants.iter().collect();
            let discriminants: Vec<u8> = variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| variant_discriminant(variant, idx))
                .collect();

            check_unique_discriminants(name, &variants, &discriminants);

            let arms: Vec<_> = variants
                .iter()
                .zip(&discriminants)
                .map(|(variant, &discriminant)| {
                    decode::enum_decode_arm(
                        name,
                        &variant.ident,
                        &variant.fields,
                        discriminant,
                        krate,
                        is_async,
                    )
                })
                .collect();

            quote! {
                let discriminant = <u8 as #krate::Decode>::decode(reader)#await_tok?;
                match discriminant {
                    #(#arms),*,
                    other => Err(#krate::Error::InvalidDiscriminant(other)),
                }
            }
        }
        Data::Union(_) => panic!("Decode derive does not support unions"),
    };

    if is_async {
        quote! {
            impl #impl_generics #krate::Decode for #name #ty_generics #where_clause {
                async fn decode<R>(reader: &mut R) -> #krate::Result<Self>
                where
                    Self: Sized,
                    R: #krate::AsyncRead + Unpin,
                {
                    #body
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics #krate::Decode for #name #ty_generics #where_clause {
                fn decode<R>(reader: &mut R) -> #krate::Result<Self>
                where
                    Self: Sized,
                    R: ::std::io::Read,
                {
                    #body
                }
            }
        }
    }
}

mod decode {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::Fields;

    fn await_tok(is_async: bool) -> TokenStream2 {
        if is_async {
            quote! { .await }
        } else {
            quote! {}
        }
    }

    pub(crate) fn struct_decode_stmts(
        fields: &Fields,
        krate: &TokenStream2,
        is_async: bool,
    ) -> (Vec<TokenStream2>, TokenStream2) {
        let await_tok = await_tok(is_async);
        match fields {
            Fields::Named(named) => {
                let stmts: Vec<_> = named
                    .named
                    .iter()
                    .map(|f| {
                        let ident = f.ident.as_ref().unwrap();
                        let ty = &f.ty;
                        quote! { let #ident = <#ty as #krate::Decode>::decode(reader)#await_tok?; }
                    })
                    .collect();
                let field_names: Vec<_> = named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                (stmts, quote! { { #(#field_names),* } })
            }
            Fields::Unnamed(unnamed) => {
                let vars: Vec<syn::Ident> = (0..unnamed.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();
                let stmts: Vec<_> = unnamed
                    .unnamed
                    .iter()
                    .zip(&vars)
                    .map(|(f, var)| {
                        let ty = &f.ty;
                        quote! { let #var = <#ty as #krate::Decode>::decode(reader)#await_tok?; }
                    })
                    .collect();
                (stmts, quote! { ( #(#vars),* ) })
            }
            Fields::Unit => (vec![], quote! {}),
        }
    }

    /// Generates one match arm of `match discriminant` inside `decode`.
    pub(crate) fn enum_decode_arm(
        enum_name: &syn::Ident,
        variant: &syn::Ident,
        fields: &Fields,
        discriminant: u8,
        krate: &TokenStream2,
        is_async: bool,
    ) -> TokenStream2 {
        let await_tok = await_tok(is_async);
        match fields {
            Fields::Unit => quote! {
                #discriminant => Ok(#enum_name::#variant)
            },
            Fields::Named(named) => {
                let stmts: Vec<_> = named
                    .named
                    .iter()
                    .map(|f| {
                        let ident = f.ident.as_ref().unwrap();
                        let ty = &f.ty;
                        quote! { let #ident = <#ty as #krate::Decode>::decode(reader)#await_tok?; }
                    })
                    .collect();
                let bindings: Vec<_> = named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                quote! {
                    #discriminant => {
                        #(#stmts)*
                        Ok(#enum_name::#variant { #(#bindings),* })
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let vars: Vec<syn::Ident> = (0..unnamed.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();
                let stmts: Vec<_> = unnamed
                    .unnamed
                    .iter()
                    .zip(&vars)
                    .map(|(f, var)| {
                        let ty = &f.ty;
                        quote! { let #var = <#ty as #krate::Decode>::decode(reader)#await_tok?; }
                    })
                    .collect();
                quote! {
                    #discriminant => {
                        #(#stmts)*
                        Ok(#enum_name::#variant(#(#vars),*))
                    }
                }
            }
        }
    }
}

mod encode {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::Fields;

    fn await_tok(is_async: bool) -> TokenStream2 {
        if is_async {
            quote! { .await }
        } else {
            quote! {}
        }
    }

    pub(crate) fn struct_encode_stmts(
        fields: &Fields,
        krate: &TokenStream2,
        is_async: bool,
    ) -> Vec<TokenStream2> {
        let await_tok = await_tok(is_async);
        match fields {
            Fields::Named(named) => named
                .named
                .iter()
                .map(|f| {
                    let ident = f.ident.as_ref().unwrap();
                    quote! { total += #krate::Encode::encode(&self.#ident, writer)#await_tok?; }
                })
                .collect(),
            Fields::Unnamed(unnamed) => unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let idx = syn::Index::from(i);
                    quote! { total += #krate::Encode::encode(&self.#idx, writer)#await_tok?; }
                })
                .collect(),
            Fields::Unit => vec![],
        }
    }

    /// Generates one match arm of `match self` inside `encode`.
    pub(crate) fn enum_encode_arm(
        enum_name: &syn::Ident,
        variant: &syn::Ident,
        fields: &Fields,
        discriminant: u8,
        krate: &TokenStream2,
        is_async: bool,
    ) -> TokenStream2 {
        let await_tok = await_tok(is_async);
        // The discriminant encode statement is the same for all three field styles.
        let disc_stmt = quote! {
            total += #krate::Encode::encode(&(#discriminant as u8), writer)#await_tok?;
        };

        match fields {
            Fields::Unit => quote! {
                #enum_name::#variant => { #disc_stmt }
            },
            Fields::Named(named) => {
                let bindings: Vec<_> = named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let field_stmts = bindings.iter().map(|b| {
                    quote! { total += #krate::Encode::encode(#b, writer)#await_tok?; }
                });
                quote! {
                    #enum_name::#variant { #(#bindings),* } => {
                        #disc_stmt
                        #(#field_stmts)*
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let vars: Vec<syn::Ident> = (0..unnamed.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();
                let field_stmts = vars.iter().map(|v| {
                    quote! { total += #krate::Encode::encode(#v, writer)#await_tok?; }
                });
                quote! {
                    #enum_name::#variant(#(#vars),*) => {
                        #disc_stmt
                        #(#field_stmts)*
                    }
                }
            }
        }
    }
}

fn broto_crate() -> TokenStream2 {
    if std::env::var("CARGO_CRATE_NAME").as_deref() == Ok("broto") {
        quote! { crate }
    } else {
        quote! { ::broto }
    }
}

fn await_tok(is_async: bool) -> TokenStream2 {
    if is_async {
        quote! { .await }
    } else {
        quote! {}
    }
}
