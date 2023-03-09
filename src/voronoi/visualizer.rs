#[derive(Default)]
pub struct Visualizer {
    statements: Vec<String>,
}

impl Visualizer {
    fn next_name(&mut self) -> String {
        format!("A_{}", self.statements.len())
    }

    pub fn add_statement(&mut self, statement: String) -> String {
        let next_name = self.next_name();
        self.statements
            .push(format!("{} = {}", next_name, statement));
        self.dump();
        next_name
    }

    fn dump(&self) {
        let statements: Vec<_> = self
            .statements
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect();
        println!("Execute({{ {} }})", statements.join(", "));
    }
}
