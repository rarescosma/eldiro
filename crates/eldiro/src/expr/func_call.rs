use crate::utils;

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FuncCall {
    pub(crate) callee: String,
    pub(crate) params: Vec<Expr>,
}

impl FuncCall {
    pub(super) fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, callee) = utils::extract_ident(s)?;
        let (s, _) = utils::extract_non_breaks(s);

        let (s, param) = Expr::new(s)?;
        let (s, _) = utils::extract_non_breaks(s);
        let (s, more_params) = utils::sequence(
            Expr::new,
            s,
            Some(Box::new(utils::extract_non_breaks))
        )?;

        let mut params: Vec<Expr> = vec![param];
        params.extend(more_params);

        Ok((s, Self { callee: callee.to_string(), params }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Number;

    #[test]
    fn parse_func_call_with_one_parameter() {
        assert_eq!(
            FuncCall::new("factorial 10"),
            Ok((
                "",
                FuncCall {
                    callee: "factorial".to_string(),
                    params: vec![Expr::Number(Number(10))],
                },
            )),
        );
    }
}
