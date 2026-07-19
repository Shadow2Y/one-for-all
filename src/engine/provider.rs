pub fn resolve_variable(registry: &'static CommandRegistry, variable: &Variable) -> Result<Value> {
    match variable {
        Variable::Literal(v) => Ok(v.clone()),
        Variable::Provided(p) => resolve_provider(registry, p),
    }
}

pub fn resolve_provider(registry: &'static CommandRegistry, provider: &Provider) -> Result<Value> {
    execute_command(registry, &provider.run, &[])
}
