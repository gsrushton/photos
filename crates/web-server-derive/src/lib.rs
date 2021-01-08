extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(ApiError, attributes(status_code))]
pub fn derive_response_error(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;

    TokenStream::from(quote::quote! {
        impl ::actix_web::ResponseError for #name {
            fn status_code(&self) -> ::actix_web::http::StatusCode {
                ::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }

            fn error_response(&self) -> ::actix_web::web::HttpResponse<::actix_web::body::Body> {
                ::actix_web::web::HttpResponse::build(self.status_code())
                    .json(::photos_web_core::ErrorDesc::from(self as &dyn std::error::Error))
            }
        }
    })
}
