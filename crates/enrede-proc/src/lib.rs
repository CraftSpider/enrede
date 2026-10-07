//! Proc macros for [`enrede`](https://docs.rs/enrede).

use enrede_core::EncString;
use enrede_core::encoding::Enc;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitCStr, LitStr};

enum Str {
    Str(LitStr),
    CStr(LitCStr),
}

impl Str {
    fn span(&self) -> Span {
        match self {
            Self::Str(s) => s.span(),
            Self::CStr(s) => s.span(),
        }
    }

    fn suffix(&self) -> &str {
        match self {
            Self::Str(s) => s.suffix(),
            Self::CStr(s) => s.suffix(),
        }
    }
}

impl Parse for Str {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let l = input.lookahead1();

        if l.peek(LitStr) {
            Ok(Str::Str(input.parse()?))
        } else if l.peek(LitCStr) {
            Ok(Str::CStr(input.parse()?))
        } else {
            Err(l.error())
        }
    }
}

fn str_inner(lit: TokenStream) -> Result<TokenStream, syn::Error> {
    let lit = syn::parse::<Str>(lit.clone())?;

    let enc = Enc::from_shorthand(lit.suffix()).ok_or_else(|| {
        syn::Error::new(
            lit.span(),
            format!("Invalid shorthand encoding {}", lit.suffix()),
        )
    })?;

    let ty_name = Ident::new(enc.type_name(), Span::call_site());

    let toks = match &lit {
        Str::Str(s) => {
            let s = EncString::from_std(s.value())
                .recode(enc)
                .map_err(|e| syn::Error::new(lit.span(), e))?
                .into_bytes();
            quote!(unsafe { enrede::Str::<enrede::encoding::#ty_name>::from_bytes_unchecked(&[#( #s, )*]) })
        }
        Str::CStr(s) => {
            let s = EncString::from_bytes(Enc::Utf8, s.value().into_bytes())
                .map_err(|e| syn::Error::new(lit.span(), e))?
                .recode(enc)
                .map_err(|e| syn::Error::new(lit.span(), e))?;
            if s.as_bytes().contains(&0) {
                return Err(syn::Error::new(
                    lit.span(),
                    "C-String contains non-zero bytes",
                ));
            }
            let mut b = s.into_bytes();
            b.push(0);
            quote!(unsafe { enrede::CStr::<enrede::encoding::#ty_name>::from_bytes_with_nul_unchecked(&[#( #b, )*]) })
        }
    };

    Ok(toks.into())
}

/// Macro to create arbitrary encoding string literals at compile-time.
///
/// See [`enrede::str`](https://docs.rs/enrede/macro.str.html) for further documentation.
#[proc_macro]
pub fn str(lit: TokenStream) -> TokenStream {
    match str_inner(lit.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
