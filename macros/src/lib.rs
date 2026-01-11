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
    ::zero::async_runtime::run(async_main())?;
    Ok(())
}}"#,
        block
    )
    .parse()
    .unwrap()
}

fn parse_attrs(attrs: TokenStream) -> String {
    let mut items = attrs.into_iter().peekable();
    let mut tokens = String::new();

    while items.peek().is_some() {
        let is_special_ident = items.peek().is_some_and(|i| match i {
            &TokenTree::Ident(_) => true,
            _ => false,
        });

        let key = if is_special_ident {
            let mut tokens = String::from("\"");
            while items.peek().is_some_and(|i| match i {
                TokenTree::Ident(_) => true,
                TokenTree::Punct(p) => p.to_string().as_str() == "-",
                _ => false,
            }) {
                let token = match items.next() {
                    Some(TokenTree::Ident(i)) => format!("{}", i),
                    Some(TokenTree::Punct(p)) => match p.to_string().as_str() {
                        "-" => String::from("-"),
                        _ => unreachable!("Expected ident or \"-\" for attribute key"),
                    },
                    _ => {
                        unreachable!("Expected ident or \"-\" for attribute key")
                    }
                };

                tokens.push_str(&token);
            }

            tokens.push('"');

            tokens
        } else {
            match items.next() {
                Some(TokenTree::Ident(_)) => unreachable!(
                    "Token was check for ident earlier, not fully sure how we got here..."
                ),
                Some(TokenTree::Literal(l)) => format!("{}", l),
                Some(TokenTree::Group(g)) => format!("{}", g),
                Some(TokenTree::Punct(p)) => {
                    panic!("Expected attribute key, found punctuation: {}", p)
                }
                None => break,
            }
        };
        expect!(items, ":");
        let val = match items.next() {
            Some(TokenTree::Ident(i)) => format!("{}", i),
            Some(TokenTree::Literal(l)) => format!("{}", l),
            Some(TokenTree::Group(g)) => format!("{}", g),
            Some(TokenTree::Punct(p)) => panic!("Expected attribute val, found punctuation: {}", p),
            None => break,
        };

        tokens.push_str(&format!(".set_attr({}.into(),{}.into())", key, val));

        match items.next() {
            Some(TokenTree::Ident(_)) => panic!("Expected punctuation or end of html attributes"),
            Some(TokenTree::Literal(_)) => panic!("Expected punctuation or end of html attributes"),
            Some(TokenTree::Group(_)) => panic!("Expected punctuation or end of html attributes"),
            Some(TokenTree::Punct(_)) => {}
            None => break,
        }
    }
    eprintln!("{}", tokens);

    tokens
}

#[proc_macro]
pub fn html(item: TokenStream) -> TokenStream {
    eprintln!("{:#?}", item);
    let mut items = item.into_iter().peekable();

    let mut tokens = String::new();
    while items.peek().is_some() {
        let tag_name = match items.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(TokenTree::Literal(l)) => {
                return format!("Into::<::zero::html::Markup>::into({})", l)
                    .parse()
                    .unwrap();
            }
            Some(TokenTree::Group(g)) => {
                return g.stream();
            }
            Some(t) => panic!("Expected TagType, found {:#?}", t),
            None => return "()".parse().unwrap(),
        };

        match items.peek() {
            Some(TokenTree::Punct(p)) => {
                if p.to_string() == ";" {
                    tokens.push_str(&format!(
                        "{{::zero::html::Tag::new(::zero::html::TagType::{})}},\n",
                        tag_name
                    ));
                    items.next();
                    continue;
                }
            }
            _ => {}
        }

        let attrs = match items.next() {
            Some(TokenTree::Group(g)) => parse_attrs(g.stream()),
            Some(_) => panic!("Expected Grouping for Attributes"),
            None => String::new(),
        };

        let inner = match items.next() {
            Some(TokenTree::Group(g)) => html(g.stream()),
            Some(_) => panic!("Expected Grouping for inner Markup"),
            None => "".parse().unwrap(),
        };

        tokens.push_str(&format!(
            "{{::zero::html::Tag::new(::zero::html::TagType::{}){}.set_content({}) }},\n",
            tag_name, attrs, inner
        ));
    }

    let s = format!("Into::<::zero::html::Markup>::into(vec![{}])", tokens);
    eprintln!("{}", s);

    s.parse().unwrap()
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
