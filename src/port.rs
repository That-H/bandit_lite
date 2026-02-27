use super::*;
use std::str::FromStr;

/// A single input or input direction for an entity.
#[derive(Clone, Debug)]
pub struct Port {
    /// Output of the port. Cannot output anything if the input is anything other than nothing.
    pub out: Expr,
}

impl Port {
    /// Create a port with the given expression.
    pub fn new(out: Expr) -> Self {
        Self {
            out,
        }
    }
}

pub type Clrs = [beam::Clr; 8];

impl FromIterator<beam::Clr> for Clrs {
    fn from_iter<T: IntoIterator<Item = beam::Clr>>(iter: T) -> Self {
        let mut t = iter.into_iter();
        std::array::from_fn(|_| t.next().unwrap())
    }
}

/// A group of ports for an entity.
#[derive(Clone, Debug)]
pub struct PortGrp([Port; 8]);

impl PortGrp {
    /// Return the output colours of this port group.
    pub fn determine(&self, inpts: &Clrs) -> Clrs {
        let mut outs = Vec::new();

        for (p, &i) in self.0.iter().zip(inpts) {
            let out = if i != beam::Clr::Black {
                beam::Clr::Black
            } else {
                beam::Clr::from(p.out.eval(inpts))
            };

            outs.push(out);
        }

        Clrs::from_iter(outs)
    }
}

impl FromIterator<Expr> for PortGrp {
    fn from_iter<T: IntoIterator<Item = Expr>>(iter: T) -> Self {
        let mut t = iter.into_iter();
        Self(std::array::from_fn(|_| Port::new(t.next().unwrap())))
    }
}

/// A value for a beam colour.
#[derive(Clone, Debug, Default)]
pub enum Expr {
    /// The input colour in the port with the given number.
    Port(usize),
    /// A literal colour.
    Clr(u8),
    /// An operation between two expressions.
    Op(Op, Box<Self>, Box<Self>),
    /// An operation on a single expression.
    UnaryOp(UnaryOp, Box<Self>),
    /// No value.
    #[default]
    Null,
}

impl Expr {
    /// Evaluate this expression, given the ports of the object.
    pub fn eval(&self, inpts: &Clrs) -> u8 {
        match self {
            Self::Port(idx) => inpts[*idx] as u8,
            Self::Clr(val) => *val,
            Self::Op(op, e1, e2) => op.eval(e1.eval(inpts), e2.eval(inpts)),
            Self::UnaryOp(op, e1) => op.eval(e1.eval(inpts)),
            Self::Null => 0,
        }
    }

    /// Parses a string into an expression. Uses this internally for parsing
    /// because it's a lot easier than using strs.
    fn char_parse(chars: &[char]) -> Result<Self, <Self as FromStr>::Err> {
        let mut expr = Self::Null;
        let mut idx = 0;

        while let Some(&ch) = chars.get(idx) {
            if let Ok(unop) = UnaryOp::try_from(ch) {
                expr.chuck(Expr::UnaryOp(unop, Box::new(Expr::Null)));
            } else if let Ok(op) = Op::try_from(ch) {
                expr = Expr::Op(op, Box::new(expr), Box::new(Expr::Null));
            } else if let Ok(clr) = beam::Clr::try_from(ch) {
                expr.chuck(Expr::Clr(clr as u8));
            } else if let Some(port) = ch.to_digit(8) {
                expr.chuck(Expr::Port(port as usize));
            } else {
                match ch {
                    '(' => {
                        let mut opened = 1;
                        let mut in_idx = idx + 1;

                        while let Some(&ch) = chars.get(in_idx) {
                            match ch {
                                '(' => opened += 1,
                                ')' => {
                                    opened -= 1;
                                    // Found matching bracket.
                                    if opened == 0 {
                                        expr.chuck(Self::char_parse(&chars[idx+1..in_idx])?);
                                        idx = in_idx;
                                        break;
                                    }
                                }
                                _ => (),
                            }
                            in_idx += 1;
                        }
                        if opened != 0 {
                            return Err(ExprParseErr::UnclosedBracket);
                        }
                    },
                    _ => return Err(ExprParseErr::InvalidToken(ch)),
                }
            }
            idx += 1;
        }

        Ok(expr)
    }

    /// Chuck the provided expression into self.
    fn chuck(&mut self, other: Expr) {
        match self {
            Self::Null => *self = other,
            Self::Op(_op, _e1, e2) => **e2 = other,
            Self::UnaryOp(_op, e1) => **e1 = other,
            _ => (),
        }
    }
}

impl TryFrom<char> for Expr {
    type Error = ExprParseErr;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            a if a.is_ascii_digit() => Self::Port(a.to_digit(8).unwrap() as usize),
            ' ' => Self::Null,
            'n' => Self::Clr(beam::Clr::Black as u8),
            'r' => Self::Clr(beam::Clr::Red as u8),
            'g' => Self::Clr(beam::Clr::Green as u8),
            'b' => Self::Clr(beam::Clr::Blue as u8),
            'y' => Self::Clr(beam::Clr::Yellow as u8),
            'm' => Self::Clr(beam::Clr::Magenta as u8),
            'c' => Self::Clr(beam::Clr::Cyan as u8),
            'w' => Self::Clr(beam::Clr::White as u8),
            _ => return Err(ExprParseErr::InvalidLiteral),
        })
    }
}

impl FromStr for Expr {
    type Err = ExprParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::char_parse(&s.chars().collect::<Vec<_>>())
    }
}

/// An error that occurs when parsing an expression.
#[derive(Clone, Debug)]
pub enum ExprParseErr {
    InvalidLiteral,
    InvalidOp,
    InvalidToken(char),
    UnclosedBracket,
}

/// An operation that can be performed on two expressions.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    /// Bitwise or.
    Or,
    /// Bitwise and.
    And,
    /// Bitwise xor.
    Xor,
}

impl Op {
    /// Performs this operation on the two inputs. The rhs is ignored if it is not needed.
    pub fn eval(&self, lhs: u8, rhs: u8) -> u8 {
        match *self {
            Self::Or => lhs | rhs,
            Self::And => lhs & rhs,
            Self::Xor => lhs ^ rhs,
        }
    }
}

impl TryFrom<char> for Op {
    type Error = ExprParseErr;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '|' => Self::Or,
            '&' => Self::And,
            '^' => Self::Xor,
            _ => return Err(ExprParseErr::InvalidOp),
        })
    }
}

/// An operation that can be performed on two expressions.
#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    /// Bitwise not.
    Not,
    /// Maps primary colours to themselves and everything else to 0.
    Primary,
}

impl UnaryOp {
    /// Performs this operation on the two inputs. The rhs is ignored if it is not needed.
    pub fn eval(&self, lhs: u8) -> u8 {
        match *self {
            Self::Not => 0b111 ^ lhs,
            Self::Primary => match lhs {
                0b100 | 0b010 | 0b001 => lhs,
                _ => 0,
            },
        }
    }
}

impl TryFrom<char> for UnaryOp {
    type Error = ExprParseErr;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '!' => Self::Not,
            'P' => Self::Primary,
            _ => return Err(ExprParseErr::InvalidOp),
        })
    }
}
