mod operator;

use crate::operator::{Operator, OPERATORS};
use itertools::iproduct;
use num::rational::Rational64;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct RationalSearch {
    denom_cut: i64,
    generatable_nums: Vec<Vec<Rational64>>,
    known_expr: HashMap<Rational64, String>,
}

impl RationalSearch {
    fn from_lists(num_lists: &Vec<Vec<Rational64>>, denom_cut: i64) -> Self {
        let mut exprs = HashMap::default();
        for num_list in num_lists {
            match num_list.len() {
                1 => {
                    exprs.insert(
                        num_list[0],
                        num_list[0].to_string()
                    );
                },
                2 => { exprs.extend(generate_pair_expr(num_list[0], num_list[1])); },
                3 => { exprs.extend(generate_triple_expr(num_list[0], num_list[1], num_list[2])); },
                _ => { unimplemented!(); }
            }
        }

        let first_generatable = exprs
            .iter()
            .map(|(num, _)| *num)
            .collect();

        Self {
            denom_cut,
            generatable_nums: vec![first_generatable],
            known_expr: exprs
        }
    }

    fn extend(&mut self, n: usize) {
        let len = self.generatable_nums.len();
        if len >= n {
            return;
        }

        for k in len..n {
            let generatable_nums = &self.generatable_nums;
            let mut next_generatable = Vec::new();
            let loop_iter = (0..k).zip((0..k).rev())
                .flat_map(move |(i, j)| {
                    iproduct!(&generatable_nums[i], &generatable_nums[j])
                });

            for (lval, rval) in loop_iter {
                for op in &OPERATORS {
                    if (op == &Operator::Add || op == &Operator::Mul) &&
                            lval > rval {
                        continue;
                    }

                    if let Some(num) = op.invoke(*lval, *rval) {
                        if num.denom() < &self.denom_cut &&
                                !self.known_expr.contains_key(&num) {
                            let lexpr = self.known_expr.get(lval).unwrap();
                            let rexpr = self.known_expr.get(rval).unwrap();
                            let expr = if op == &Operator::Mul {
                                format!("{}{}{}", lexpr, op, rexpr)
                            } else {
                                format!("({}{}{})", lexpr, op, rexpr)
                            };

                            self.known_expr.insert(num, expr);
                            next_generatable.push(num);
                        }
                    }
                }
            }

            next_generatable.sort_by_key(|num| *num.denom());
            self.generatable_nums.push(next_generatable);
        }
    }
}

fn generate_pair_expr(num1: Rational64, num2: Rational64) -> HashMap<Rational64, String> {
    OPERATORS
        .iter()
        .filter_map(move |op| {
            op.invoke(num1, num2)
                .map(|res| {
                    let expr = format!("({}{}{})", num1, op, num2);
                    (res, expr)
                })
        })
        .collect()
}

fn generate_triple_expr(num1: Rational64, num2: Rational64, num3: Rational64) -> HashMap<Rational64, String> {
    // ((num1 op1 num2) op2 num3)
    let invoke_left = move |op1: Operator, op2: Operator| -> Option<(Rational64, String)> {
        let tmp = op1.invoke(num1, num2)?;
        op2.invoke(tmp, num3)
            .map(move |num| {
                let expr = if (op1 == Operator::Mul || op1 == Operator::Div) ||
                        ((op1 == Operator::Add || op1 == Operator::Sub) &&
                        (op2 == Operator::Add || op2 == Operator::Sub)) {
                    format!("({}{}{}{}{})", num1, op1, num2, op2, num3)
                } else {
                    format!("(({}{}{}){}{})", num1, op1, num2, op2, num3)
                };
                (num, expr)
            })
    };

    // (num1 op1 (num2 op2 num3))
    let invoke_right = move |op1: Operator, op2: Operator| -> Option<(Rational64, String)> {
        let tmp = op2.invoke(num2, num3)?;
        op1.invoke(num1, tmp)
            .map(move |num| {
                let expr = if op1 == Operator::Add ||
                        (op1 != Operator::Div && (op2 == Operator::Mul || op2 == Operator::Div)) {
                    format!("({}{}{}{}{})", num1, op1, num2, op2, num3)
                } else {
                    format!("({}{}({}{}{}))", num1, op1, num2, op2, num3)
                };
                (num, expr)
            })
    };

    iproduct!(&OPERATORS, &OPERATORS)
        .fold(HashMap::default(), |mut exprs, (op1, op2)| {
            if let Some((num, expr)) = invoke_left(*op1, *op2) {
                exprs.insert(num, expr);
            }

            if let Some((num, expr)) = invoke_right(*op1, *op2) {
                exprs.insert(num, expr);
            }
            exprs
        })
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum MulExpr {
    Mul(i64),
    MulAdd(i64, i64),
    MulSub(i64, i64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExpressionGenerator {
    search_ordering: Vec<MulExpr>,
    known_expr: HashMap<i64, String>,
}

impl ExpressionGenerator {
    pub fn from_lists(num_lists: &Vec<Vec<i64>>, search_depth: usize, denom_cut: i64) -> Self {
        // convert value type from i64 into Rational64
        let rat_num_lists = num_lists
            .iter()
            .map(|num_list| {
                num_list.iter()
                    .map(|&n| n.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut rat_search = RationalSearch::from_lists(&rat_num_lists, denom_cut);
        rat_search.extend(search_depth);

        let gen_nums: Vec<Vec<_>> = rat_search.generatable_nums
            .into_iter()
            .map(|nums| {
                nums
                    .into_iter()
                    .filter_map(|num| {
                        if num.is_integer() {
                            Some(num.to_integer())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();

        let mut mul_list = all_mul_offset_with_score(&gen_nums);

        // sort mul_list in decending order of score
        mul_list.sort_by(|(lscore, _), (rscore, _)| rscore.partial_cmp(lscore).unwrap());
        let mut search_ordering: Vec<_> = mul_list
            .into_iter()
            .map(|(_, expr)| expr)
            .collect();

        let uni_max = gen_nums[0].iter().max().unwrap();
        shrink_ordering(&mut search_ordering, *uni_max);

        // convert value type from Rational64 into i64
        let known_expr: HashMap<_, _> = rat_search.known_expr
            .into_iter()
            .filter_map(|(num, expr)| {
                if num.is_integer() {
                    Some((num.to_integer(), expr))
                } else {
                    None
                }
            })
            .collect();

        Self {
            search_ordering,
            known_expr
        }
    }

    pub fn new_nananiji(search_depth: usize, denom_cut: i64) -> Self {
        Self::from_lists(&vec![
            vec![227],
            vec![22, 7],
            vec![2, 2, 7],
        ], search_depth, denom_cut)
    }

    pub fn new_hanshin(allow_3_34: bool, search_depth: usize, denom_cut: i64) -> Self {
        let mut num_list = vec![
            vec![334],
            vec![33, 4],
            vec![3, 3, 4]
        ];

        if allow_3_34 {
            num_list.push(vec![3, 34]);
        }

        Self::from_lists(&num_list, search_depth, denom_cut)
    }

    pub fn new_kyojin(allow_2_64: bool, search_depth: usize, denom_cut: i64) -> Self {
        let mut num_list = vec![
            vec![264],
            vec![26, 4],
            vec![2, 6, 4]
        ];

        if allow_2_64 {
            num_list.push(vec![2, 64]);
        }

        Self::from_lists(&num_list, search_depth, denom_cut)
    }

    pub fn generate(&self, n: i64) -> String {
        if let Some(expr) = self.known_expr.get(&n) {
            return expr.to_owned();
        } else {
            for expr in &self.search_ordering {
                match expr {
                    &MulExpr::Mul(mul) if n % mul == 0 => {
                        return format!("{}*{}",
                            self.generate(n / mul),
                            self.known_expr.get(&mul).unwrap());
                    },
                    &MulExpr::MulAdd(mul, add) if (n - add) % mul == 0 => {
                        if ((n - add) / mul).abs() == 1 {
                            return format!("({}+{})",
                                self.generate(n - add),
                                self.known_expr.get(&add).unwrap(),
                            );
                        } else {
                            return format!("({}*{}+{})",
                                self.generate((n - add) / mul),
                                self.known_expr.get(&mul).unwrap(),
                                self.known_expr.get(&add).unwrap(),
                            );
                        }
                    },
                    &MulExpr::MulSub(mul, sub) if (n + sub) % mul == 0 => {
                        if ((n + sub) / mul).abs() == 1 {
                            return format!("({}-{})",
                                self.generate(n + sub),
                                self.known_expr.get(&sub).unwrap(),
                            );
                        } else {
                            return format!("({}*{}-{})",
                                self.generate((n + sub) / mul),
                                self.known_expr.get(&mul).unwrap(),
                                self.known_expr.get(&sub).unwrap(),
                            );
                        }
                    },
                    _ => {}
                }
            }
            unimplemented!()
        }
    }
}

fn all_mul_offset_with_score(gen_nums: &Vec<Vec<i64>>) -> Vec<(f64, MulExpr)> {
    let mut mul_set = HashSet::default();

    let mut ret = Vec::new();
    for (i, muls) in gen_nums.iter().enumerate() {
        for mul in muls {
            if mul_set.contains(&-mul) {
                continue;
            }
            mul_set.insert(mul);

            let mulabs = mul.abs();
            let mut rem_set = HashSet::default();
            let score = (mulabs as f64).powf(1.0 / ((i + 1) as f64));
            if score > 2.0 && mul != &0 {
                ret.push((score, MulExpr::Mul(*mul)));
            }
            rem_set.insert(0);

            'offs_loop: for (j, offsets) in gen_nums.iter().enumerate() {
                if muls.len() * offsets.len() > 2_000_000 {
                    break;
                }

                for offset in offsets {
                    let score = (mulabs as f64).powf(1.0 / ((i + j + 2) as f64));
                    if score > 2.0 && mul != &0 {
                        if rem_set.insert((offset % mulabs + mulabs) % mulabs) {
                            ret.push((score, MulExpr::MulSub(*mul, *offset)));
                        }

                        if rem_set.insert((-offset % mulabs + mulabs) % mulabs) {
                            ret.push((score, MulExpr::MulAdd(*mul, *offset)));
                        }

                        if rem_set.len() == *mul as usize {
                            break 'offs_loop;
                        }
                    }
                }
            }
        }
    }

    ret
}

fn shrink_ordering(search_ordering: &mut Vec<MulExpr>, div: i64) {
    let mut rem_map = HashMap::default();
    for (idx, expr) in search_ordering.iter().enumerate() {
        match *expr {
            MulExpr::Mul(mul) if mul == div => {
                if !rem_map.contains_key(&0) {
                    rem_map.insert(0, idx);
                    if rem_map.len() == div as usize {
                        break;
                    }
                }
            },
            MulExpr::MulAdd(mul, ofs) if mul == div => {
                let rem = ((ofs % mul) + mul) % mul;
                if !rem_map.contains_key(&rem) {
                    rem_map.insert(rem, idx);
                    if rem_map.len() == div as usize {
                        break;
                    }
                }
            },
            MulExpr::MulSub(mul, ofs) if mul == div => {
                let rem = ((-ofs % mul) + mul) % mul;
                if !rem_map.contains_key(&rem) {
                    rem_map.insert(rem, idx);
                    if rem_map.len() == div as usize {
                        break;
                    }
                }
            },
            _ => {}
        }
    }

    if rem_map.len() == div as usize {
        let max_idx = rem_map.values()
            .max()
            .unwrap();
        search_ordering.truncate(max_idx+1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_pair_expr() {
        let mut ans = HashMap::default();

        ans.insert(37.into(), "(33+4)".to_string());
        ans.insert(29.into(), "(33-4)".to_string());
        ans.insert(132.into(), "(33*4)".to_string());
        ans.insert(Rational64::new(33, 4), "(33/4)".to_string());

        assert_eq!(
            generate_pair_expr(33.into(), 4.into()),
            ans
        );
    }

    #[test]
    fn test_generate_triple_expr() {
        let ans: HashSet<_> = vec![
            4.into(),              // (1+3)+0
            (-2).into(),           // (1-3)+0
            3.into(),              // (1*3)+0
            Rational64::new(1, 3), // (1*3)+0
            0.into(),              // (1+3)*0
            1.into(),              // 1+(3*0)
        ]
            .into_iter()
            .collect();

        let result: HashSet<_> = generate_triple_expr(1.into(), 3.into(), 0.into())
            .keys()
            .map(|k| *k)
            .collect();

        assert_eq!(result, ans);
    }

    #[test]
    fn from_lists() {
        let exprs = RationalSearch::from_lists(
            &vec![
                vec![334.into()],
                vec![33.into(), 4.into()],
                vec![1.into(), 3.into(), 0.into()]
            ],
            30);

        let ans: HashSet<_> = vec![
            4.into(),              // (1+3)+0
            (-2).into(),           // (1-3)+0
            3.into(),              // (1*3)+0
            Rational64::new(1, 3), // (1*3)+0
            0.into(),              // (1+3)*0
            1.into(),              // 1+(3*0)
            37.into(),             // 33+4
            29.into(),             // 33-4
            132.into(),            // 33*4
            Rational64::new(33, 4),// 33/4
            334.into(),
        ]
            .into_iter()
            .collect();

        let result: HashSet<_> = exprs.generatable_nums[0]
            .iter()
            .map(|num| *num)
            .collect();

        assert_eq!(result, ans);
        assert_eq!(result, exprs.known_expr.keys().map(|num| *num).collect());
    }

    #[test]
    fn extend() {
        let mut exprs = RationalSearch::from_lists(
            &vec![vec![334.into()]],
            30);

        exprs.extend(2);

        let mut ans_vec: HashSet<Rational64> = HashSet::new();
        ans_vec.insert(0.into());       // (334-334)
        ans_vec.insert(668.into());     // (334+334)
        ans_vec.insert(111556.into());  // (334*334)
        ans_vec.insert(1.into());       // (334/334)

        assert_eq!(ans_vec, exprs.generatable_nums[1].clone().into_iter().collect::<HashSet<_>>());

        let ans_set: HashSet<_> = vec![
            334.into(),
            0.into(),
            668.into(),
            111556.into(),
            1.into()
        ]
            .into_iter()
            .collect();
        assert_eq!(ans_set, exprs.known_expr.keys().map(|num| *num).collect());
    }
}
