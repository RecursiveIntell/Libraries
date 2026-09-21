//! Exact, bounded finite-dimensional witnesses. No continuum or truth admission.
//!
//! The problem is A b = r, T b = 0, b >= 0. A dual witness satisfies
//! [A; T]^T y <= 0 and [r; 0]^T y > 0. Both cannot hold simultaneously.
//! Search is deliberately incomplete; only checked witnesses produce decisions.
#![forbid(unsafe_code)]

pub mod operators;

use std::fmt;
use std::str::FromStr;

pub const MAX_BYTES: usize = 65_536;
pub const MAX_ROWS: usize = 32;
pub const MAX_COLS: usize = 16;
pub const MAX_ATTEMPTS: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Syntax,
    Shape,
    Limit,
    Arithmetic,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Reduced rational with positive denominator. Overflow is an error, never rounding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rational {
    num: i128,
    den: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Rational {
    pub const ZERO: Self = Self { num: 0, den: 1 };
    pub const ONE: Self = Self { num: 1, den: 1 };

    pub fn new(num: i128, den: i128) -> Result<Self, Error> {
        if den <= 0 {
            return Err(Error::Arithmetic);
        }
        let abs = num.checked_abs().ok_or(Error::Arithmetic)?;
        let g = gcd(abs, den);
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }

    pub fn negative(self) -> bool {
        self.num < 0
    }

    pub fn negated(self) -> Result<Self, Error> {
        Self::new(self.num.checked_neg().ok_or(Error::Arithmetic)?, self.den)
    }

    pub fn plus(self, rhs: Self) -> Result<Self, Error> {
        let g = gcd(self.den, rhs.den);
        let left = self.num.checked_mul(rhs.den / g).ok_or(Error::Arithmetic)?;
        let right = rhs.num.checked_mul(self.den / g).ok_or(Error::Arithmetic)?;
        let num = left.checked_add(right).ok_or(Error::Arithmetic)?;
        let den = self.den.checked_mul(rhs.den / g).ok_or(Error::Arithmetic)?;
        Self::new(num, den)
    }

    pub fn minus(self, rhs: Self) -> Result<Self, Error> {
        self.plus(rhs.negated()?)
    }

    pub fn times(self, rhs: Self) -> Result<Self, Error> {
        let g = gcd(self.num.checked_abs().ok_or(Error::Arithmetic)?, rhs.den);
        let h = gcd(rhs.num.checked_abs().ok_or(Error::Arithmetic)?, self.den);
        let num = (self.num / g)
            .checked_mul(rhs.num / h)
            .ok_or(Error::Arithmetic)?;
        let den = (self.den / h)
            .checked_mul(rhs.den / g)
            .ok_or(Error::Arithmetic)?;
        Self::new(num, den)
    }

    pub fn divided_by(self, rhs: Self) -> Result<Self, Error> {
        if rhs.num == 0 {
            return Err(Error::Arithmetic);
        }
        let num = if rhs.num < 0 { -rhs.den } else { rhs.den };
        let den = rhs.num.checked_abs().ok_or(Error::Arithmetic)?;
        self.times(Self::new(num, den)?)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

impl FromStr for Rational {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        if s.len() > 80 {
            return Err(Error::Limit);
        }
        let (n, d) = s.split_once('/').ok_or(Error::Syntax)?;
        let num = n.parse::<i128>().map_err(|_| Error::Syntax)?;
        let den = d.parse::<i128>().map_err(|_| Error::Syntax)?;
        let value = Self::new(num, den)?;
        if value.to_string() != s {
            return Err(Error::Syntax);
        }
        Ok(value)
    }
}

fn dot(a: &[Rational], b: &[Rational]) -> Result<Rational, Error> {
    if a.len() != b.len() {
        return Err(Error::Shape);
    }
    a.iter()
        .zip(b)
        .try_fold(Rational::ZERO, |acc, (x, y)| acc.plus(x.times(*y)?))
}

/// A supplied discretization, not a certified map from a PDE to a matrix.
#[derive(Clone, Debug)]
pub struct Problem {
    a: Vec<Vec<Rational>>,
    rhs: Vec<Rational>,
    transport: Vec<Vec<Rational>>,
}

impl Problem {
    pub fn new(
        a: Vec<Vec<Rational>>,
        rhs: Vec<Rational>,
        transport: Vec<Vec<Rational>>,
    ) -> Result<Self, Error> {
        let n = a.first().map_or(0, Vec::len);
        if a.is_empty() || n == 0 || rhs.len() != a.len() {
            return Err(Error::Shape);
        }
        if a.len() + transport.len() > MAX_ROWS || n > MAX_COLS {
            return Err(Error::Limit);
        }
        if a.iter().chain(&transport).any(|row| row.len() != n) {
            return Err(Error::Shape);
        }
        Ok(Self { a, rhs, transport })
    }

    fn augmented(&self) -> (Vec<Vec<Rational>>, Vec<Rational>) {
        let mut matrix = self.a.clone();
        matrix.extend(self.transport.clone());
        let mut rhs = self.rhs.clone();
        rhs.resize(matrix.len(), Rational::ZERO);
        (matrix, rhs)
    }

    pub fn verify_primal(&self, b: &[Rational]) -> Result<bool, Error> {
        if b.len() != self.a[0].len() {
            return Err(Error::Shape);
        }
        if b.iter().any(|x| x.negative()) {
            return Ok(false);
        }
        let (matrix, rhs) = self.augmented();
        for (row, target) in matrix.iter().zip(rhs) {
            if dot(row, b)? != target {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn verify_dual(&self, y: &[Rational]) -> Result<bool, Error> {
        let (matrix, rhs) = self.augmented();
        if y.len() != matrix.len() {
            return Err(Error::Shape);
        }
        let margin = dot(&rhs, y)?;
        if margin.num <= 0 {
            return Ok(false);
        }
        for j in 0..self.a[0].len() {
            let column: Vec<_> = matrix.iter().map(|row| row[j]).collect();
            if dot(&column, y)?.num > 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Primal(Vec<Rational>),
    Dual(Vec<Rational>),
    Unresolved(&'static str),
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub outcome: Outcome,
    pub attempts: usize,
}

impl SearchResult {
    /// Candidate JSON. The evidence consumer must bind inputs and recheck it.
    pub fn to_json(&self) -> String {
        let (kind, values, reason) = match &self.outcome {
            Outcome::Primal(v) => ("primal", v.as_slice(), "checked_exactly"),
            Outcome::Dual(v) => ("dual", v.as_slice(), "checked_exactly"),
            Outcome::Unresolved(r) => ("unresolved", &[][..], *r),
        };
        // Public library callers cannot inject an arbitrary reason into JSON.
        let reason = match reason {
            "checked_exactly" | "search_budget_exhausted" | "no_certificate_in_bounded_search" => {
                reason
            }
            _ => "unrecognized_reason",
        };
        let values = values
            .iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"cw.candidate.v1\",\"scope\":\"finite_dimensional\",\"kind\":\"{kind}\",\"values\":[{values}],\"attempts\":{},\"reason\":\"{reason}\"}}",
            self.attempts
        )
    }
}

fn columns_solution(
    matrix: &[Vec<Rational>],
    rhs: &[Rational],
    selected: &[usize],
) -> Result<Option<Vec<Rational>>, Error> {
    let k = selected.len();
    if k > matrix.len() {
        return Ok(None);
    }
    let mut work: Vec<Vec<_>> = matrix
        .iter()
        .zip(rhs)
        .map(|(row, value)| {
            let mut r: Vec<_> = selected.iter().map(|j| row[*j]).collect();
            r.push(*value);
            r
        })
        .collect();
    for col in 0..k {
        let Some(pivot) = (col..work.len()).find(|i| work[*i][col] != Rational::ZERO) else {
            return Ok(None);
        };
        work.swap(col, pivot);
        let divisor = work[col][col];
        for value in work[col].iter_mut().take(k + 1).skip(col) {
            *value = (*value).divided_by(divisor)?;
        }
        let pivot_row = work[col].clone();
        for (i, row) in work.iter_mut().enumerate() {
            if i != col {
                let factor = row[col];
                for j in col..=k {
                    row[j] = row[j].minus(factor.times(pivot_row[j])?)?;
                }
            }
        }
    }
    if work.iter().skip(k).any(|row| row[k] != Rational::ZERO) {
        return Ok(None);
    }
    Ok(Some(work.iter().take(k).map(|row| row[k]).collect()))
}

/// Bounded coordinate/pair dual search followed by exact basic-column search.
/// It is not an optimized LP solver and never infers infeasibility from failure.
pub fn solve(problem: &Problem, budget: usize) -> Result<SearchResult, Error> {
    if budget == 0 || budget > MAX_ATTEMPTS {
        return Err(Error::Limit);
    }
    let (matrix, rhs) = problem.augmented();
    let n = problem.a[0].len();
    let mut attempts = 1;
    let zero = vec![Rational::ZERO; n];
    if problem.verify_primal(&zero)? {
        return Ok(SearchResult {
            outcome: Outcome::Primal(zero),
            attempts,
        });
    }
    // Inspect one- and two-row separating witnesses before expensive elimination.
    for i in 0..matrix.len() {
        for j in i..matrix.len() {
            for left in [-1, 1] {
                for right in [-1, 1] {
                    if attempts >= budget {
                        return Ok(SearchResult {
                            outcome: Outcome::Unresolved("search_budget_exhausted"),
                            attempts,
                        });
                    }
                    let mut y = vec![Rational::ZERO; matrix.len()];
                    y[i] = Rational::new(left, 1)?;
                    if j != i {
                        y[j] = Rational::new(right, 1)?;
                    }
                    attempts += 1;
                    if problem.verify_dual(&y)? {
                        return Ok(SearchResult {
                            outcome: Outcome::Dual(y),
                            attempts,
                        });
                    }
                }
            }
        }
    }
    for mask in 1usize..(1usize << n) {
        if mask.count_ones() as usize > matrix.len() {
            continue;
        }
        if attempts >= budget {
            return Ok(SearchResult {
                outcome: Outcome::Unresolved("search_budget_exhausted"),
                attempts,
            });
        }
        attempts += 1;
        let selected: Vec<_> = (0..n).filter(|j| mask & (1usize << j) != 0).collect();
        if let Some(values) = columns_solution(&matrix, &rhs, &selected)? {
            let mut b = vec![Rational::ZERO; n];
            for (j, value) in selected.iter().zip(values) {
                b[*j] = value;
            }
            if problem.verify_primal(&b)? {
                return Ok(SearchResult {
                    outcome: Outcome::Primal(b),
                    attempts,
                });
            }
        }
    }
    // A small exhaustive ternary dual grid catches transport/range combinations
    // needing three or more rows without displacing the earlier primal search.
    if matrix.len() <= 6 {
        for code in 1usize..3usize.pow(matrix.len() as u32) {
            if attempts >= budget {
                return Ok(SearchResult {
                    outcome: Outcome::Unresolved("search_budget_exhausted"),
                    attempts,
                });
            }
            attempts += 1;
            let mut digits = code;
            let mut y = Vec::with_capacity(matrix.len());
            for _ in 0..matrix.len() {
                let digit = match digits % 3 {
                    0 => 0,
                    1 => 1,
                    _ => -1,
                };
                y.push(Rational::new(digit, 1)?);
                digits /= 3;
            }
            if problem.verify_dual(&y)? {
                return Ok(SearchResult {
                    outcome: Outcome::Dual(y),
                    attempts,
                });
            }
        }
    }
    Ok(SearchResult {
        outcome: Outcome::Unresolved("no_certificate_in_bounded_search"),
        attempts,
    })
}

/// Strict native wire: CW1 m n k budget, then row-major A, r, row-major T.
/// Every number is a canonical reduced p/q. No trailing tokens are admitted.
pub fn parse_request(text: &str) -> Result<(Problem, usize), Error> {
    if text.len() > MAX_BYTES || !text.is_ascii() {
        return Err(Error::Limit);
    }
    let mut words = text.split_ascii_whitespace();
    if words.next() != Some("CW1") {
        return Err(Error::Syntax);
    }
    let mut integer = || -> Result<usize, Error> {
        let raw = words.next().ok_or(Error::Syntax)?;
        let n = raw.parse::<usize>().map_err(|_| Error::Syntax)?;
        if n.to_string() != raw {
            return Err(Error::Syntax);
        }
        Ok(n)
    };
    let m = integer()?;
    let n = integer()?;
    let k = integer()?;
    let budget = integer()?;
    if m == 0
        || n == 0
        || m > MAX_ROWS
        || k > MAX_ROWS
        || m + k > MAX_ROWS
        || n > MAX_COLS
        || budget == 0
        || budget > MAX_ATTEMPTS
    {
        return Err(Error::Limit);
    }
    let mut row = || -> Result<Vec<Rational>, Error> {
        (0..n)
            .map(|_| words.next().ok_or(Error::Syntax)?.parse())
            .collect()
    };
    let a = (0..m).map(|_| row()).collect::<Result<Vec<_>, _>>()?;
    let rhs = (0..m)
        .map(|_| words.next().ok_or(Error::Syntax)?.parse())
        .collect::<Result<Vec<_>, _>>()?;
    let mut transport = Vec::with_capacity(k);
    for _ in 0..k {
        transport.push(
            (0..n)
                .map(|_| words.next().ok_or(Error::Syntax)?.parse())
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    if words.next().is_some() {
        return Err(Error::Syntax);
    }
    Ok((Problem::new(a, rhs, transport)?, budget))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i128) -> Rational {
        Rational::new(n, 1).unwrap()
    }
    fn p(a: &[&[i128]], rhs: &[i128], t: &[&[i128]]) -> Problem {
        Problem::new(
            a.iter()
                .map(|r| r.iter().map(|v| q(*v)).collect())
                .collect(),
            rhs.iter().map(|v| q(*v)).collect(),
            t.iter()
                .map(|r| r.iter().map(|v| q(*v)).collect())
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn canonical_wire_rejects_ambiguous_numbers() {
        for s in [
            "1", "01/1", "1/01", "2/2", "0/2", "-0/1", "+1/1", "1/-1", "1/0", "NaN", "1/1/1",
        ] {
            assert!(s.parse::<Rational>().is_err(), "{s}");
        }
        assert_eq!("-1/2".parse::<Rational>().unwrap().to_string(), "-1/2");
    }

    #[test]
    fn exact_arithmetic_and_overflow() {
        let a = Rational::new(1, 3).unwrap();
        let b = Rational::new(1, 6).unwrap();
        assert_eq!(a.plus(b).unwrap(), Rational::new(1, 2).unwrap());
        assert_eq!(a.divided_by(b).unwrap(), q(2));
        assert_eq!(q(-2).times(q(-3)).unwrap(), q(6));
        assert!(q(1).divided_by(q(0)).is_err());
        assert!(Rational::new(i128::MIN, 1).is_err());
        assert!(q(i128::MAX).plus(q(1)).is_err());
        assert_eq!(
            Rational::new(i128::MAX, 2)
                .unwrap()
                .times(Rational::new(2, i128::MAX).unwrap())
                .unwrap(),
            q(1)
        );
    }

    #[test]
    fn primal_and_energy_separator() {
        let good = p(&[&[-1]], &[-2], &[]);
        assert!(good.verify_primal(&[q(2)]).unwrap());
        assert!(!good.verify_primal(&[q(-2)]).unwrap());
        let bad = p(&[&[-1]], &[1], &[]);
        assert!(bad.verify_dual(&[q(1)]).unwrap());
        assert!(!bad.verify_dual(&[q(-1)]).unwrap());
        assert!(matches!(
            solve(&bad, 100).unwrap().outcome,
            Outcome::Dual(_)
        ));
    }

    #[test]
    fn range_obstruction_uses_zero_row() {
        let problem = p(&[&[-1], &[0]], &[-1, 1], &[]);
        assert!(problem.verify_dual(&[q(0), q(1)]).unwrap());
        assert!(matches!(
            solve(&problem, 100).unwrap().outcome,
            Outcome::Dual(_)
        ));
    }

    #[test]
    fn transport_is_not_optional() {
        let problem = p(&[&[1, 0], &[0, 1]], &[1, 2], &[&[-1, 1]]);
        assert!(!problem.verify_primal(&[q(1), q(2)]).unwrap());
        assert!(problem.verify_dual(&[q(-1), q(1), q(-1)]).unwrap());
        assert!(problem.verify_dual(&[q(-1), q(1)]).is_err());
        assert!(matches!(
            solve(&problem, 500).unwrap().outcome,
            Outcome::Dual(_)
        ));
    }

    #[test]
    fn solver_checks_multi_column_and_dependent_systems() {
        for problem in [
            p(&[&[1, 0], &[0, 1]], &[2, 3], &[]),
            p(&[&[1, 1], &[2, 2]], &[3, 6], &[]),
        ] {
            let result = solve(&problem, 500).unwrap();
            let Outcome::Primal(b) = result.outcome else {
                panic!("missing primal")
            };
            assert!(problem.verify_primal(&b).unwrap());
        }
    }

    #[test]
    fn zero_case_and_strict_margin() {
        let problem = p(&[&[0]], &[0], &[]);
        assert!(matches!(
            solve(&problem, 1).unwrap().outcome,
            Outcome::Primal(_)
        ));
        assert!(!problem.verify_dual(&[q(1)]).unwrap());
    }

    #[test]
    fn budget_exhaustion_is_unresolved() {
        let result = solve(&p(&[&[1]], &[2], &[]), 1).unwrap();
        assert_eq!(
            result.outcome,
            Outcome::Unresolved("search_budget_exhausted")
        );
        assert_eq!(result.attempts, 1);
        assert!(solve(&p(&[&[1]], &[2], &[]), 0).is_err());
    }

    #[test]
    fn wire_dimensions_trailing_and_limits() {
        assert!(parse_request("CW1 1 1 0 50 -1/1 2/1").is_ok());
        for wire in [
            "CW2 1 1 0 50 -1/1 2/1",
            "CW1 1 1 0 50 -1/1 2/1 extra",
            "CW1 33 1 0 50",
            "CW1 1 17 0 50",
            "CW1 1 1 32 50",
            "CW1 01 1 0 50",
            "CW1 1 1 0 50 -1/1",
            "CW1 0 1 0 50",
        ] {
            assert!(parse_request(wire).is_err(), "{wire}");
        }
    }

    #[test]
    fn public_result_reason_cannot_inject_json() {
        let report = SearchResult {
            outcome: Outcome::Unresolved("bad\"reason"),
            attempts: 1,
        };
        assert!(report.to_json().contains("unrecognized_reason"));
        assert!(!report.to_json().contains("bad"));
    }

    #[test]
    fn wrong_shapes_and_tampered_certificates() {
        assert!(Problem::new(vec![], vec![], vec![]).is_err());
        assert!(Problem::new(vec![vec![q(1)]], vec![q(1)], vec![vec![]]).is_err());
        let problem = p(&[&[2, 0], &[0, 3]], &[4, 9], &[]);
        assert!(problem.verify_primal(&[q(2), q(3)]).unwrap());
        assert!(!problem.verify_primal(&[q(2), q(4)]).unwrap());
        assert!(problem.verify_primal(&[q(2)]).is_err());
    }
}
