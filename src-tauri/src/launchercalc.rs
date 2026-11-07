//! Launcher (fase L3): calcolatrice del campo di ricerca. Parser a discesa ricorsiva, puro e
//! testato, senza `eval` né esecuzione di nulla: legge solo numeri, operatori e poche funzioni.
//!
//! Sintassi: `+ - * / ^`, parentesi, `x`/`×`/`÷`, decimali con virgola o punto, `sqrt()`,
//! `abs()`, `pi`, e percentuali alla maniera di Spotlight: `20% di 340` (o `of`) = 68,
//! `340 + 10%` = 374, `340 - 10%` = 306. Un numero da solo o una percentuale da sola non è un
//! calcolo (nessun risultato): serve almeno un operatore o una funzione, così "7-zip" o "42"
//! non producono rumore.

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
    Percent,
}

fn lex(input: &str) -> Option<Vec<Tok>> {
    let cs: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '0'..='9' | '.' | ',' => {
                let start = i;
                let mut seps = 0;
                while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.' || cs[i] == ',') {
                    if cs[i] == '.' || cs[i] == ',' {
                        seps += 1;
                    }
                    i += 1;
                }
                if seps > 1 {
                    return None;
                }
                let text: String = cs[start..i].iter().map(|&c| if c == ',' { '.' } else { c }).collect();
                out.push(Tok::Num(text.parse().ok()?));
            }
            c if c.is_alphabetic() => {
                let start = i;
                while i < cs.len() && cs[i].is_alphabetic() {
                    i += 1;
                }
                out.push(Tok::Ident(cs[start..i].iter().collect::<String>().to_lowercase()));
            }
            '+' => {
                out.push(Tok::Op('+'));
                i += 1;
            }
            '-' | '−' | '–' => {
                out.push(Tok::Op('-'));
                i += 1;
            }
            '*' | '×' | '·' => {
                out.push(Tok::Op('*'));
                i += 1;
            }
            '/' | '÷' | ':' => {
                out.push(Tok::Op('/'));
                i += 1;
            }
            '^' => {
                out.push(Tok::Op('^'));
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
            '%' => {
                out.push(Tok::Percent);
                i += 1;
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Un valore; `pct` indica che è scritto come percentuale (`n` è già diviso per 100).
#[derive(Clone, Copy, Debug)]
struct V {
    n: f64,
    pct: bool,
}

struct P {
    t: Vec<Tok>,
    i: usize,
    /// Operatori binari e funzioni incontrati: senza, non è un calcolo.
    ops: usize,
    depth: usize,
}

const MAX_DEPTH: usize = 64;

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.t.get(self.i).cloned();
        self.i += 1;
        t
    }
    fn is_ident(&self, names: &[&str]) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if names.contains(&s.as_str()))
    }

    fn expr(&mut self) -> Option<V> {
        let mut acc = self.term()?;
        while let Some(&Tok::Op(op @ ('+' | '-'))) = self.peek() {
            self.i += 1;
            self.ops += 1;
            let rhs = self.term()?;
            acc = if rhs.pct && !acc.pct {
                // 340 + 10% = 340 * 1,10 ; 340 - 10% = 340 * 0,90
                let f = if op == '+' { 1.0 + rhs.n } else { 1.0 - rhs.n };
                V { n: acc.n * f, pct: false }
            } else {
                let n = if op == '+' { acc.n + rhs.n } else { acc.n - rhs.n };
                V { n, pct: acc.pct && rhs.pct }
            };
        }
        Some(acc)
    }

    fn term(&mut self) -> Option<V> {
        let mut acc = self.power()?;
        loop {
            if acc.pct && self.is_ident(&["di", "of"]) {
                self.i += 1;
                self.ops += 1;
                let rhs = self.power()?;
                acc = V { n: acc.n * rhs.n, pct: false };
                continue;
            }
            let op = match self.peek() {
                Some(Tok::Op(c @ ('*' | '/'))) => *c,
                Some(Tok::Ident(s)) if s == "x" => '*',
                _ => break,
            };
            self.i += 1;
            self.ops += 1;
            let rhs = self.power()?;
            let n = if op == '*' {
                acc.n * rhs.n
            } else if rhs.n == 0.0 {
                return None;
            } else {
                acc.n / rhs.n
            };
            acc = V { n, pct: false };
        }
        Some(acc)
    }

    fn power(&mut self) -> Option<V> {
        let base = self.unary()?;
        if matches!(self.peek(), Some(Tok::Op('^'))) {
            self.i += 1;
            self.ops += 1;
            let exp = self.power()?; // associativa a destra
            return Some(V { n: base.n.powf(exp.n), pct: false });
        }
        Some(base)
    }

    fn unary(&mut self) -> Option<V> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return None;
        }
        let r = match self.peek() {
            Some(Tok::Op('-')) => {
                self.i += 1;
                self.unary().map(|v| V { n: -v.n, pct: v.pct })
            }
            Some(Tok::Op('+')) => {
                self.i += 1;
                self.unary()
            }
            _ => self.postfix(),
        };
        self.depth -= 1;
        r
    }

    fn postfix(&mut self) -> Option<V> {
        let v = self.primary()?;
        if matches!(self.peek(), Some(Tok::Percent)) {
            self.i += 1;
            return Some(V { n: v.n / 100.0, pct: true });
        }
        Some(v)
    }

    fn primary(&mut self) -> Option<V> {
        match self.next()? {
            Tok::Num(n) => Some(V { n, pct: false }),
            Tok::LParen => {
                let v = self.expr()?;
                matches!(self.next()?, Tok::RParen).then_some(v)
            }
            Tok::Ident(name) => match name.as_str() {
                "pi" => Some(V { n: std::f64::consts::PI, pct: false }),
                "sqrt" | "abs" => {
                    if !matches!(self.next()?, Tok::LParen) {
                        return None;
                    }
                    let v = self.expr()?;
                    if !matches!(self.next()?, Tok::RParen) {
                        return None;
                    }
                    self.ops += 1;
                    let n = if name == "sqrt" {
                        if v.n < 0.0 {
                            return None;
                        }
                        v.n.sqrt()
                    } else {
                        v.n.abs()
                    };
                    Some(V { n, pct: false })
                }
                _ => None,
            },
            _ => None,
        }
    }
}

/// Valuta `input` se è un calcolo; `None` altrimenti (testo qualunque, errore di sintassi,
/// divisione per zero, risultato non finito, nessun operatore). Pura, testata.
pub fn evaluate(input: &str) -> Option<f64> {
    let t = lex(input.trim())?;
    if t.is_empty() || t.len() > 200 {
        return None;
    }
    let mut p = P { t, i: 0, ops: 0, depth: 0 };
    let v = p.expr()?;
    if p.i != p.t.len() || p.ops == 0 || !v.n.is_finite() {
        return None;
    }
    Some(v.n)
}

/// Il numero come testo per l'utente: virgola decimale (italiano), al più 10 decimali senza zeri
/// in coda, notazione scientifica oltre 1e15. Pura, testata.
pub fn format(n: f64) -> String {
    if n == 0.0 {
        return "0".into();
    }
    if n.abs() >= 1e15 || n.abs() < 1e-9 {
        return format!("{n:e}").replace('.', ",");
    }
    let s = format!("{n:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(s: &str) -> Option<f64> {
        evaluate(s)
    }
    fn close(a: Option<f64>, b: f64) -> bool {
        a.is_some_and(|a| (a - b).abs() < 1e-9)
    }

    #[test]
    fn basic_arithmetic_and_precedence() {
        assert!(close(ev("12*7"), 84.0));
        assert!(close(ev("2+3*4"), 14.0));
        assert!(close(ev("(2+3)*4"), 20.0));
        assert!(close(ev("10/4"), 2.5));
        assert!(close(ev("10 - 2 - 3"), 5.0), "sinistra-associativa");
        assert!(close(ev("2^3^2"), 512.0), "potenza destra-associativa");
        assert!(close(ev("-3+5"), 2.0));
        assert!(close(ev("2*-3"), -6.0));
        assert!(close(ev("--4+1"), 5.0));
    }

    #[test]
    fn decimals_accept_comma_and_dot() {
        assert!(close(ev("1,5+1,5"), 3.0));
        assert!(close(ev("0.1+0.2"), 0.3));
        assert!(ev("1,2,3+1").is_none(), "due separatori in un numero");
        assert!(ev("1.2.3*2").is_none());
    }

    #[test]
    fn alternative_operator_symbols() {
        assert!(close(ev("12x7"), 84.0));
        assert!(close(ev("12 x 7"), 84.0));
        assert!(close(ev("12×7"), 84.0));
        assert!(close(ev("84÷7"), 12.0));
        assert!(close(ev("9−4"), 5.0));
    }

    #[test]
    fn percentages_work_like_spotlight() {
        assert!(close(ev("20% di 340"), 68.0));
        assert!(close(ev("20% of 340"), 68.0));
        assert!(close(ev("340 + 10%"), 374.0));
        assert!(close(ev("340 - 10%"), 306.0));
        assert!(close(ev("50% di 80 + 1"), 41.0));
        assert!(close(ev("200*10%"), 20.0));
    }

    #[test]
    fn functions_and_constants() {
        assert!(close(ev("sqrt(16)+1"), 5.0));
        assert!(close(ev("abs(-7)*2"), 14.0));
        assert!(close(ev("2*pi"), 2.0 * std::f64::consts::PI));
        assert!(ev("sqrt(-4)").is_none());
        assert!(ev("sqrt 16").is_none(), "servono le parentesi");
        assert!(ev("sin(1)+1").is_none(), "funzione sconosciuta");
    }

    #[test]
    fn a_bare_number_or_percentage_is_not_a_calculation() {
        assert!(ev("42").is_none());
        assert!(ev("-5").is_none());
        assert!(ev("50%").is_none());
        assert!(ev("(7)").is_none());
        assert!(ev("pi").is_none());
        assert!(ev("").is_none());
        assert!(ev("   ").is_none());
    }

    #[test]
    fn app_names_and_text_do_not_evaluate() {
        for s in ["7-zip", "vs code", "chrome", "2fast", "3 4", "calc 5", "x", "12 x", "x 12", "12 di 5", "1+", "(1+2", "1+2)", "()", "5 5%"] {
            assert!(ev(s).is_none(), "{s} non e' un calcolo");
        }
    }

    #[test]
    fn division_by_zero_and_overflow_give_nothing() {
        assert!(ev("5/0").is_none());
        assert!(ev("0/0").is_none());
        assert!(ev("10^1000").is_none());
        assert!(ev("(2-2)*5/(3-3)").is_none());
    }

    #[test]
    fn hostile_input_is_bounded() {
        let deep = format!("{}1+1{}", "(".repeat(5000), ")".repeat(5000));
        assert!(ev(&deep).is_none());
        let minuses = format!("{}1+1", "-".repeat(5000));
        assert!(ev(&minuses).is_none());
        assert!(ev(&"1+".repeat(5000)).is_none(), "oltre il limite di lunghezza");
        assert!(ev("1+1; rm -rf").is_none());
        assert!(ev("alert(1)+1").is_none());
    }

    #[test]
    fn format_is_tidy_and_italian() {
        assert_eq!(format(84.0), "84");
        assert_eq!(format(2.5), "2,5");
        assert_eq!(format(0.1 + 0.2), "0,3");
        assert_eq!(format(-1.0 / 3.0), "-0,3333333333");
        assert_eq!(format(0.0), "0");
        assert_eq!(format(-0.0), "0");
        assert_eq!(format(1e21), "1e21");
        assert_eq!(format(1.5e-12), "1,5e-12");
        assert_eq!(format(123456789.0), "123456789");
    }
}
