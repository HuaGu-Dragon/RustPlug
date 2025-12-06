use std::{
    ffi::{CString, c_char},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
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

        let mut input = input.split_whitespace();

        let func = input.next().context("")?;
        let parsed_args = input.map(lexer).collect::<Vec<_>>();

        let args = parsed_args.iter().map(|arg| match arg {
            InputType::Interger(i) => (libffi::middle::Type::c_int(), libffi::middle::arg(i)),
            InputType::Text(_, ptr) => (libffi::middle::Type::pointer(), libffi::middle::arg(ptr)),
        });

        unsafe { manager.call_func::<()>(func, args, libffi::middle::Type::void())? };

        stdin.clear();
    }

    Ok(())
}

enum InputType {
    Interger(i32),
    Text(CString, *const i8),
}

fn lexer(input: &str) -> InputType {
    if input.starts_with('"') {
        let end = input.rfind('"').expect("missing closing quote");
        let s = CString::new(&input[1..end]).expect("invalid C string");
        println!("{s:?}");
        InputType::Text(s)
    } else {
        InputType::Integer(input.parse().expect("invalid integer"))
    }
}

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
