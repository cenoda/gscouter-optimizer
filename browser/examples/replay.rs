//! Native execution of the browser state machine for cross-target replay checks.
use gso_browser::{Input, Session};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: replay INPUT.json")?;
    let input: Input = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut session = Session::start(input)?;
    loop {
        let report = session.step()?;
        if report["complete"] == true {
            println!("{}", serde_json::to_string(&report)?);
            return Ok(());
        }
    }
}
