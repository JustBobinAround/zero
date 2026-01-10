// async fn async_main() -> Result<(), $err_ty> $b
// fn main() -> Result<(), ()> {
//     $crate::async_runtime::Runtime::run_async(async_main()).map_err(|_| ())?;

//     Ok(())
// }
// Ident {
//     ident: "fn",
//     span: #0 bytes(687..689),
// }
// Ident {
//     ident: "main",
//     span: #0 bytes(690..694),
// }
// Group {
//     delimiter: Parenthesis,
//     stream: TokenStream [],
//     span: #0 bytes(694..696),
// }
// Group {
//     delimiter: Brace,
//     stream: TokenStream [
//         Ident {
//             ident: "let",
//             span: #0 bytes(703..706),
//         },
//         Ident {
//             ident: "i",
//             span: #0 bytes(707..708),
//         },
//         Punct {
//             ch: '=',
//             spacing: Alone,
//             span: #0 bytes(709..710),
//         },
//         Literal {
//             kind: Integer,
//             symbol: "0",
//             suffix: None,
//             span: #0 bytes(711..712),
//         },
//         Punct {
//             ch: ';',
//             spacing: Alone,
//             span: #0 bytes(712..713),
//         },
//     ],
//     span: #0 bytes(697..715),
// }
