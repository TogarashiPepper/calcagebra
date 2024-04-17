use std::{iter::Peekable, slice::Iter};

use crate::{
    ast::{Ast, Expression},
    token::Token,
};

pub struct Parser {
    tokens: Vec<Vec<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Vec<Token>>) -> Self {
        Self { tokens }
    }

    pub fn ast(&self) -> Vec<Ast> {
        let mut ast = vec![];
        let lines = &self.tokens;

        for line in lines {
            let mut tokens = line.iter().peekable();

            let identifier = tokens.next().unwrap();

            let is_eq = **tokens.peek().unwrap() == Token::Eq;

            if is_eq {
                tokens.next();
                let mut name = "";
                if let Token::Identifier(str) = identifier {
                    name = str;
                }

                let expr = self.pratt_parser(tokens, 0).0;

                ast.push(Ast::Assignment(name.to_string(), expr));
            } else {
                let name = match identifier {
                    Token::Identifier(name) => name,
                    _ => unreachable!(),
                };

                if line.contains(&Token::Eq) {
                    let mut args = vec![];

                    loop {
                        let t = tokens.peek();

                        if t.is_none() || **t.unwrap() == Token::Eq {
                            break;
                        }

                        let t = tokens.next().unwrap();

                        args.push(match t {
                            Token::Identifier(i) => i.to_string(),
                            _ => unreachable!(),
                        })
                    }
                    tokens.next();
                    let expr = self.pratt_parser(tokens, 0).0;
                    ast.push(Ast::FunctionDeclaration(name.to_string(), args, expr));
                } else {
                    let (args, _) = self.pratt_parser(line.iter().peekable(), 0);

                    match args {
                        Expression::FunctionCall(name, args) => {
                            ast.push(Ast::FunctionCall(name.to_string(), args))
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        ast
    }

    pub fn pratt_parser<'a>(
        &'a self,
        mut tokens: Peekable<Iter<'a, Token>>,
        prec: u16,
    ) -> (Expression, Peekable<Iter<Token>>) {
        let token = tokens.next().unwrap();
        let mut expr: Option<Expression> = None;

        match token {
            Token::Identifier(i) => {
                if tokens.peek().is_some()
                    && self.infix_binding_power(tokens.peek().unwrap()) == 0
                    && **tokens.peek().unwrap() != Token::RParen
                {
                    let mut depth = 0;
                    let mut params = vec![];
                    let mut expression = vec![];

                    assert!(
                        *tokens.peek().unwrap() == &Token::LParen,
                        "expected `(` found {:?}",
                        tokens.peek().unwrap()
                    );

                    tokens.next();

                    loop {
                        let token = tokens.next();

                        if token.is_none() {
                            break;
                        }

                        let token = token.unwrap();
                        if *token == Token::RParen {
                            if depth == 0 {
                                if !expression.is_empty() && depth == 0 {
                                    let lex = expression.iter().peekable();
                                    let (data, _) = self.pratt_parser(lex, 0);

                                    params.push(data);
                                    expression.clear();
                                }
                                break;
                            }
                            depth -= 1;
                        }

                        if *token == Token::RCurly {
                            depth -= 1;
                        }

                        if *token == Token::LParen || *token == Token::LCurly {
                            depth += 1;
                        }

                        if *token == Token::Comma && depth == 0 {
                            let lex = expression.iter().peekable();
                            let (data, _) = self.pratt_parser(lex, 0);

                            params.push(data);

                            expression.clear();
                            continue;
                        }

                        expression.push(token.to_owned());
                    }
                    if !expression.is_empty() {
                        let lex = expression.iter().peekable();
                        let (data, _) = self.pratt_parser(lex, 0);

                        params.push(data);
                        expression.clear();
                    }
                    expr = Some(Expression::FunctionCall(i.to_string(), params));
                } else {
                    expr = Some(Expression::Identifier(i.to_string()));
                }
            }
            Token::LParen => {
                let exp;
                (exp, tokens) = self.pratt_parser(tokens, 0);
                expr = Some(exp);
            }
            Token::If => {
                let mut depth = 1;
                let mut params = vec![];
                let mut expression = vec![];

                loop {
                    let token = tokens.next();

                    if token.is_none() {
                        break;
                    }

                    let token = token.unwrap();

                    if *token == Token::Then || *token == Token::Else {
                        let lex = expression.iter().peekable();
                        let (data, _) = self.pratt_parser(lex, 0);

                        params.push(data);
                        expression.clear();
                        continue;
                    }

                    if *token == Token::If {
                        depth += 1;
                    }

                    if *token == Token::End {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }

                    expression.push(token.to_owned());
                }

                if !expression.is_empty() {
                    let lex = expression.iter().peekable();
                    let (data, _) = self.pratt_parser(lex, 0);

                    params.push(data);
                    expression.clear();
                }

                expr = Some(Expression::Branched(
                    Box::new(params[0].clone()),
                    Box::new(params[1].clone()),
                    Box::new(params[2].clone()),
                ))
            }
            Token::LCurly => {
                let mut depth = 0;
                let mut params = vec![];
                let mut expression = vec![];

                loop {
                    let token = tokens.next();

                    if token.is_none() {
                        break;
                    }

                    let token = token.unwrap();
                    if *token == Token::RCurly {
                        if depth == 0 {
                            if !expression.is_empty() && depth == 0 {
                                let lex = expression.iter().peekable();
                                let (data, _) = self.pratt_parser(lex, 0);

                                params.push(data);
                                expression.clear();
                            }
                            break;
                        }
                        depth -= 1;
                    }

                    if *token == Token::LCurly {
                        depth += 1;
                    }

                    if *token == Token::Comma {
                        let lex = expression.iter().peekable();
                        let (data, _) = self.pratt_parser(lex, 0);

                        params.push(data);

                        expression.clear();
                        continue;
                    }

                    expression.push(token.to_owned());
                }

                if !expression.is_empty() {
                    let lex = expression.iter().peekable();
                    let (data, _) = self.pratt_parser(lex, 0);

                    params.push(data);
                    expression.clear();
                }

                expr = Some(Expression::SizedSet(params))
            }
            Token::Sub => {
                if let Token::Number(i) = tokens.peek().unwrap() {
                    expr = Some(Expression::Number(-i));
                    tokens.next();
                }
            }
            _ => {
                if let Token::Number(i) = token {
                    expr = Some(Expression::Number(*i));
                }
            }
        };

        loop {
            let op = tokens.peek();

            if op.is_none() || **op.unwrap() == Token::RParen {
                tokens.next();
                break;
            }

            let op = tokens.next().unwrap();

            if *op == Token::Pow && self.infix_binding_power(op) < prec {
                break;
            }

            if *op != Token::Pow && self.infix_binding_power(op) <= prec {
                break;
            }

            let rhs;
            (rhs, tokens) = self.pratt_parser(tokens, self.infix_binding_power(op));
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.clone(),
                Box::new(rhs),
            ))
        }

        (expr.unwrap(), tokens)
    }

    fn infix_binding_power(&self, op: &Token) -> u16 {
        match op {
            Token::Add => 1,
            Token::Sub => 2,
            Token::Mul => 3,
            Token::Div => 4,
            Token::Modulo => 5,
            Token::Pow => 6,
            Token::IsEq | Token::Gt | Token::Lt | Token::GtEq | Token::LtEq => 7,
            Token::If | Token::Then | Token::Else | Token::End => 8,
            _ => 0,
        }
    }
}
