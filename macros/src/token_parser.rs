use proc_macro::{TokenStream, TokenTree, token_stream::IntoIter};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct StructField {
    name: Arc<String>,
    is_public: bool,
    ty: Vec<TokenTree>,
}

impl StructField {
    pub fn ty_str(&self) -> String {
        self.ty.iter().map(|t| t.to_string()).collect()
    }
}

#[derive(Debug)]
pub struct Struct {
    is_public: bool,
    name: String,
    generics: Vec<TokenTree>,
    fields: HashMap<Arc<String>, StructField>,
}

impl Struct {
    pub fn new(is_public: bool, name: String, generics: Vec<TokenTree>) -> Self {
        Struct {
            is_public,
            name,
            generics,
            fields: HashMap::new(),
        }
    }

    pub fn is_public(&self) -> bool {
        self.is_public
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn generics(&self) -> &Vec<TokenTree> {
        &self.generics
    }

    pub fn fields(&self) -> &HashMap<Arc<String>, StructField> {
        &self.fields
    }

    pub fn add_field(&mut self, name: String, is_public: bool, ty: Vec<TokenTree>) {
        let name = Arc::new(name);
        self.fields.insert(
            name.clone(),
            StructField {
                name,
                is_public,
                ty,
            },
        );
    }
}

pub struct TokenParser {
    stream: IntoIter,
    peek: Option<TokenTree>,
}

macro_rules! impl_is_any {
    ($name:ident, $tt:tt) => {
        pub fn $name(&mut self) -> bool {
            match self.peek() {
                Some(TokenTree::$tt(_)) => true,
                _ => false,
            }
        }
    };
}

impl TokenParser {
    pub fn new(stream: TokenStream) -> Self {
        let stream = stream.into_iter();
        Self { stream, peek: None }
    }

    pub fn peek(&mut self) -> &Option<TokenTree> {
        if self.peek.is_none() {
            self.peek = self.stream.next();
            &self.peek
        } else {
            &self.peek
        }
    }

    pub fn consume(&mut self) -> Option<TokenTree> {
        self.peek();
        self.peek.take()
    }

    pub fn consume_as_str(&mut self) -> Option<String> {
        self.peek();
        match self.peek.take() {
            Some(TokenTree::Ident(i)) => Some(format!("{}", i)),
            Some(TokenTree::Literal(l)) => Some(format!("{}", l)),
            Some(TokenTree::Group(g)) => Some(format!("{}", g)),
            Some(TokenTree::Punct(p)) => Some(format!("{}", p)),
            None => None,
        }
    }

    pub fn has_tokens_left(&mut self) -> bool {
        self.peek().is_some()
    }

    pub fn is_void(&mut self) -> bool {
        match self.peek() {
            Some(TokenTree::Group(g)) => g.to_string() == "()",
            _ => false,
        }
    }

    impl_is_any!(is_any_ident, Ident);
    impl_is_any!(is_any_group, Group);
    impl_is_any!(is_any_punct, Punct);
    impl_is_any!(is_any_literal, Literal);

    pub fn is_ident(&mut self, s: &str) -> bool {
        match self.peek() {
            Some(TokenTree::Ident(i)) => i.to_string() == s,
            _ => false,
        }
    }

    pub fn is_punct(&mut self, s: &str) -> bool {
        match self.peek() {
            Some(TokenTree::Punct(p)) => p.to_string() == s,
            _ => false,
        }
    }

    pub fn is_literal(&mut self, s: &str) -> bool {
        match self.peek() {
            Some(TokenTree::Literal(l)) => l.to_string() == format!("\"{}\"", s),
            _ => false,
        }
    }

    pub fn consume_while<F: Fn(&mut Self) -> bool>(&mut self, f: F) -> Vec<TokenTree> {
        let mut s = Vec::new();

        while f(self) {
            if let Some(tt) = self.consume() {
                s.push(tt);
            } else {
                break;
            }
        }

        s
    }

    pub fn consume_if<F: Fn(&mut Self) -> bool>(&mut self, f: F) -> Result<TokenTree, ()> {
        if f(self) {
            if let Some(tt) = self.consume() {
                Ok(tt)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn consume_sub_type(&mut self, mut tokens: Vec<TokenTree>) -> Result<Vec<TokenTree>, ()> {
        tokens.push(self.consume_if(|p| p.is_punct("<"))?);
        while self.has_tokens_left() {
            tokens = self.consume_type_impl(tokens)?;
            if let Ok(t) = self.consume_if(|p| p.is_punct(",")) {
                tokens.push(t)
            } else {
                break;
            }
        }

        tokens.push(self.consume_if(|p| p.is_punct(">"))?);
        Ok(tokens)
    }

    fn consume_type_impl(&mut self, mut tokens: Vec<TokenTree>) -> Result<Vec<TokenTree>, ()> {
        if let Ok(t) = self.consume_if(|p| p.is_punct("&")) {
            tokens.push(t);
            tokens.push(self.consume_if(|p| p.is_punct("'"))?);
            tokens.push(self.consume_if(|p| p.is_any_ident())?);
        }

        tokens.push(self.consume_if(|p| p.is_any_ident() || p.is_void())?);

        if self.is_punct("<") {
            self.consume_sub_type(tokens)
        } else {
            Ok(tokens)
        }
    }

    pub fn consume_type(&mut self) -> Result<Vec<TokenTree>, ()> {
        let tokens = Vec::new();
        self.consume_type_impl(tokens)
    }

    pub fn consume_generics_impl(
        &mut self,
        mut tokens: Vec<TokenTree>,
    ) -> Result<Vec<TokenTree>, ()> {
        tokens.push(self.consume_if(|p| p.is_punct("<"))?);
        while self.has_tokens_left() && !self.is_punct(">") {
            if let Ok(t) = self.consume_if(|p| p.is_punct("'")) {
                tokens.push(t);
                tokens.push(self.consume_if(|p| p.is_any_ident())?);
            } else if let Ok(t) = self.consume_if(|p| p.is_any_ident()) {
                tokens.push(t);
                if let Ok(t) = self.consume_if(|p| p.is_punct(":")) {
                    tokens.push(t);
                    while self.has_tokens_left() {
                        if let Ok(t) = self.consume_if(|p| p.is_any_ident()) {
                            tokens.push(t);
                            if let Ok(t) = self.consume_if(|p| p.is_punct("+")) {
                                tokens.push(t);
                            } else if let Ok(g) = self.consume_if(|p| p.is_any_group()) {
                                unimplemented!("parsing of closures")
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            if !self.is_punct(",") {
                break;
            } else {
                self.consume();
            }
        }
        tokens.push(self.consume_if(|p| p.is_punct(">"))?);

        Ok(tokens)
    }

    pub fn consume_generics(&mut self) -> Result<Vec<TokenTree>, ()> {
        let tokens = Vec::new();
        self.consume_generics_impl(tokens)
    }

    pub fn consume_struct(&mut self, is_public: bool) -> Result<Struct, ()> {
        let name = self.consume_if(|p| p.is_any_ident())?.to_string();

        let generics = if self.is_punct("<") {
            self.consume_generics()?
        } else {
            Vec::new()
        };

        let mut data_struct = Struct::new(is_public, name, generics);

        let fields = match self.consume() {
            Some(TokenTree::Group(g)) => g,
            _ => return Err(()),
        };

        let mut inner_parser = TokenParser::new(fields.stream());

        while inner_parser.has_tokens_left() {
            let ident = inner_parser.consume_if(|p| p.is_any_ident())?.to_string();

            let is_pub = ident == "pub";

            let ident = if is_pub {
                inner_parser.consume_if(|p| p.is_any_ident())?.to_string()
            } else {
                ident
            };

            inner_parser.consume_if(|p| p.is_punct(":"))?;

            let ty = inner_parser.consume_type()?;

            data_struct.add_field(ident, is_pub, ty);

            let _ = inner_parser.consume_if(|p| p.is_punct(","));
        }

        Ok(data_struct)
    }

    pub fn to_token_stream(s: Vec<TokenTree>) -> TokenStream {
        s.into_iter().map(|tt| tt).collect()
    }
}
