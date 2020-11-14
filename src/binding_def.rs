use crate::env::Env;
use crate::expr::Expr;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub(crate) struct BindingDef {
    pub(crate) name: String,
    pub(crate) val: Expr,
}

impl BindingDef {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        let s = utils::tag("let", s)?;
        let (s, _) = utils::extract_whitespace1(s)?;

        let (s, name) = utils::extract_ident(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag("=", s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, val) = Expr::new(s)?;

        Ok((s, Self { name: name.to_string(), val }))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let val = self.val.eval(env)?;
        env.store_binding(self.name.clone(), val);
        Ok(Val::Unit)
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::{Number, Op};

    use super::*;

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            BindingDef::new("let a = 10 / 2"),
            Ok((
                "",
                BindingDef {
                    name: "a".to_string(),
                    val: Expr::Operation {
                        lhs: Number(10),
                        rhs: Number(2),
                        op: Op::Div,
                    },
                },
            )),
        );
    }

    #[test]
    fn parse_more_binding_def() {
        assert_eq!(
            BindingDef::new("let a123a=121"),
            Ok((
                "",
                BindingDef {
                    name: "a123a".to_string(),
                    val: Expr::Number(Number(121)),
                },
            )),
        );
    }

    #[test]
    fn cannot_parse_binding_def_without_space_after_let() {
        assert_eq!(
            BindingDef::new("letaaa=1+2"),
            Err("expected a space".to_string()),
        );
    }
}
