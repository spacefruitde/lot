//! Zahleneingabe am Zeiger (ADR 0044, 0046): eine Schreibweise für alle
//! Aktionen. Komma oder Punkt als Dezimalzeichen, Einheiten, Rechnen, Winkel,
//! relative Punkte und Reihen.
//!
//! - „3,65“, „365cm“, „3650mm“: Länge in m
//! - „2*1,25+0,115“, „(3,6-0,24)/2“: gerechnet
//! - „<30“: Winkel in Grad, gegen den Uhrzeigersinn; „3,65<30“: beides
//! - „@1,20;0,80“: relativ in x und y (Semikolon, weil das Komma die Dezimalstelle ist)
//! - „3x“: drei Stück im getippten Abstand; „3/“: drei Stück auf die Strecke verteilt

/// Was getippt wurde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry {
    Length(f64),
    Angle(f64),
    Polar { length: f64, angle: f64 },
    Offset { dx: f64, dy: f64 },
    Count(u32),
    Divide(u32),
}

/// Zeichen, die zur Zahleneingabe gehören.
pub fn accepts(c: char) -> bool {
    c.is_ascii_digit() || ",.+-*/()<@;xXcmM ".contains(c)
}

/// Ob ein Zeichen eine Zahleneingabe beginnen kann (Buchstaben sind Kürzel).
pub fn starts(c: char) -> bool {
    c.is_ascii_digit() || ",.+-(<@".contains(c)
}

/// Liest eine Eingabe; `None`, solange sie unvollständig oder falsch ist.
pub fn parse(text: &str) -> Option<Entry> {
    let t: String = text.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase();
    if t.is_empty() {
        return None;
    }
    if let Some(n) = t.strip_suffix('x') {
        return count(n).map(Entry::Count);
    }
    if let Some(n) = t.strip_suffix('/')
        && !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
    {
        return count(n).map(Entry::Divide);
    }
    if let Some(rest) = t.strip_prefix('@') {
        let (x, y) = rest.split_once(';')?;
        return Some(Entry::Offset { dx: expression(x)?, dy: expression(y)? });
    }
    if let Some((length, angle)) = t.split_once('<') {
        let angle = expression_unitless(angle)?;
        if length.is_empty() {
            return Some(Entry::Angle(angle));
        }
        return Some(Entry::Polar { length: expression(length)?, angle });
    }
    Some(Entry::Length(expression(&t)?))
}

fn count(n: &str) -> Option<u32> {
    let v: u32 = n.parse().ok()?;
    (v >= 1).then_some(v)
}

/// Ausdruck mit Einheiten, Ergebnis in m.
pub fn expression(text: &str) -> Option<f64> {
    let mut p = Parser { chars: text.chars().collect(), pos: 0, units: true };
    let v = p.sum()?;
    (p.pos == p.chars.len() && v.is_finite()).then_some(v)
}

fn expression_unitless(text: &str) -> Option<f64> {
    let mut p = Parser { chars: text.chars().collect(), pos: 0, units: false };
    let v = p.sum()?;
    (p.pos == p.chars.len() && v.is_finite()).then_some(v)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    units: bool,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn sum(&mut self) -> Option<f64> {
        let mut v = self.product()?;
        while let Some(op @ ('+' | '-')) = self.peek() {
            self.pos += 1;
            let r = self.product()?;
            v = if op == '+' { v + r } else { v - r };
        }
        Some(v)
    }

    fn product(&mut self) -> Option<f64> {
        let mut v = self.factor()?;
        while let Some(op @ ('*' | '/')) = self.peek() {
            self.pos += 1;
            // Faktoren und Teiler sind reine Zahlen: „2*1,25“ heißt zweimal 1,25 m.
            let units = std::mem::replace(&mut self.units, false);
            let r = self.factor();
            self.units = units;
            let r = r?;
            v = if op == '*' { v * r } else { v / r };
        }
        Some(v)
    }

    fn factor(&mut self) -> Option<f64> {
        match self.peek()? {
            '-' => {
                self.pos += 1;
                Some(-self.factor()?)
            }
            '+' => {
                self.pos += 1;
                self.factor()
            }
            '(' => {
                self.pos += 1;
                let v = self.sum()?;
                (self.peek() == Some(')')).then(|| self.pos += 1)?;
                Some(v)
            }
            _ => self.number(),
        }
    }

    fn number(&mut self) -> Option<f64> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == ',' || c == '.') {
            self.pos += 1;
        }
        let digits: String = self.chars[start..self.pos].iter().collect::<String>().replace(',', ".");
        let v: f64 = digits.parse().ok()?;
        if !self.units {
            return Some(v);
        }
        let rest: String = self.chars[self.pos..].iter().collect();
        for (unit, factor) in [("mm", 0.001), ("cm", 0.01), ("m", 1.0)] {
            if rest.starts_with(unit) {
                self.pos += unit.len();
                return Some(v * factor);
            }
        }
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn len(t: &str) -> f64 {
        match parse(t) {
            Some(Entry::Length(v)) => v,
            other => panic!("{t}: {other:?}"),
        }
    }

    #[test]
    fn lengths_with_units_and_arithmetic() {
        assert!((len("3,65") - 3.65).abs() < 1e-12);
        assert!((len("365cm") - 3.65).abs() < 1e-12);
        assert!((len("3650 mm") - 3.65).abs() < 1e-12);
        assert!((len("2*1,25+0,115") - 2.615).abs() < 1e-12);
        assert!((len("(3,6-0,24)/2") - 1.68).abs() < 1e-12);
        assert!((len("2,40+36,5cm") - 2.765).abs() < 1e-12);
        assert_eq!(parse("3,"), Some(Entry::Length(3.0)));
        assert_eq!(parse("2*"), None);
    }

    #[test]
    fn angles_offsets_and_series() {
        assert_eq!(parse("<30"), Some(Entry::Angle(30.0)));
        assert_eq!(parse("3,65<30"), Some(Entry::Polar { length: 3.65, angle: 30.0 }));
        assert_eq!(parse("@1,20;0,80"), Some(Entry::Offset { dx: 1.2, dy: 0.8 }));
        assert_eq!(parse("@-50cm;0"), Some(Entry::Offset { dx: -0.5, dy: 0.0 }));
        assert_eq!(parse("3x"), Some(Entry::Count(3)));
        assert_eq!(parse("4/"), Some(Entry::Divide(4)));
        assert_eq!(parse("0x"), None);
    }
}
