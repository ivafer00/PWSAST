use std::fs::File;
use std::io::Write;
use std::process::{Command, Output};
use std::ptr::null;
use askama::Template;
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use clap::builder::Str;
use log::info;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::controllers::html::html::{error404, Error404Template, HtmlTemplate, HTTPResponse};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/upload", post(upload_script))
}

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    info!("Request to /");

    let version = state.config.app.name;

    let template = IndexTemplate { app_name: version };

    HtmlTemplate(template)
}

pub fn is_valid_extension(filename: &str) -> bool {
    let valid_extensions = ["txt", "ps1"]; // Agrega las extensiones válidas aquí
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    valid_extensions.contains(&extension.as_str())
}

fn generate_random_string(length: usize) -> String {
    let rng = rand::thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    random_string
}

pub fn execute_powershell_analysis(command: String) -> Result<Output, std::io::Error> {
    Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
}

#[derive(Serialize, Deserialize)]
pub struct Finding {
    pub rule_name: String,
    pub severity: String,
    pub line: String,
    pub message: String
}

pub fn parse_output(output: String) -> String {
    let lines: Vec<&str> = output.split_terminator('\n').collect();

    let mut findings: Vec<Finding> = Vec::new();

    for line in lines{
        println!("{}", line);
        let parsed_line = line.replace('\r',"");
        let parsed_line_vec: Vec<&str> = parsed_line.split('\t').collect();
        let finding = Finding{
            rule_name : parsed_line_vec[0].parse().unwrap(),
            severity: parsed_line_vec[1].parse().unwrap(),
            line: parsed_line_vec[2].parse().unwrap(),
            message: parsed_line_vec[3].parse().unwrap(),
        };
        findings.push(finding);
    }

    serde_json::to_string(&findings).expect("Failed to serialize")
}

pub async fn upload_script(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    info!("Request to /upload");

    let version = state.clone().config.app.name;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );

        if(is_valid_extension(file_name.as_str())){
            println!("Valid extension");

            let random_filename = generate_random_string(20) + ".ps1";

            {
                let mut file = match File::create(&random_filename) {
                    Ok(f) => f,
                    Err(_) => {
                        return HTTPResponse::NOTFOUND404(Error404Template {
                            app_name: version.clone(),
                        })
                    }
                };

                if let Err(_) = file.write_all(data.as_ref()) {
                    return HTTPResponse::NOTFOUND404(Error404Template {
                        app_name: version.clone(),
                    })
                }
            }


            let command = format!("$a=Invoke-ScriptAnalyzer {};foreach($b in $a){{$b.RuleName+\"`t\"+$b.Severity+\"`t\"+$b.Line+\"`t\"+$b.Message}}", random_filename);
            let output_string = match execute_powershell_analysis(command){
                    Ok(output) => {
                        if output.status.success() {
                            String::from_utf8_lossy(&output.stdout).to_string()
                        } else {
                            return HTTPResponse::NOTFOUND404(Error404Template {
                                app_name: version.clone(),
                            })
                        }
                    },
                    Err(_) => {
                        return HTTPResponse::NOTFOUND404(Error404Template {
                            app_name: version.clone(),
                        })
                    }
            };

            let template = UploadTemplate { app_name: version, script: parse_output(output_string) };
            return HTTPResponse::OK200(template);

        } else {
            println!("Invalid extension");
        }

    }

    HTTPResponse::NOTFOUND404(Error404Template {app_name: version})
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    app_name: String,
}

#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate {
    app_name: String,
    script: String
}