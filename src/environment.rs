use std::collections::HashMap;

use crate::{loxresult::LoxResult, token::Token, value::Value};

///Environment 是一个作用域中定义的变量的集合
///
///{ ----------------\
///                  \
///                  \
///    { ------      \     
///           \      \
///           \      \
///           \      \
///           \      \
///           \      \
/// enclosing \      \  values
///           \      \
///           \      \
///           \      \
///           \      \
///    { -----\      \
///                  \
///                  \
///                  \
///} ----------------\
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Environment {
    ///这里存放了全局变量
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment::default()
    }

    ///定义变量
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    ///产生子环境
    pub fn new_enclosing(enclosing: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn get_enclosing_env(&mut self) -> Option<Box<Self>> {
        self.enclosing.clone()
    }

    // remember to handle none
    ///这是[`Environment`]中的核心函数
    ///在当前的环境中搜索变量，如果没有找到，那么就向其父环境寻找,由此反复
    ///如果到global仍没有找到，那么就抛出异常
    pub fn get(&self, name: Token) -> Result<Value, LoxResult> {
        if let Some(v) = self.values.get(&name.lexeme) {
            return Ok(v.clone());
        } else if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }
        // BUG: error occur when calling function
        Err(LoxResult::RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", &name.lexeme),
        })
    }

    ///赋值语句
    pub fn assign(&mut self, name: Token, value: Value) -> Result<(), LoxResult> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)?;
            return Ok(());
        }

        Err(LoxResult::RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", &name.lexeme),
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new_enclosing() {
        let mut env = Environment::new();
        env.define("a".to_string(), Value::Number(1.0));
        let mut child_env = Environment::new_enclosing(env.clone());
        assert_eq!(
            child_env,
            Environment {
                values: HashMap::new(),
                enclosing: Some(Box::new(env))
            }
        );
    }

    #[test]
    fn test_new() {
        assert_eq!(
            Environment {
                values: HashMap::new(),
                enclosing: None
            },
            Environment::new()
        )
    }
}
