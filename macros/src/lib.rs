mod extract_macro;
mod token_parser;

use std::{collections::HashMap, iter::Peekable};

use crate::{extract_macro::ExtractType, token_parser::TokenParser};
use proc_macro::{TokenStream, TokenTree, token_stream::IntoIter};

#[proc_macro]
pub fn impl_extract_permutations(_item: TokenStream) -> TokenStream {
    let choices = ExtractType::all_choices();
    ExtractType::make_combinations(choices).parse().unwrap()
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

    s.parse().unwrap()
}
fn parse_enum(parser: TokenParser, tokens: Vec<TokenTree>) -> TokenStream {
    unimplemented!("Enum serialization is not supported at this time")
}

fn parse_struct(mut parser: TokenParser, mut tokens: Vec<TokenTree>) -> TokenStream {
    let struct_name = parser
        .consume_if(|p| p.is_any_ident())
        .expect("structure ident");
    tokens.push(struct_name);
    if parser.is_punct("<") {
        tokens = parser
            .consume_generics_impl(tokens)
            .expect("Expected valid generics");
    }

    let group = match parser.consume() {
        Some(TokenTree::Group(g)) => g,
        _ => panic!("expected inner structure"),
    };

    let mut inner_parser = TokenParser::new(group.stream());

    let mut struct_map = HashMap::new();

    while inner_parser.has_tokens_left() {
        let ident = inner_parser
            .consume_if(|p| p.is_any_ident())
            .expect("a field name")
            .to_string();

        let (is_pub, ident) = if ident == "pub" {
            let ident = inner_parser
                .consume_if(|p| p.is_any_ident())
                .expect("a field name")
                .to_string();

            (true, ident)
        } else {
            (false, ident)
        };

        inner_parser
            .consume_if(|p| p.is_punct(":"))
            .expect("a ':' after field name");

        let ty = inner_parser.consume_type().expect("a field type");

        struct_map.insert(ident, (is_pub, ty));

        let _ = inner_parser.consume_if(|p| p.is_punct(","));
    }

    // tokens.push(group);
    dbg!(&struct_map);

    "".parse().unwrap()
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(items: TokenStream) -> TokenStream {
    let mut parser = TokenParser::new(items);
    let mut tokens = Vec::new();

    match parser.consume_if(|p| p.is_ident("pub")) {
        Ok(t) => tokens.push(t),
        Err(_) => {}
    }

    match parser.consume_if(|p| p.is_ident("struct")) {
        Ok(s) => {
            tokens.push(s);
            parse_struct(parser, tokens)
        }
        Err(_) => match parser.consume_if(|p| p.is_ident("enum")) {
            Ok(s) => {
                tokens.push(s);
                parse_enum(parser, tokens)
            }

            Err(_) => panic!("Expected a struct or enum"),
        },
    }
}
