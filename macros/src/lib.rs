mod extract_macro;
mod token_parser;

use std::iter::Peekable;

use crate::{extract_macro::ExtractType, token_parser::TokenParser};
use proc_macro::{TokenStream, TokenTree, token_stream::IntoIter};

#[proc_macro]
pub fn impl_extract_permutations(_item: TokenStream) -> TokenStream {
    let choices = ExtractType::all_choices();
    ExtractType::make_combinations(choices).parse().unwrap()
}

macro_rules! expect {
    ($items: expr, $s:literal) => {{
        let tmp = $items.next().is_some_and(|i| {
            eprintln!("{:#?}", i);
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
    let mut parser = TokenParser::new(item);

    parser
        .consume_if(|p| p.is_ident("async"))
        .expect("async token");
    parser.consume_if(|p| p.is_ident("fn")).expect("fn token");
    parser
        .consume_if(|p| p.is_ident("main"))
        .expect("main token");
    parser
        .consume_if(|p| p.is_any_group())
        .expect("empty fn parameters");
    parser.consume_if(|p| p.is_punct("-")).expect("-> token");
    parser.consume_if(|p| p.is_punct(">")).expect("-> token");
    let return_type: String = parser
        .consume_type()
        .expect("a return type")
        .into_iter()
        .map(|t| t.to_string())
        .collect();
    let function_block = parser
        .consume_if(|p| p.is_any_group())
        .expect("main function block");

    let s = format!(
        r#"async fn async_main() -> {} {}
fn main() -> {} {{
    ::zero::async_runtime::run(async_main())
}}"#,
        &return_type, function_block, &return_type,
    );
    eprintln!("{}", s);
    s.parse().expect("Failed to parse proc macro str")
}
fn parse_attrs(attrs: TokenStream) -> Result<String, ()> {
    let mut parser = TokenParser::new(attrs);

    let mut tokens = String::new();

    while parser.has_tokens_left() {
        let key = if parser.is_any_ident() {
            let name: String = parser
                .consume_while(|p| p.is_any_ident() || p.is_punct("-"))
                .into_iter()
                .map(|t| t.to_string())
                .collect();
            format!("\"{}\"", name)
        } else {
            if parser.is_any_punct() || parser.is_any_ident() {
                panic!("Expected attribute key, found punctuation or ident");
            } else if let Some(t) = parser.consume() {
                t.to_string()
            } else {
                break;
            }
        };
        parser.consume_if(|p| p.is_punct(":"))?;
        if parser.is_any_punct() {
            panic!("Expected attribute val, found punctuation");
        }
        let val = match parser.consume_as_str() {
            Some(s) => s,
            None => break,
        };
        tokens.push_str(&format!(".set_attr({}.into(),{}.into())", key, val));

        if !parser.has_tokens_left() {
            break;
        } else if parser.is_any_punct() {
            parser.consume();
        } else {
            panic!("Expected punctuation or end of html attributes")
        }
    }

    Ok(tokens)
}

#[proc_macro]
pub fn html(item: TokenStream) -> TokenStream {
    let mut parser = TokenParser::new(item);

    let mut tokens = String::new();
    while parser.has_tokens_left() {
        let tag_name = match parser.consume() {
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

        if parser.is_punct(";") {
            tokens.push_str(&format!(
                "{{::zero::html::Tag::new(::zero::html::TagType::{})}},\n",
                tag_name
            ));
            parser.consume();
            continue;
        }

        let tt = parser.consume();

        let attrs = if let Some(TokenTree::Group(g)) = tt {
            parse_attrs(g.stream()).expect("expected valid attribute")
        } else if tt.is_some() {
            panic!("Expected Grouping for Attributes")
        } else {
            String::new()
        };

        let tt = parser.consume();

        let inner = if let Some(TokenTree::Group(g)) = tt {
            html(g.stream())
        } else if tt.is_some() {
            panic!("Expected Grouping for inner markup")
        } else {
            "".parse().unwrap()
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

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(items: TokenStream) -> TokenStream {
    let mut items = items.into_iter().peekable();

    let has_pub = items.peek().is_some_and(|i| match i {
        TokenTree::Ident(i) => i.to_string() == "pub",
        _ => false,
    });

    let visiblity = if has_pub {
        items.next();
        "pub "
    } else {
        ""
    };

    let is_struct = items.peek().is_some_and(|i| match i {
        TokenTree::Ident(i) => i.to_string() == "struct",
        _ => false,
    });

    let is_enum = items.peek().is_some_and(|i| match i {
        TokenTree::Ident(i) => i.to_string() == "enum",
        _ => false,
    });

    let item_ty = if is_struct {
        items.next();
        "struct "
    } else if is_enum {
        unimplemented!("Haven't created a serialization standard for enums yet")
        // items.next();
        // "enum "
    } else {
        panic!("Expected an enum or struct")
    };

    for item in items {
        eprintln!("{:#?}", item);
    }
    "".parse().unwrap()
}
