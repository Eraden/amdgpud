#[derive(Debug, gumdrop::Options)]
pub enum VoltageCommand {
    Placeholder(Placeholder),
}

#[derive(Debug, gumdrop::Options)]
pub struct Placeholder {
    help: bool,
}
