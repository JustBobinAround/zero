use proc_macro::{TokenStream, TokenTree, token_stream::IntoIter};

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
        while !self.is_punct(">") && self.has_tokens_left() {
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

    pub fn to_token_stream(s: Vec<TokenTree>) -> TokenStream {
        s.into_iter().map(|tt| tt).collect()
    }
}
