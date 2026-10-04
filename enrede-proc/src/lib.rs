use enrede::EncString;
use enrede::encoding::Enc;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr};

fn str_inner(lit: TokenStream) -> Result<TokenStream, syn::Error> {
    let lit = syn::parse::<LitStr>(lit.clone())?;

    let enc = Enc::from_shorthand(lit.suffix()).ok_or_else(|| {
        syn::Error::new(
            lit.span(),
            format!("Invalid shorthand encoding {}", lit.suffix()),
        )
    })?;
    let s = EncString::from_std(lit.value())
        .recode(enc)
        .map_err(|e| syn::Error::new(lit.span(), e))?;
    let s = s.as_bytes();

    let ty_name = Ident::new(enc.type_name(), Span::call_site());

    Ok(quote!(unsafe { enrede::Str::<enrede::encoding::#ty_name>::from_bytes_unchecked(&[#( #s, )*]) }).into())
}

#[proc_macro]
pub fn str(lit: TokenStream) -> TokenStream {
    match str_inner(lit.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
