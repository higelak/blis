//! Parser module
//! Contains token stuff, common utility stuff and arithmetic parser submodule
//! Author: Pavel N
//! Date: 05.01.2021

/// Token module for representing simple element of math expression
pub mod token {

    #[derive(Copy, Clone)]
    #[repr(u8)]
    pub enum TokenType {
        Empty = 0,
        Number,
        Operation,
        Function,
        Unknown,
    }

    impl PartialEq for TokenType {
        fn eq(&self, other: &TokenType) -> bool {
            match (self, other) {
                (&TokenType::Empty, &TokenType::Empty) |
                (&TokenType::Number, &TokenType::Number) |
                (&TokenType::Operation, &TokenType::Operation) |
                (&TokenType::Function, &TokenType::Function) |
                (&TokenType::Unknown, &TokenType::Unknown) => true,
                (x, y) => *x as u8 == *y as u8,
            }
        }
    }

    #[derive(Clone)]
    pub struct Token {
        pub value: String,
        pub token_type: TokenType,
    }

    impl Token {
        pub fn set(&mut self, value: String, token_type: TokenType) {
            self.value = value;
            self.token_type = token_type;
        }

        pub fn get_type(&self) -> TokenType {
            self.token_type.clone()
        }
        pub fn get_value(&self) -> String {
            self.value.clone()
        }

        pub fn is_empty(self) -> bool {
            self.token_type == TokenType::Empty
        }
        pub fn is_number(&self) -> bool {
            self.token_type == TokenType::Number
        }
        pub fn is_operation(&self) -> bool {
            self.token_type == TokenType::Operation
        }
        pub fn is_function(&self) -> bool {
            self.token_type == TokenType::Function
        }
    }
}

/// Common module contains utility functions for all possible parsers
pub mod common {

    use super::token::*;

    #[derive(PartialEq, Eq)]
    pub enum ParseError {
        InvalidOperation,
        OperationBalance,
        PopFailure,
        BadExpression,
    }

    /// Remove whitespaces from string
    pub fn remove_whitespace(str: &mut String) {
        str.retain(|c| !c.is_whitespace());
    }

    /// Return operation string with only one character for a while... (Because we work with one character operations "+", "-", "*", "/")
    // TODO: expand to the required number of operations, including multi-character
    fn get_operation(expression: &String, pos: &mut i32) -> String {
        let op: String = String::from(expression.chars().nth(*pos as usize).unwrap());
        *pos += 1;
        op
    }

    /// Return a string containing a number.
    /// Work with numbers, including fractional ones. Delimiter "."
    fn get_number(expression: &String, pos: &mut i32) -> String {
        let mut num: String = String::new();
        while *pos < (expression.len() as i32) && (expression.chars().nth(*pos as usize).unwrap().is_digit(10) || expression.chars().nth(*pos as usize).unwrap() == '.') {
            num.push(expression.chars().nth(*pos as usize).unwrap());
            *pos += 1;
        }
        num
    }

    /// Return token at specified position
    pub fn get_token(expression: &String, pos: &mut i32) -> Token {
        let mut result = Token {
            value: String::new(),
            token_type: TokenType::Empty,
        };

        // Reached the end, returning an empty token
        if *pos == (expression.len() as i32) {
            result
        }

        // Let's check the character for a number
        else if expression.chars().nth(*pos as usize).unwrap().is_digit(10) {
            result.set(get_number(&expression, pos), TokenType::Number);
            result
        }
        // In other cases we have an operation (or incorrect input)
        else {
            result.set(get_operation(&expression, pos), TokenType::Operation);
            result
        }
    }

    /// Get the priority of an operation
    pub fn get_priority(operation: &String) -> Result<i8, ParseError> {
        if operation == "(" {
            // The open parenthesis is a special case and is also considered an operation. It does not pop anyone off the stack of operations, but it also does not allow itself to be popped off the stack.
            // Only a closing parenthesis can pop it. Accordingly, when the token ")" is received during parsing, it will pop all operations up to the first opening parenthesis.
            Ok(-1)
        } else if operation == "*" || operation == "/" {
            // Lowest priority for multiplication and division
            Ok(1)
        } else if operation == "+" || operation == "-" {
            // Highest priority for addition and subtraction. The highest priority operation pops the lowest priority operation from the stack.
            // At the same time, we pop the last two digits from the stack of numbers, perform the popped operation on them. The result is pushed onto the stack with numbers.
            Ok(2)
        } else {
            Err(ParseError::InvalidOperation)
        }
    }
}

/// This module contains arithmetic parser. Calculates the result of an arithmetic expressions
pub mod arithmetic_parser {

    use super::token::*;
    use super::common::*;

    trait Stack<T> {
        fn top(&mut self) -> Option<&T>;
    }

    // Since we use a vector as a stack, we implement a trait for it. We need to add the "top" function to the vector type
    impl<T> Stack<T> for Vec<T> {
        fn top(&mut self) -> Option<&T> {
            match self.len() {
                0 => None,
                n => Some(&self[n - 1])
            }
        }
    }

    pub struct Parser {
        pub numbers: std::vec::Vec<f64>,
        pub operations: std::vec::Vec<String>,
    }

    impl Parser {

        /// Return a result of calculation of specified arithmetic expression
        pub fn calculate(&mut self, origin_expression: &String) -> Result<f64, ParseError> {
            let mut expression = format!("({})", origin_expression);
            remove_whitespace(&mut expression);

            let mut token;
            let mut prevtoken = Token {
                value: String::from("X"),
                token_type: TokenType::Operation,
            };

            let mut pos: i32 = 0;

            self.numbers.clear();
            self.operations.clear();

            loop {
                // Get token
                token = get_token(&expression, &mut pos);

                // Process the unary + and -
                if token.is_operation() && ((token.get_value() == "+") || (token.get_value() == "-")) &&
                    prevtoken.is_operation() && (prevtoken.get_value() == "(") {
                    // Substitute 0. Thus, for example, the expression 4+(-1)*(2+2) becomes 4+(0-1)*(2 + 2)
                    self.numbers.push(0.0);
                }

                // If token is number then push it to stack of numbers
                if token.is_number() {
                    // convert the string to double
                    let number: f64 = token.get_value().parse().unwrap();
                    self.numbers.push(number);
                }

                // If token is operation
                if token.is_operation() {
                    let op = token.get_value();
                    // then checking for a closing parenthesis
                    if op == ")" {
                        // if it's a closing parenthesis, then pop up to the first opening parenthesis inclusive
                        while !self.operations.is_empty() && self.operations.top().unwrap() != "(" {
                            let result = self.pop_operation();
                            let _ = match result {
                                Ok(content) => { content },
                                Err(error) => { return Err(error.into()); }
                            };
                        }

                        // Open parenthesis is popped here
                        self.operations.pop();
                    } else {
                        // If we can pop the operation, then do it
                        if self.can_pop_operation(&op) {
                            let result = self.pop_operation();
                            let _ = match result {
                                Ok(content) => { content },
                                Err(error) => { return Err(error.into()); }
                            };
                        }

                        // Push new operation to stack of operations
                        self.operations.push(op);
                    }
                }

                prevtoken = token.clone();

                if token.is_empty() {
                    break;
                }
            }

            if self.numbers.len() > 1 || self.operations.len() > 0 {
                return Err(ParseError::BadExpression)
            }

            // One number should remain at the top of the stack of numbers. This will be the result of calculations
            match self.numbers.top() {
                Some(&val) => Ok(val),
                None => Err(ParseError::BadExpression),
            }
        }

        fn can_pop_operation(&mut self, operation: &String) -> bool {
            if self.operations.is_empty() {
                false
            } else {

                // Input operation priority
                let prior1 = get_priority(operation);

                // Priority of the operation at the top of the operation stack
                match self.operations.top() {
                    Some(val) => {
                        let prior2 = get_priority(val);
                        match prior1 {
                            Ok(v) => {
                                let p1 = v;
                                match prior2 {
                                    Ok(v) => {
                                        let p2 = v;
                                        // We remember about the opening parenthesis (its priority is -1), it is non-popable (it will be popped out only by the closing parenthesis), so let's check the priorities for >= 0
                                        // If the priorities of the operations are equal, then we can pop. If the priority of the input operation is higher, then we can pop.
                                        // In other cases, we cannot pop
                                        p1 >= 0 && p2 >= 0 && p1 >= p2
                                    },
                                    Err(_) => false,
                                }
                            },
                            Err(_) => false,
                        }
                    },
                    None => false,
                }
            }
        }

        fn pop_number(&mut self) -> Result<f64, ParseError> {
            let x: f64;
            if !self.numbers.is_empty() {
                match self.numbers.top() {
                    Some(&val) => {
                        x = val;
                        self.numbers.pop();
                        Ok(x)
                    },
                    None => Err(ParseError::PopFailure),
                }
            } else {
                Err(ParseError::OperationBalance)
            }
        }

        fn pop_operation(&mut self) -> Result<(), ParseError> {

            // Pop the first number from the stack of numbers
            let a = match self.pop_number() {
                Ok(v) => v,
                Err(_) => return Err(ParseError::OperationBalance),
            };

            // Pop the second number from the stack of numbers
            let b = match self.pop_number() {
                Ok(v) => v,
                Err(_) => return Err(ParseError::OperationBalance),
            };

            // Pop the operation
            let operation = match self.operations.top() {
                Some(val) => val.clone(),
                None => return Err(ParseError::PopFailure),
            };
            self.operations.pop();

            // Calculate and push the result to stack of numbers
            // We take into account that data is popped from the stack in reverse order, so the first is "b", and then "a"
            if operation == "+" {
                self.numbers.push(b + a);
            } else if operation == "-" {
                self.numbers.push(b - a);
            } else if operation == "*" {
                self.numbers.push(b * a);
            } else if operation == "/" {
                self.numbers.push(b / a);
            }

            Ok(())
        }
    }

    // TODO: Create all the necessary tests
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sum() {
            let mut parser = Parser {
                numbers: vec![],
                operations: vec![]
            };

            let exp = String::from("2+2");
            let expected: f64 = 4.0;

            let _ = match parser.calculate(&exp) {
                Ok(value) => assert_eq!(value, expected),
                Err(_) => assert!(false),
            };
        }
    }
}
