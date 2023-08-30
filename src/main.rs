use clap::{arg, Command};

use lexer::TokenType;
mod lexer;


fn cli() -> Command {
    Command::new("Danfe")
        .about("A programming langauge made by Sairash")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("file")
                .about("Use a df file")
                .arg(arg!(<REMOTE> "The remote to open a file"))
                .arg_required_else_help(true),
        )
}

fn main() -> std::io::Result<()> {

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("file", sub_matches)) => {
            let file_name = sub_matches.get_one::<String>("REMOTE").expect("required");
            let text = std::fs::read_to_string(file_name)?;
            let mut lex  = lexer::Lexer::new(&text);

            loop {
                match lex.next_token() {
                    Ok(TokenType::EOF)=> break,
                    Ok(tok) => println!("{0:?}", tok),
                    Err(err) => println!("{0:?}", err)
                }
            }
        }
        _ => {}
    }
    Ok(())
}