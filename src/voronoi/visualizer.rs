use std::collections::HashMap;

#[derive(Default)]
pub struct Visualizer {
    statement_names: HashMap<String, String>,
    statements: Vec<String>,
}

impl Visualizer {
    fn get_new_statement_name(&mut self) -> String {
        format!("A_{}", self.statement_names.len())
    }

    pub fn add_statement(&mut self, statement: String) -> String {
        let new_name = self.get_new_statement_name();
        if !self.statement_names.contains_key(&statement) {
            self.statement_names.insert(statement.clone(), new_name);
            self.statements.push(statement.clone());
        }
        self.dump();
        self.statement_names[&statement].clone()
    }

    fn dump(&self) {
        // The second list is to make sure we iterate in the correct order. Hacky but who cares
        let statements: Vec<_> = self
            .statements
            .iter()
            .map(|statement| format!("\"{} = {}\"", &self.statement_names[statement], &statement))
            .collect();
        println!("Execute({{ {} }})", statements.join(", "));
    }
}
