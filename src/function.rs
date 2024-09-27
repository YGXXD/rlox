use crate::chunk::*;
use std::rc::Rc;

pub struct Function {
    pub name: String,
    pub params_num: usize,
    pub chunk: Rc<Chunk>,
}

impl ToString for Function {
    fn to_string(&self) -> String {
        match self.name.len() {
            0 => "<script>".to_string(),
            _ => format!("<fn {}>", self.name),
        }
    }
}

impl Function {
    pub fn disassemble(&self) {
        self.chunk.disassemble(&self.to_string());
    }
}
