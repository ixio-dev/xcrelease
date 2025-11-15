use anyhow::Result;

pub fn handle_template() -> Result<()> {
    print_deployment_template();
    Ok(())
}

fn print_deployment_template() {
    println!("{}", include_str!("../../template.deployment"));
}