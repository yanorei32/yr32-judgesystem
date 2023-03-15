use serde::{Deserialize, Serialize};
use rust_embed::RustEmbed;

#[derive(Deserialize, Serialize)]
pub struct Practice {
    pub id: String,
    pub title: String,
    pub description: String,
    pub header: String,
    pub footer: String,
    pub timeout_ms: u64,
    pub testcases: Vec<Case>,
    #[serde(default = "default_answers")]
    pub answers: Vec<Answer>,
}

const fn default_answers() -> Vec<Answer> {
    Vec::new()
}

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Assets;

#[derive(Deserialize, Serialize)]
pub struct Answer {
    pub code: String,
    pub note: String,
}

#[derive(Deserialize)]
pub struct Environments {
    #[serde(default = "default_practices_path")]
    pub practices: String,
    #[serde(default = "default_users_path")]
    pub users: String,
    #[serde(default = "default_user_id")]
    pub userid: String,
    #[serde(default = "default_user_code_path")]
    pub user_code_path: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_practices_path() -> String {
    "/etc/practices.json".to_string()
}

fn default_users_path() -> String {
    "/etc/users.json".to_string()
}

const fn default_port() -> u16 {
    8080
}

fn default_user_id() -> String {
    "#1000".to_string()
}

fn default_user_code_path() -> String {
    "/tmp".to_string()
}

#[derive(Deserialize, Serialize)]
pub struct Case {
    pub note: String,
    pub input: String,
    pub output: String,
}

#[derive(Serialize)]
pub struct Empty {}

#[derive(Deserialize)]
pub struct JudgeRequest {
    pub code: String,
    pub id: String,
}

#[derive(Deserialize)]
pub struct User {
    pub name: String,
    pub password_hash: String,
}

#[derive(Serialize)]
pub enum JudgeResult {
    Ok,
    Timeout,
    Invalid,
    SystemError,
    IllegalProcessExit,
}
