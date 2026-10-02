//! Exact discrete operators for manufactured tests; no continuum lifting claim.
use crate::{Error, Rational, MAX_COLS, MAX_ROWS};

/// Periodic edge-coefficient diffusion on a uniform grid:
/// (A b)_i = [b_i(u_{i+1}-u_i) - b_{i-1}(u_i-u_{i-1})] / h^2.
/// Its exact discrete work identity is
/// h * sum_i u_i (A b)_i = -sum_i b_i (u_{i+1}-u_i)^2 / h.
/// This represents a one-dimensional invariant shear class, not arbitrary 3-D flow.
pub fn periodic_diffusion(u: &[Rational], h: Rational) -> Result<Vec<Vec<Rational>>, Error> {
    let n = u.len();
    if !(3..=MAX_COLS).contains(&n) || h == Rational::ZERO || h.negative() {
        return Err(Error::Shape);
    }
    let h2 = h.times(h)?;
    let mut matrix = vec![vec![Rational::ZERO; n]; n];
    for i in 0..n {
        let previous = (i + n - 1) % n;
        let next = (i + 1) % n;
        matrix[i][i] = u[next].minus(u[i])?.divided_by(h2)?;
        matrix[i][previous] = u[previous].minus(u[i])?.divided_by(h2)?;
    }
    Ok(matrix)
}

/// Equal material labels across supplied time slices: b_(t+1,l) = b_(t,l).
/// These are explicit finite transport equalities, not an advection discretization
/// or a numerical proof that the supplied labels follow a physical flow.
pub fn material_label_equalities(
    slices: usize,
    labels: usize,
) -> Result<Vec<Vec<Rational>>, Error> {
    let columns = slices.checked_mul(labels).ok_or(Error::Limit)?;
    if slices < 2 || labels == 0 || columns > MAX_COLS {
        return Err(Error::Shape);
    }
    let rows = (slices - 1).checked_mul(labels).ok_or(Error::Limit)?;
    if rows > MAX_ROWS {
        return Err(Error::Limit);
    }
    let mut matrix = vec![vec![Rational::ZERO; columns]; rows];
    for t in 0..slices - 1 {
        for label in 0..labels {
            let row = t * labels + label;
            matrix[row][t * labels + label] = Rational::ONE.negated()?;
            matrix[row][(t + 1) * labels + label] = Rational::ONE;
        }
    }
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dot, Problem};

    fn q(n: i128) -> Rational {
        Rational::new(n, 1).unwrap()
    }

    #[test]
    fn discrete_energy_identity_and_mass_conservation() {
        let u = [q(0), q(1), q(0), q(-1)];
        let b = [q(1), q(2), q(3), q(4)];
        let h = Rational::new(1, 2).unwrap();
        let a = periodic_diffusion(&u, h).unwrap();
        let rhs: Vec<_> = a.iter().map(|row| dot(row, &b).unwrap()).collect();
        let work = h.times(dot(&u, &rhs).unwrap()).unwrap();
        let mut dissipation = Rational::ZERO;
        for i in 0..u.len() {
            let delta = u[(i + 1) % u.len()].minus(u[i]).unwrap();
            dissipation = dissipation
                .plus(
                    b[i].times(delta.times(delta).unwrap())
                        .unwrap()
                        .divided_by(h)
                        .unwrap(),
                )
                .unwrap();
        }
        assert_eq!(work, dissipation.negated().unwrap());
        assert_eq!(dot(&rhs, &vec![q(1); rhs.len()]).unwrap(), q(0));
        assert!(Problem::new(a, rhs, vec![])
            .unwrap()
            .verify_primal(&b)
            .unwrap());
    }

    #[test]
    fn constant_shear_and_invalid_grids() {
        let a = periodic_diffusion(&[q(2); 4], q(1)).unwrap();
        assert!(a.iter().flatten().all(|x| *x == q(0)));
        assert!(periodic_diffusion(&[q(1); 2], q(1)).is_err());
        assert!(periodic_diffusion(&[q(1); 4], q(0)).is_err());
        assert!(periodic_diffusion(&[q(1); 4], q(-1)).is_err());
    }

    #[test]
    fn material_labels_reject_instantaneous_only_fits() {
        let t = material_label_equalities(2, 1).unwrap();
        let a = vec![vec![q(1), q(0)], vec![q(0), q(1)]];
        let p = Problem::new(a, vec![q(1), q(2)], t).unwrap();
        assert!(!p.verify_primal(&[q(1), q(2)]).unwrap());
        assert!(p.verify_dual(&[q(-1), q(1), q(-1)]).unwrap());
        assert!(material_label_equalities(usize::MAX, 2).is_err());
        assert!(material_label_equalities(1, 1).is_err());
    }
}
