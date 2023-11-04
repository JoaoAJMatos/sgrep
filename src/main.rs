//
// Created by JoaoAJMatos on 04/11/2023
//

use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::fs;

use regex::Regex;
use clap::{App, Arg};

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};


// Constants
const INITIAL_PROMPT_CONTEXT : &str = "
    I am going to give you a text in natural language. 
    Your job is to write a regular expression for Rust Regex that matches the search patterns described by the text.

    You must return the output in the format: <regex> NOTHING else.
    You MUST NOT, under any circumstances, return an output that does not match the format above. 
    You also MUST NOT print anything besides the output, like an introduction to the response or any explanation. 
    For everything I say you MUST respond with the regex pattern only without any type of markdown formatting.
    Just plain text.
";


fn main() {
    let matches = App::new("sgrep")
        .version("0.5.0")
        .author("JoaoAJMatos")
        .about("AI for grep. Convert natural language to regex.")
        .arg(Arg::with_name("pattern")
            .help("The pattern to search for (description in natural language)")
            .required(false)
            .index(1))
        .arg(Arg::with_name("auth")
            .short("a")
            .long("auth")
            .value_name("AUTH_TOKEN")
            .help("Sets the OpenAI API authentication token"))
        .get_matches();

    store_auth_key_if_auth_flag_is_present(&matches);

    // Ensure the user is authenticated
    if !user_is_authenticated() {
        println!("You need to authenticate first. Run 'sgrep --auth AUTH_TOKEN'");
        std::process::exit(1);
    }

    let auth_token = get_auth_token();

    // Read the input from stdin (piped input)
    let stdin = io::stdin();
    let input = BufReader::new(stdin);

    // Extract the pattern from the command line 
    // arguments and build a regex from it
    let pattern = matches.value_of("pattern").unwrap();
    let result = build_regex_from_pattern(pattern, &auth_token);

    let _regex = match result {
        Ok(regex) => regex,
        Err(e) => {
            println!("Failed to build regex from pattern: {}", e);
            std::process::exit(1);
        }
    };

    // Search for the pattern in the input and 
    // print the lines that match
    for line in input.lines() {
        let line = line.unwrap();
        if _regex.is_match(&line) {
            println!("{}", line);
        }
    }
}


// Checks if the AUTH flag is present and stores the auth token
// in the appropriate file
fn store_auth_key_if_auth_flag_is_present(matches: &clap::ArgMatches) {
    if matches.is_present("auth") {
        if let Some(auth_token) = matches.value_of("auth") {
            store_auth_token(auth_token);
            println!("Auth token stored successfully");
            std::process::exit(0);
        } else {
            println!("--auth flag requires an argument: 'AUTH_TOKEN'");
            std::process::exit(1);
        }
    }   
}


// Stores the auth token in a file
fn store_auth_token(auth_token: &str) {
    let path = get_auth_token_file();
    fs::create_dir_all(path.parent().unwrap()).expect("Failed to create directory");
    fs::write(path, auth_token).expect("Failed to write auth token to file");
}


// Returns the path to the file where the auth token is stored
// depending on the OS
fn get_auth_token_file() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    let path = Path::new(&std::env::var("APPDATA").unwrap()).join("sgrep").join("auth_token");

    #[cfg(target_os = "linux")]
    let path = Path::new(&std::env::var("HOME").unwrap()).join(".sgrep").join("auth_token");

    #[cfg(target_os = "macos")]
    let path = Path::new(&std::env::var("HOME").unwrap()).join(".sgrep").join("auth_token");

    path
}


// Checks if the user is authenticated
fn user_is_authenticated() -> bool {
    let path = get_auth_token_file();
    path.exists()
} 


// Returns the auth token stored in the appropriate file
fn get_auth_token() -> String {
    let path = get_auth_token_file();
    fs::read_to_string(path).expect("Failed to read auth token from file")
}


// Fetches the OpenAI API for converting 
// natural language to regex 
fn build_regex_from_pattern(pattern: &str, auth_token: &str) -> Result<Regex, Box<dyn std::error::Error>> {
    let client = Client::new(auth_token.to_string());

    let request = ChatCompletionRequest::new(
        chat_completion::GPT3_5_TURBO.to_string(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: INITIAL_PROMPT_CONTEXT.to_string(),
                name: None,
                function_call: None,
            }, chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: pattern.to_string(),
                name: None,
                function_call: None,
            }],
    );

    let response = client.chat_completion(request);

    if let Ok(response) = response {
        let content = response.choices.get(0).and_then(|choice| choice.message.content.as_deref());
        if let Some(content_string) = content {
            let regex = Regex::new(content_string)?;
            return Ok(regex);
        }
    }

    Err("Failed to build regex from pattern".into())
}