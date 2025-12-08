use std::{ffi::CString, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use libffi::middle::{Arg, Type};
use logos::{Lexer, Logos};
use rust_plug::handler::DllManager;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manager = DllManager::new(cli.path)?;

    let mut stdin = String::new();
    loop {
        std::io::stdin()
            .read_line(&mut stdin)
            .context("read input from stdin")?;

        let input = stdin.trim();
        if input == ":q" {
            break;
        }

        let func_end = input.find(' ').unwrap_or(input.len());

        let func = &input[..func_end];

        // Is sad to allocate a vector for args
        // because we need to assume that `CString` is not dropped during the call
        let values = Token::lexer(&input[func_end..])
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow::anyhow!("Failed to parse arguments"))?;

        let args = values.iter().map(parse_args);

        unsafe { manager.call_func::<()>(func, args, libffi::middle::Type::void())? };

        stdin.clear();
    }

    Ok(())
}

fn parse_args(arg: &Token) -> (Type, Arg<'_>) {
    match arg {
        Token::String(c) => (Type::pointer(), Arg::new(c)),
        Token::Float(f) => (Type::f32(), Arg::new(f)),
        Token::Integer(i) => (Type::i32(), Arg::new(i)),
        Token::Hex(h) => (Type::u32(), Arg::new(h)),
    }
}

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ ,\t\n\f]+")]
enum Token {
    #[regex(
        r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#,
        get_string_content
    )]
    String(CString),

    #[regex(r"0x[0-9a-fA-F]+", |lex| u32::from_str_radix(&lex.slice()[2..], 16).unwrap())]
    Hex(u32),

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap(), priority = 1)]
    Float(f64),

    #[regex(r"-?(?:0|[1-9]\d*)", |lex| lex.slice().parse::<i32>().unwrap(), priority = 2)]
    Integer(i32),
}

fn get_string_content(lex: &mut Lexer<Token>) -> CString {
    CString::new(&lex.slice().as_bytes()[1..lex.slice().len() - 1]).expect("invalid C string")
}

#[test]
fn test_lex_string() {
    let mut lex = Token::lexer(r#" "" "String" "#);

    assert_eq!(
        lex.next(),
        Some(Ok(Token::String(CString::new("").unwrap())))
    );
    assert_eq!(
        lex.next(),
        Some(Ok(Token::String(CString::new("String").unwrap())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_number() {
    let mut lex = Token::lexer(r#" 42 -123 1.14 1.23e-4 "#);

    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(-123))));
    assert_eq!(lex.next(), Some(Ok(Token::Float(1.14))));
    assert_eq!(lex.next(), Some(Ok(Token::Float(1.23e-4))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_hex() {
    let mut lex = Token::lexer(r#" 0xFF000000 0x00FF00 "#);

    assert_eq!(lex.next(), Some(Ok(Token::Hex(0xFF000000))));
    assert_eq!(lex.next(), Some(Ok(Token::Hex(0x00FF00))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex() {
    let mut lex = Token::lexer(r#" 100, 80, 90, 90"#);

    assert_eq!(lex.next(), Some(Ok(Token::Integer(100))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(80))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(90))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(90))));
    assert_eq!(lex.next(), None);
}
