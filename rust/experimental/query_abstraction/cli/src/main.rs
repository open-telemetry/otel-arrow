use intermediate_language::query_processor::QueryProcessor;
use kql_plugin::kql_plugin::KqlPlugin;
use ottl_plugin::ottl_plugin::OttlPlugin;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <plugin> <input>", args[0]);
        eprintln!("Parser options: kql, ottl");
        std::process::exit(1);
    }

    let plugin = &args[1];
    let input = &args[2];

    match plugin.as_str() {
        "kql" => {
            let result = KqlPlugin::process_query(input);
            match result {
                Ok(query) => {
                    println!("Parsed KQL query:\n{query:?}");
                }
                Err(e) => {
                    eprintln!("Error parsing KQL query:\n{e}");
                    std::process::exit(1);
                }
            }
        }
        "ottl" => {
            let result = OttlPlugin::process_query(input);
            match result {
                Ok(query) => {
                    println!("Parsed OTTL query: {query:?}");
                }
                Err(e) => {
                    eprintln!("Error parsing OTTL query: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Invalid plugin option. Use 'kql' or 'ottl'.");
            std::process::exit(1);
        }
    }
}
