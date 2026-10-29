//! Hard-coded synthetic report example. No filesystem or process collection.
#[path = "support/report_fixture.rs"]
mod synthetic;
use anti_ai_guard::report;
fn main() {
    let public = report::project(&synthetic::fixture());
    if std::env::args().any(|a| a == "--json") {
        print!("{}", public.json());
    } else {
        print!("{}", public.text());
    }
}
