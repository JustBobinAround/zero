mod extract_macro;
mod token_parsing;
use crate::extract_macro::ExtractType;
use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn impl_extract_permutations(_item: TokenStream) -> TokenStream {
    let choices = ExtractType::all_choices();
    ExtractType::make_combinations(choices).parse().unwrap()
}

macro_rules! expect {
    ($items: expr, $s:literal) => {{
        let tmp = $items.next().is_some_and(|i| {
            let i = i.to_string();
            i == $s
        });
        if !tmp {
            panic!("Failed to find {}", $s);
        }
    }};
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut items = item.into_iter();

    expect!(items, "async");
    expect!(items, "fn");
    expect!(items, "main");
    expect!(items, "()");
    expect!(items, "-");
    expect!(items, ">");
    expect!(items, "Result");
    expect!(items, "<");
    expect!(items, "()");
    expect!(items, ",");
    expect!(items, "()");
    expect!(items, ">");
    let block = items.next().expect("Expected function block");
    // for mut i in item {
    //     eprintln!("{:#?}", i);
    //     let s = match i {
    //         TokenTree::Group(g) => g.to_string(),
    //         TokenTree::Ident(i) => i.to_string(),
    //         TokenTree::Punct(p) => p.to_string(),
    //         TokenTree::Literal(l) => l.to_string(),
    //     };
    //     eprintln!("{}", s);
    // }
    // async fn async_main() -> Result<(), $err_ty> $b
    // fn main() -> Result<(), ()> {
    //     $crate::async_runtime::Runtime::run_async(async_main()).map_err(|_| ())?;

    //     Ok(())
    // }

    format!(
        r#"async fn async_main() -> Result<(), ()> {}
fn main() -> Result<(), ()> {{
    zero::async_runtime::run(async_main())?;
    Ok(())
}}"#,
        block
    )
    .parse()
    .unwrap()
}

#[proc_macro]
pub fn html(item: TokenStream) -> TokenStream {
    for i in item {
        dbg!(&i);
        match i {
            TokenTree::Group(g) => {
                eprintln!("{}", g);
            }
            _ => {}
        }
    }
    "".parse().unwrap()
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
    // use super::*;

    #[test]
    fn it_works() {}
}
