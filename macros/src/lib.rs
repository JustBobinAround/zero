use proc_macro::TokenStream;

#[proc_macro]
pub fn html(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(FromRequest)]
pub fn derive_from_request(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

/*
pub fn some_route<>
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
