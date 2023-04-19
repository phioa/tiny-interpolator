use num::pow::Pow;
use num::{BigRational, BigUint, One, Zero};

pub type Vector = Vec<BigRational>;
pub type Matrix = Vec<Vector>; // 行向量在内

pub fn interpolate(xs: Vector, ys: Vector) -> Vector {
    let n = xs.len();
    assert!(n != 0);
    assert_eq!(ys.len(), n);
    let mut mat: Matrix = xs
        .into_iter()
        .map(|x| {
            (0..n)
                .into_iter()
                .map(|i| <BigRational as Pow<BigUint>>::pow(x.clone(), BigUint::from(i)))
                .collect()
        })
        .collect();
    for (v, y) in mat.iter_mut().zip(ys.into_iter()) {
        v.push(y)
    }
    let mat = solve(mat);
    assert!(is_triangular(&mat));
    let mut res: Vector = mat
        .into_iter()
        .map(|v| v.into_iter().rev().next().unwrap())
        .collect();
    while let Some(a) = res.last() {
        if a.is_zero() {
            res.pop();
        } else {
            break;
        }
    }
    if res.is_empty() {
        vec![BigRational::zero()]
    } else {
        res
    }
}

pub fn is_triangular(mat: &Matrix) -> bool {
    if mat.is_empty() {
        return false;
    }
    if mat[0].len() <= mat.len() {
        return false;
    }
    mat.iter()
        .enumerate()
        .all(|(i, v)| (0..i).all(|k| v[k].is_zero()) && !v[i].is_zero())
}

/// 将一矩阵变换为行最简式
pub fn solve(mat: Matrix) -> Matrix {
    back_substitute(eliminate(mat))
}

fn eliminate(mut mat: Matrix) -> Matrix {
    if mat.is_empty() {
        return mat;
    }
    let m = mat.len();
    let n = mat[0].len();
    let (mut pivotal_column, mut current_row) = (0, 0);
    while current_row < m && pivotal_column < n {
        let mut found = false;
        for pivotal_row in current_row..m {
            if !mat[pivotal_row][pivotal_column].is_zero() {
                mat.swap(current_row, pivotal_row);
                found = true;
                break;
            }
        }
        if !found {
            pivotal_column += 1;
            continue;
        }
        let pivot = mat[current_row][pivotal_column].clone();
        for j in pivotal_column..n {
            mat[current_row][j] /= pivot.clone();
        }
        for i in current_row + 1..m {
            let elim = mat[i][pivotal_column].clone();
            for j in pivotal_column..n {
                let temp = mat[current_row][j].clone();
                mat[i][j] -= elim.clone() * temp;
            }
        }
        current_row += 1;
        pivotal_column += 1;
    }
    mat
}

fn back_substitute(mut mat: Matrix) -> Matrix {
    if mat.is_empty() {
        return mat;
    }
    mat.reverse();
    let m = mat.len();
    let n = mat[0].len();
    let mut current_row = 0;
    let mut pivotal_column = n;
    while current_row < m {
        let mut found = false;
        for j in 0..pivotal_column {
            if !mat[current_row][j].is_zero() {
                pivotal_column = j;
                found = true;
                break;
            }
        }
        if !found {
            current_row += 1;
            continue;
        }
        assert!(mat[current_row][pivotal_column].is_one());
        for i in current_row + 1..m {
            let elim = mat[i][pivotal_column].clone();
            for j in pivotal_column..n {
                let temp = mat[current_row][j].clone();
                mat[i][j] -= elim.clone() * temp;
            }
        }
        current_row += 1;
    }
    mat.reverse();
    mat
}

#[cfg(test)]
mod test {
    use super::*;
    use num::FromPrimitive;

    macro_rules! brvec {
        ($($x:expr), *) => {
            vec![$(BigRational::from_f64($x).unwrap()), *]
        }
    }

    #[test]
    fn test_solve() {
        assert_eq!(
            solve(vec![
                brvec![1., 2., 3., 4.],
                brvec![4., 5., 6., 7.],
                brvec![7., 8., 9., 10.],
            ]),
            vec![
                brvec![1., 0., -1., -2.],
                brvec![0., 1., 2., 3.],
                brvec![0., 0., 0., 0.],
            ]
        );
    }

    #[test]
    fn test_interpolate() {
        assert_eq!(
            interpolate(brvec![1., 2., 3., 4.], brvec![1., 3., 5., 10.]),
            brvec![-4., 7.5, -3., 0.5]
        );
    }

    #[test]
    #[should_panic]
    fn test_interpolate_panic() {
        assert_ne!(
            interpolate(brvec![1., 1., 2.], brvec![2., 2., 2.]),
            brvec![]
        )
    }

    #[test]
    fn test_zeroth_power_of_zero() {
        assert_eq!(
            <BigRational as Pow<BigUint>>::pow(BigRational::zero(), BigUint::from(0usize)),
            BigRational::one()
        )
    }
}
