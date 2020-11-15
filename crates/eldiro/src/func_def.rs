use crate::stmt::Stmt;
use crate::{utils, Env, Val};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FuncDef {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) body: Box<Stmt>,
}

impl FuncDef {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        let s = utils::tag("fn", s)?;
        let (s, _) = utils::extract_whitespace1(s)?;

        let (s, name) = utils::extract_ident(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, params) = utils::sequence(
            |s| utils::extract_ident(s).map(|(s, ident)| (s, ident.to_string())),
            s,
            None,
        )?;

        let s = utils::tag("=>", s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, body) = Stmt::new(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
                params,
                body: Box::new(body),
            }
        ))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        env.store_func(
            self.name.clone(),
            self.params.clone(),
            *self.body.clone(),
        );
        Ok(Val::Unit)
    }
}
