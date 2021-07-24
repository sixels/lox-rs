use lox_rs::lexer::{Lexer, TokenKind};

#[test]
fn lexer_from_file() {
    let lexer = Lexer::from_file("tests/lox/hello.lox").unwrap();
    // including file directly
    let file = include_bytes!("lox/hello.lox");

    let buffer = lexer.buffer;
    assert_eq!(buffer, String::from_utf8_lossy(file));
    assert_eq!(buffer.capacity(), file.len() + 1);
}

#[test]
fn tokenize_hello() {
    let lexer = Lexer::from_file("tests/lox/hello.lox").unwrap();

    let hello_tokens = vec![
        TokenKind::Print,
        TokenKind::String("Hello, World!".to_string()),
        TokenKind::SemiColon,
    ];

    let mut tokens = lexer.into_iter();
    let token_vec = tokens.clone().collect::<Vec<TokenKind>>();

    assert_eq!(token_vec, hello_tokens);
    assert!(tokens.nth(3).is_none());
}

#[test]
fn tokenize_all() {
    let fibonacci = r#"
        fun main() {
            fun fibonacci(n) {
                if (n < 2) {
                    return 0;
                } else {
                    return fibonacci(n-1) + fibonacci(n-2);
                }
            }

            var n = 10;

            print "The fibonacci of 10 is:";
            print n;
        }

        while(true) {
            main();
        }
    "#;

    let lexer = Lexer::new(fibonacci.to_string());

    println!("{:?}", lexer.collect::<Vec<TokenKind>>());
}