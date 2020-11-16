use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
    let mut rl = Editor::<()>::new();
    let history_file = ".eldiro_history";
    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    let mut env = eldiro::Env::default();

    loop {
        let readline = rl.readline("→ ");
        match readline {
            Ok(line) => {
                let line = line.as_str().trim();
                rl.add_history_entry(line);
                match run(line, &mut env) {
                    Ok(Some(val)) => println!("{}", val),
                    Ok(None) => {}
                    Err(msg) => println!("{}", msg),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history(history_file).unwrap();
}



fn run(input: &str, env: &mut eldiro::Env) -> Result<Option<eldiro::Val>, String> {
    let parse = eldiro::parse(input)
        .map_err(|msg| format!("Parse error: {}", msg))?;

    let evaluated = parse
        .eval(env)
        .map_err(|msg| format!("Evaluation error: {}", msg))?;

    if evaluated == eldiro::Val::Unit {
        Ok(None)
    } else {
        Ok(Some(evaluated))
    }
}
