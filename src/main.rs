use std::{
    ffi::{CString, c_char},
    path::PathBuf,
};

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
        Token::String(string_wrapper) => (Type::pointer(), Arg::new(&string_wrapper.ptr)),
        Token::Float(f) => (Type::f64(), Arg::new(f)),
        Token::Integer(i) => (Type::i32(), Arg::new(i)),
    }
}

// The StringWrapper struct is used to wrap a CString and provide a pointer to its contents.
// To avoid multiple allocations, we use a single CString and a pointer to its contents.
// Because we can't not create a `ref` to  `CString`, during steam handling
#[derive(Debug)]
struct StringWrapper {
    inner: CString,
    ptr: *const c_char,
}

impl StringWrapper {
    fn new(s: impl AsRef<[u8]>) -> Self {
        let inner = CString::new(s.as_ref()).expect("invalid C string");
        let ptr = inner.as_ptr();
        Self { inner, ptr }
    }
}

impl PartialEq for StringWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[regex(
        r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#,
        get_string_content
    )]
    String(StringWrapper),

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap(), priority = 1)]
    Float(f64),

    #[regex(r"-?(?:0|[1-9]\d*)", |lex| lex.slice().parse::<i32>().unwrap(), priority = 2)]
    Integer(i32),
}

fn get_string_content(lex: &mut Lexer<Token>) -> StringWrapper {
    StringWrapper::new(&lex.slice().as_bytes()[1..lex.slice().len() - 1])
}

#[test]
fn test_lex_string() {
    let mut lex = Token::lexer(r#" "" "String" "#);

    assert_eq!(lex.next(), Some(Ok(Token::String(StringWrapper::new("")))));
    assert_eq!(
        lex.next(),
        Some(Ok(Token::String(StringWrapper::new("String"))))
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
