use std::io::{self, Write};

mod parser;
pub use crate::parser::*;
use crate::parser::common::ParseError;

fn main() {
        println!("blis {}", env!("CARGO_PKG_VERSION"));
        println!("Simple arithmetic calculator. Just type expression and hit Enter. Type \"quit\" to exit.");
        println!("Example: (2+2)*5 - (-3+2.1)/(25*3.1415) + 0.0001");

        let mut parser = arithmetic_parser::Parser {
                numbers: vec![],
                operations: vec![]
        };

        loop {
                print!("\n>> ");
                let _ = io::stdout().flush();

                let mut expression = String::new();
                io::stdin().read_line(&mut expression).expect("Input failure.");
                // TODO: Input check

                if expression.trim() == "quit" {
                        println!("...bye");
                        break;
                }

                let _ = match parser.calculate(&expression) {
                        Ok(v) => println!("{}", v),
                        Err(ref err) => match err {
                                ParseError::BadExpression => println!("Bad expression"),
                                ParseError::InvalidOperation => println!("Invalid operation"),
                                ParseError::OperationBalance => println!("Parse error"),
                                ParseError::PopFailure => println!("Parse error"),
                        },
                };
        }
}
