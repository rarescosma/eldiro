use std::collections::HashMap;

use crate::val::Val;
use crate::stmt::Stmt;

#[derive(Debug, PartialEq, Clone)]
enum NamedInfo {
    Binding(Val),
    Func { params: Vec<String>, body: Stmt },
}

#[derive(Debug, PartialEq, Default)]
pub struct Env<'parent> {
    bindings: HashMap<String, NamedInfo>,
    parent: Option<&'parent Self>,
}

impl<'parent> Env<'parent> {
    pub(crate) fn create_child(&'parent self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(self),
        }
    }

    pub(crate) fn store_binding(&mut self, name: String, val: Val) {
        self.bindings.insert(name, NamedInfo::Binding(val));
    }

    pub(crate) fn get_binding(&self, name: &str) -> Result<Val, String> {
        self.chain_lookup(name)
            .and_then(|named_info| match named_info {
                NamedInfo::Binding(val) => Some(val),
                _ => None,
            })
            .ok_or_else(|| format!(
                "binding with name '{}' does not exist",
                name,
            ))
    }

    fn chain_lookup(&self, name: &str) -> Option<NamedInfo> {
        self.bindings.get(name).cloned().or_else(|| {
            self.parent.and_then(|parent| parent.chain_lookup(name))
        })
    }
}


