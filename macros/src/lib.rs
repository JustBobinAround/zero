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

fn parse_struct(mut parser: TokenParser, is_public: bool) -> TokenStream {
    let data_struct = parser.consume_struct(is_public).expect("a valid struct");

    let struct_name = data_struct.name();
    let generics = data_struct.generics();
    if generics.len() > 0 {
        // TODO: add generic support
        unimplemented!("deriving deserialize with generics is not currently supported");
    }
    let fields: String = data_struct
        .fields()
        .iter()
        .map(|(name, field_data)| {
            format!(
                "{}: match dh.remove(\"{}\") {{
                    Some(dh) => <{}>::deserialize(dh)?,
                    None => return Err(())
                }},",
                name,
                name,
                field_data.ty_str()
            )
        })
        .collect();

    let output = format!(
        r#"impl ::zero::serializer::Deserialize for {} {{
    fn deserialize(dh: ::zero::serializer::DataHolder) -> Result<Self, ()> {{
        match dh {{
            ::zero::serializer::DataHolder::Struct(mut dh) => Ok(Self {{
                {}
            }}),
            _ => Err(())
        }}
    }}
}}"#,
        struct_name, fields
    );

    // tokens.push(group);
    eprintln!("{}", &output);

    output.parse().unwrap()
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(items: TokenStream) -> TokenStream {
    let mut parser = TokenParser::new(items);

    let is_pub = parser.is_ident("pub");
    if is_pub {
        parser.consume();
    }

    match parser.consume_if(|p| p.is_ident("struct")) {
        Ok(s) => parse_struct(parser, is_pub),
        Err(_) => match parser.consume_if(|p| p.is_ident("enum")) {
            Ok(s) => {
                unimplemented!("Enum serialization is not supported at this time")
            }

            Err(_) => panic!("Expected a struct or enum"),
        },
    }
}
