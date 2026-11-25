//! An inline calculator for the palette: `2+2*3`, `sqrt(16)`, `200 + 10%`, `2pi`, `5!`.
//! A small recursive-descent parser over `f64`. No `eval`, no dependencies.

#[derive(Clone, Debug, PartialEq)]
pub enum CalcError {
    Empty,
    Syntax,
    DivideByZero,
    Domain,
    UnknownName(String),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    Bang,
    LParen,
    RParen,
    Comma,
}

fn lex(src: &str) -> Result<Vec<Tok>, CalcError> {
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < c.len() {
        let ch = c[i];
        match ch {
            ' ' | '\t' | '\u{a0}' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
                    i += 1;
                }
                // 1e3, 2.5e-4 (an `e` followed by digits is an exponent, otherwise Euler's number)
                if i < c.len() && (c[i] == 'e' || c[i] == 'E') {
                    let mut j = i + 1;
                    if j < c.len() && (c[j] == '+' || c[j] == '-') {
                        j += 1;
                    }
                    if j < c.len() && c[j].is_ascii_digit() {
                        while j < c.len() && c[j].is_ascii_digit() {
                            j += 1;
                        }
                        i = j;
                    }
                }
                let text: String = c[start..i].iter().collect();
                out.push(Tok::Num(text.parse().map_err(|_| CalcError::Syntax)?));
            }
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' | '\u{2212}' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' | '\u{d7}' => {
                if ch == '*' && c.get(i + 1) == Some(&'*') {
                    out.push(Tok::Caret);
                    i += 2;
                } else {
                    out.push(Tok::Star);
                    i += 1;
                }
            }
            '/' | '\u{f7}' => {
                out.push(Tok::Slash);
                i += 1;
            }
            '^' => {
                out.push(Tok::Caret);
                i += 1;
            }
            '%' => {
                out.push(Tok::Percent);
                i += 1;
            }
            '!' => {
                out.push(Tok::Bang);
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '\u{3c0}' => {
                out.push(Tok::Name("pi".into()));
                i += 1;
            }
            '\u{221a}' => {
                out.push(Tok::Name("sqrt".into()));
                i += 1;
            }
            _ if ch.is_alphabetic() => {
                let start = i;
                while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                    i += 1;
                }
                out.push(Tok::Name(
                    c[start..i].iter().collect::<String>().to_lowercase(),
                ));
            }
            _ => return Err(CalcError::Syntax),
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
enum Node {
    Num(f64),
    Neg(Box<Node>),
    Bin(char, Box<Node>, Box<Node>),
    Pct(Box<Node>),
    Fact(Box<Node>),
    Call(String, Vec<Node>),
}

struct Parser {
    t: Vec<Tok>,
    p: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.p)
    }
    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == Some(tok) {
            self.p += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<Node, CalcError> {
        let mut left = self.term()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => '+',
                Some(Tok::Minus) => '-',
                _ => return Ok(left),
            };
            self.p += 1;
            let right = self.term()?;
            left = Node::Bin(op, Box::new(left), Box::new(right));
        }
    }
    fn starts_primary(&self) -> bool {
        matches!(self.peek(), Some(Tok::Num(_) | Tok::Name(_) | Tok::LParen))
            && !matches!(self.peek(), Some(Tok::Name(n)) if n == "mod")
    }
    fn term(&mut self) -> Result<Node, CalcError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => '*',
                Some(Tok::Slash) => '/',
                Some(Tok::Name(n)) if n == "mod" => 'm',
                _ if self.starts_primary() => '*', // 2pi, 3(4+1)
                _ => return Ok(left),
            };
            if op != '*' || matches!(self.peek(), Some(Tok::Star)) {
                self.p += 1;
            }
            let right = self.unary()?;
            left = Node::Bin(op, Box::new(left), Box::new(right));
        }
    }
    fn unary(&mut self) -> Result<Node, CalcError> {
        if self.eat(&Tok::Minus) {
            return Ok(Node::Neg(Box::new(self.unary()?)));
        }
        if self.eat(&Tok::Plus) {
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> Result<Node, CalcError> {
        let base = self.postfix()?;
        if self.eat(&Tok::Caret) {
            let exp = self.unary()?; // right associative, allows 2^-1
            return Ok(Node::Bin('^', Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }
    fn postfix(&mut self) -> Result<Node, CalcError> {
        let mut node = self.primary()?;
        loop {
            if self.eat(&Tok::Percent) {
                node = Node::Pct(Box::new(node));
            } else if self.eat(&Tok::Bang) {
                node = Node::Fact(Box::new(node));
            } else {
                return Ok(node);
            }
        }
    }
    fn primary(&mut self) -> Result<Node, CalcError> {
        match self.t.get(self.p).cloned() {
            Some(Tok::Num(n)) => {
                self.p += 1;
                Ok(Node::Num(n))
            }
            Some(Tok::LParen) => {
                self.p += 1;
                let inner = self.expr()?;
                if !self.eat(&Tok::RParen) {
                    return Err(CalcError::Syntax);
                }
                Ok(inner)
            }
            Some(Tok::Name(name)) => {
                self.p += 1;
                if self.eat(&Tok::LParen) {
                    let mut args = Vec::new();
                    if !self.eat(&Tok::RParen) {
                        loop {
                            args.push(self.expr()?);
                            if self.eat(&Tok::Comma) {
                                continue;
                            }
                            if self.eat(&Tok::RParen) {
                                break;
                            }
                            return Err(CalcError::Syntax);
                        }
                    }
                    Ok(Node::Call(name, args))
                } else {
                    match name.as_str() {
                        "pi" => Ok(Node::Num(std::f64::consts::PI)),
                        "tau" => Ok(Node::Num(std::f64::consts::TAU)),
                        "e" => Ok(Node::Num(std::f64::consts::E)),
                        // `sqrt 16`: a function name with no bracket takes the next unary
                        _ if is_function(&name) => {
                            let arg = self.unary()?;
                            Ok(Node::Call(name, vec![arg]))
                        }
                        _ => Err(CalcError::UnknownName(name)),
                    }
                }
            }
            _ => Err(CalcError::Syntax),
        }
    }
}

fn is_function(name: &str) -> bool {
    matches!(
        name,
        "sqrt"
            | "cbrt"
            | "abs"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "ln"
            | "log"
            | "log2"
            | "exp"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
            | "pow"
    )
}

fn eval(node: &Node) -> Result<f64, CalcError> {
    Ok(match node {
        Node::Num(n) => *n,
        Node::Neg(n) => -eval(n)?,
        Node::Pct(n) => eval(n)? / 100.0,
        Node::Fact(n) => {
            let v = eval(n)?;
            if v < 0.0 || v.fract() != 0.0 || v > 170.0 {
                return Err(CalcError::Domain);
            }
            (1..=(v as u64)).fold(1.0, |a, k| a * k as f64)
        }
        // `200 + 10%` and `200 - 10%` mean "ten percent of the left side"
        Node::Bin(op @ ('+' | '-'), l, r) if matches!(**r, Node::Pct(_)) => {
            let left = eval(l)?;
            let Node::Pct(inner) = &**r else {
                return Err(CalcError::Syntax);
            };
            let pct = eval(inner)? / 100.0;
            if *op == '+' {
                left * (1.0 + pct)
            } else {
                left * (1.0 - pct)
            }
        }
        Node::Bin(op, l, r) => {
            let (a, b) = (eval(l)?, eval(r)?);
            match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' => {
                    if b == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    a / b
                }
                'm' => {
                    if b == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    a.rem_euclid(b)
                }
                _ => a.powf(b),
            }
        }
        Node::Call(name, args) => {
            let v: Vec<f64> = args.iter().map(eval).collect::<Result<_, _>>()?;
            let one = |f: fn(f64) -> f64| -> Result<f64, CalcError> {
                if v.len() == 1 {
                    Ok(f(v[0]))
                } else {
                    Err(CalcError::Syntax)
                }
            };
            match name.as_str() {
                "sqrt" => {
                    if v.first().is_some_and(|x| *x < 0.0) {
                        return Err(CalcError::Domain);
                    }
                    one(f64::sqrt)?
                }
                "cbrt" => one(f64::cbrt)?,
                "abs" => one(f64::abs)?,
                "sin" => one(f64::sin)?,
                "cos" => one(f64::cos)?,
                "tan" => one(f64::tan)?,
                "asin" => one(f64::asin)?,
                "acos" => one(f64::acos)?,
                "atan" => one(f64::atan)?,
                "ln" | "log" | "log2" => {
                    if v.first().is_some_and(|x| *x <= 0.0) {
                        return Err(CalcError::Domain);
                    }
                    match name.as_str() {
                        "ln" => one(f64::ln)?,
                        "log" => one(f64::log10)?,
                        _ => one(f64::log2)?,
                    }
                }
                "exp" => one(f64::exp)?,
                "floor" => one(f64::floor)?,
                "ceil" => one(f64::ceil)?,
                "round" => one(f64::round)?,
                "min" if !v.is_empty() => v.iter().copied().fold(f64::INFINITY, f64::min),
                "max" if !v.is_empty() => v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                "pow" if v.len() == 2 => v[0].powf(v[1]),
                _ => return Err(CalcError::UnknownName(name.clone())),
            }
        }
    })
}

/// Evaluates an expression. Errors for anything that is not clean arithmetic.
pub fn evaluate(src: &str) -> Result<f64, CalcError> {
    let toks = lex(src)?;
    if toks.is_empty() {
        return Err(CalcError::Empty);
    }
    let mut p = Parser { t: toks, p: 0 };
    let node = p.expr()?;
    if p.p != p.t.len() {
        return Err(CalcError::Syntax);
    }
    let v = eval(&node)?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(CalcError::Domain)
    }
}

/// Whether a palette query is a calculation worth answering: it evaluates, contains an operator or
/// a function, and is not a bare number or constant (so `7`, `pi` and `chrome` stay searches).
#[must_use]
pub fn is_calculation(src: &str) -> bool {
    let s = src.trim();
    if !s.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    let operator = s
        .chars()
        .skip(1)
        .any(|c| "+-*/^%!\u{d7}\u{f7}(".contains(c))
        || s.chars()
            .next()
            .is_some_and(|c| c == '(' || c == '\u{221a}')
        || s.to_lowercase()
            .split(|c: char| !c.is_alphabetic())
            .any(|w| is_function(w) || w == "mod");
    operator && evaluate(s).is_ok()
}

fn round_significant(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    format!("{x:.11e}").parse().unwrap_or(x)
}

/// A result as plain text: integers without a decimal point, no float noise (`0.1+0.2` is `0.3`).
#[must_use]
pub fn format_result(x: f64) -> String {
    let r = round_significant(x);
    if r.fract() == 0.0 && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else if r.abs() >= 1e15 || r.abs() < 1e-6 {
        format!("{r:e}")
    } else {
        format!("{r}")
    }
}

/// `1234567.891` to `1,234,567.891`, for display (the clipboard gets the plain text).
#[must_use]
pub fn group_thousands(plain: &str) -> String {
    if plain.contains('e') {
        return plain.to_owned();
    }
    let (sign, rest) = plain.strip_prefix('-').map_or(("", plain), |r| ("-", r));
    let (int, frac) = rest
        .split_once('.')
        .map_or((rest, None), |(i, f)| (i, Some(f)));
    let mut grouped = String::new();
    for (n, ch) in int.chars().enumerate() {
        if n > 0 && (int.len() - n) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    match frac {
        Some(f) => format!("{sign}{grouped}.{f}"),
        None => format!("{sign}{grouped}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(s: &str) -> String {
        evaluate(s).map_or_else(|e| format!("{e:?}"), format_result)
    }

    #[test]
    fn arithmetic_and_precedence() {
        assert_eq!(ev("2+2*3"), "8");
        assert_eq!(ev("(2+2)*3"), "12");
        assert_eq!(ev("10/4"), "2.5");
        assert_eq!(ev("2^3^2"), "512");
        assert_eq!(ev("2^-1"), "0.5");
        assert_eq!(ev("-3^2"), "-9");
        assert_eq!(ev("7 mod 3"), "1");
        assert_eq!(ev("-7 mod 3"), "2");
        assert_eq!(ev("2 ** 10"), "1024");
        assert_eq!(ev("6 \u{d7} 7 \u{f7} 2"), "21");
    }

    #[test]
    fn float_noise_is_hidden() {
        assert_eq!(ev("0.1+0.2"), "0.3");
        assert_eq!(ev("1/3"), "0.333333333333");
        assert_eq!(ev("1e3+1"), "1001");
        assert_eq!(ev("2.5e-4*2"), "0.0005");
    }

    #[test]
    fn percent_and_factorial() {
        assert_eq!(ev("50%"), "0.5");
        assert_eq!(ev("200+10%"), "220");
        assert_eq!(ev("200-10%"), "180");
        assert_eq!(ev("50%*200"), "100");
        assert_eq!(ev("5!"), "120");
        assert_eq!(ev("0!"), "1");
        assert_eq!(ev("2.5!"), "Domain");
    }

    #[test]
    fn functions_constants_and_implicit_multiplication() {
        assert_eq!(ev("sqrt(16)"), "4");
        assert_eq!(ev("sqrt 81"), "9");
        assert_eq!(ev("max(3, 9, 4)"), "9");
        assert_eq!(ev("pow(2,8)"), "256");
        assert_eq!(ev("round(2.5)"), "3");
        assert_eq!(ev("2(3+4)"), "14");
        assert_eq!(ev("log(1000)"), "3");
        assert_eq!(ev("sin(0)+cos(0)"), "1");
        assert_eq!(ev("\u{3c0}"), "3.14159265359");
        assert_eq!(ev("2pi"), "6.28318530718");
        assert_eq!(ev("e^1"), "2.71828182846");
    }

    #[test]
    fn errors() {
        assert_eq!(ev(""), "Empty");
        assert_eq!(ev("1/0"), "DivideByZero");
        assert_eq!(ev("sqrt(-1)"), "Domain");
        assert_eq!(ev("ln(0)"), "Domain");
        assert_eq!(ev("2+"), "Syntax");
        assert_eq!(ev("(2+3"), "Syntax");
        assert_eq!(ev("2 3 )"), "Syntax");
        assert_eq!(ev("foo+1"), "UnknownName(\"foo\")");
        assert_eq!(ev("2 $ 3"), "Syntax");
        assert_eq!(ev("99999^99999"), "Domain");
    }

    #[test]
    fn which_queries_are_calculations() {
        for yes in [
            "2+2", "2+2*3", "sqrt(16)", "(1+2)*3", "200+10%", "5!", "2^8", "7 mod 3", "10/4",
        ] {
            assert!(is_calculation(yes), "{yes}");
        }
        for no in [
            "7",
            "pi",
            "chrome",
            "",
            "7-zip",
            "visual studio 2022",
            "3d",
            "e",
            "windows 11",
        ] {
            assert!(!is_calculation(no), "{no}");
        }
    }

    #[test]
    fn grouping() {
        assert_eq!(group_thousands("1234567.891"), "1,234,567.891");
        assert_eq!(group_thousands("-1000"), "-1,000");
        assert_eq!(group_thousands("999"), "999");
        assert_eq!(group_thousands("1e20"), "1e20");
    }
}
