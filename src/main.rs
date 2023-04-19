use num::pow::Pow;
use num::{BigRational, BigUint, One, Signed, Zero};
use std::collections::HashMap;
use std::io::Write;
use tiny_interpolator::*;

fn main() {
    let mut polys = PolyMap::new();
    let mut next_name = 1u128;
    let commands = CmdMap::from([
        (
            String::from("help"),
            Box::new(|_: &mut _, _: &mut _, _: _| {
                println!("help -- show the command list.");
                println!("itrp (x1 x2 ... xn) (y1 y2 ... yn) -- calculate a polynomial, which is evaluated y1, y2, ... yn when x=x1, x2, ..., xn.");
                println!(
                    "eval (a0 a1 ... an) x -- evaluate the polynomial a0+a1*x+a2*x^2+...+an*x^n."
                );
                println!("ls -- list all polynomials stored in the map.");
                println!("print (a0 a1 ... an) -- print a polynomial.");
                println!("add name (a0 a1 ... an) -- add a named polynomial in the map.");
                println!("rn old-name new-name -- rename a polynomial in the map.");
                println!("rm name -- remove a polynomial from the map.");
                println!("NOTE: names in the map can be used as vectors.");
            }) as Cmd,
        ),
        (
            String::from("itrp"),
            Box::new(
                |polys: &mut PolyMap, next_name: &mut u128, cmd: Vec<String>| {
                    if cmd.len() < 3 {
                        println!("not enough arguments!");
                        return;
                    }
                    let xs = match parse_vector(&cmd[1], &polys) {
                        Ok(xs) => xs,
                        Err(s) => {
                            println!("{}", s);
                            return;
                        }
                    };
                    if xs.is_empty() {
                        println!("xs should not be empty.");
                        return;
                    }
                    if !is_unique(&xs) {
                        println!("all xs should be unique.");
                        return;
                    }

                    let ys = match parse_vector(&cmd[2], &polys) {
                        Ok(ys) => ys,
                        Err(s) => {
                            println!("{}", s);
                            return;
                        }
                    };
                    if xs.len() != ys.len() {
                        println!("xs and ys should be in the same length.");
                        return;
                    }

                    let res = interpolate(xs, ys);
                    print_vec(&res);
                    println!();
                    let name = format!("p{}", next_name);
                    println!("saved as '{}' in map.", name);
                    polys.insert(name, res);
                    *next_name += 1;
                },
            ),
        ),
        (
            String::from("eval"),
            Box::new(|polys, _, cmd| {
                if cmd.len() < 3 {
                    println!("not enough arguments!");
                    return;
                }
                let coeffs = match parse_vector(&cmd[1], &polys) {
                    Ok(coeffs) => coeffs,
                    Err(s) => {
                        println!("{}", s);
                        return;
                    }
                };
                let x = match cmd[2].parse::<BigRational>() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                };
                let res: BigRational = coeffs
                    .into_iter()
                    .enumerate()
                    .map(|(i, a)| {
                        a * <BigRational as Pow<BigUint>>::pow(x.clone(), BigUint::from(i))
                    })
                    .sum();
                println!("{}", res);
            }),
        ),
        (
            String::from("ls"),
            Box::new(|polys, _, _| {
                for (k, v) in polys {
                    print!("{}: ", k);
                    print_vec(v);
                    println!();
                }
            }),
        ),
        (
            String::from("print"),
            Box::new(|polys, _, cmd| {
                if cmd.len() < 2 {
                    println!("not enough arguments!");
                    return;
                }
                let coeffs = match parse_vector(&cmd[1], &polys) {
                    Ok(coeffs) => coeffs,
                    Err(s) => {
                        println!("{}", s);
                        return;
                    }
                };
                print!("f(x)=");
                if coeffs.iter().all(BigRational::is_zero) {
                    print!("0");
                } else {
                    let mut first_time = true;
                    for (i, a) in coeffs.into_iter().enumerate().rev() {
                        if a.is_zero() {
                            continue;
                        }
                        if !first_time {
                            if a.is_positive() {
                                print!("+");
                            }
                        } else {
                            first_time = false;
                        }
                        if a == -BigRational::one() {
                            print!("-");
                        } else if !a.is_one() || i == 0 {
                            print!("{}", a);
                        }
                        if i > 0 {
                            if !a.is_one() && a != -BigRational::one() {
                                print!("*");
                            }
                            print!("x");
                            if i > 1 {
                                print!("^{}", i);
                            }
                        }
                    }
                }
                println!();
            }),
        ),
        (
            String::from("add"),
            Box::new(|polys, _, cmd| {
                if cmd.len() < 3 {
                    println!("not enough arguments!");
                    return;
                }
                let v = match parse_vector(&cmd[2], &polys) {
                    Ok(v) => v,
                    Err(s) => {
                        println!("{}", s);
                        return;
                    }
                };
                print!("{}: ", cmd[1]);
                print_vec(&v);
                println!();
                polys.insert(cmd[1].clone(), v);
            }),
        ),
        (
            String::from("rn"),
            Box::new(|polys, _, cmd| {
                if cmd.len() < 3 {
                    println!("not enough arguments!");
                    return;
                }
                let old = polys.get(&cmd[1]);
                let old = if let Some(old) = old {
                    old
                } else {
                    println!("unknown name.");
                    return;
                };
                let old = old.clone();
                polys.remove(&cmd[1]);
                print!("{}: ", cmd[2]);
                print_vec(&old);
                println!();
                polys.insert(cmd[2].clone(), old);
            }),
        ),
        (
            String::from("rm"),
            Box::new(|polys, _, cmd| {
                if cmd.len() < 2 {
                    println!("not enough arguments!");
                    return;
                }
                if let None = polys.remove(&cmd[1]) {
                    println!("unknown name.");
                }
            }),
        ),
    ]);
    println!("a Tiny Polynomial Interpolator. Type 'help' for more information.");
    run_console(&mut polys, &mut next_name, &commands);
}

type PolyMap = HashMap<String, Vector>;
type Cmd = Box<dyn Fn(&mut PolyMap, &mut u128, Vec<String>)>;
type CmdMap = HashMap<String, Cmd>;

fn run_console(polys: &mut PolyMap, next_name: &mut u128, commands: &CmdMap) {
    loop {
        let mut input = String::new();
        print!(">>");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let cmd = chop_command(&input);
        if cmd.is_empty() {
            continue;
        }
        if let Some(f) = commands.get(&cmd[0]) {
            f(polys, next_name, cmd);
        } else if cmd[0] == "quit" {
            break;
        } else {
            println!("Unknown command '{}'.", cmd[0]);
        }
    }
}

fn chop_command<'a>(command: &str) -> Vec<String> {
    let iter = command.trim().split_whitespace();
    let mut res = Vec::new();
    let mut buf = String::new();
    for s in iter {
        if buf.is_empty() {
            if s.starts_with('(') && !s.ends_with(')') {
                buf.push_str(s);
                buf.push(' ');
            } else {
                res.push(String::from(s));
            }
        } else {
            buf.push_str(s);
            if s.ends_with(')') {
                res.push(buf);
                buf = String::new();
            } else {
                buf.push(' ');
            }
        }
    }
    if !buf.is_empty() {
        res.push(buf);
    }
    res
}

fn parse_vector(s: &str, polys: &PolyMap) -> Result<Vector, String> {
    assert!(!s.is_empty());
    if s.chars().next().unwrap() != '(' {
        polys.get(s).cloned().ok_or(format!("unknown name '{}'", s))
    } else {
        if !s.ends_with(')') {
            Err(format!("invalid vector '{}'", s))
        } else {
            let iter = s[1..s.len() - 1]
                .split_whitespace()
                .map(|subs| subs.parse::<BigRational>().map_err(|e| format!("{}", e)));
            let mut res = Vec::new();
            for scalar in iter {
                res.push(scalar?);
            }
            Ok(res)
        }
    }
}

fn is_unique(vec: &Vector) -> bool {
    let mut vec = vec.clone();
    vec.sort();
    vec.windows(2).all(|w| w[0] != w[1])
}

fn print_vec(cr: &Vector) {
    print!("(");
    let mut first_time = true;
    for x in cr {
        if first_time {
            first_time = false;
        } else {
            print!(" ");
        }
        print!("{}", x);
    }
    print!(")");
}
