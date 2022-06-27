#![warn(clippy::pedantic, clippy::nursery)]

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use actix_embed::Embed;
use actix_web::dev::ServiceRequest;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::basic::{BasicAuth, Config as AuthConfig};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
use bcrypt::verify;
use once_cell::sync::OnceCell;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ulid::Ulid;
use wait_timeout::ChildExt;

static PRACTICES: OnceCell<Vec<Practice>> = OnceCell::new();
static ENVIRONMENTS: OnceCell<Environments> = OnceCell::new();
static USERS: OnceCell<Vec<User>> = OnceCell::new();
static JUDGE_MUTEX: OnceCell<Arc<Mutex<i32>>> = OnceCell::new();

#[derive(Deserialize, Serialize)]
struct Practice {
    id: String,
    title: String,
    description: String,
    header: String,
    footer: String,
    timeout_ms: u64,
    testcases: Vec<Case>,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Deserialize)]
struct Environments {
    #[serde(default = "default_practices_path")]
    practices: String,
    #[serde(default = "default_users_path")]
    users: String,
    #[serde(default = "default_user_id")]
    userid: String,
    #[serde(default = "default_user_code_path")]
    user_code_path: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_practices_path() -> String {
    "/etc/practices.json".to_string()
}

fn default_users_path() -> String {
    "/etc/users.json".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_user_id() -> String {
    "#1000".to_string()
}

fn default_user_code_path() -> String {
    "/tmp".to_string()
}

#[derive(Deserialize, Serialize)]
struct Case {
    note: String,
    input: String,
    output: String,
}

#[derive(Serialize)]
struct Empty {}

#[derive(Deserialize)]
struct JudgeRequest {
    code: String,
    id: String,
}

#[derive(Deserialize)]
struct User {
    name: String,
    password_hash: String,
}

#[derive(Serialize)]
enum JudgeResult {
    Ok,
    Timeout,
    Invalid,
    SystemError,
    IllegalProcessExit,
}

#[allow(clippy::unused_async)]
#[get("/api/list")]
async fn list() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(PRACTICES.get()))
}

#[allow(clippy::unused_async)]
#[post("/api/judge")]
async fn judge(req: web::Json<JudgeRequest>) -> actix_web::Result<HttpResponse> {
    let e = ENVIRONMENTS.get().unwrap();

    let p = match PRACTICES.get().unwrap().iter().find(|p| p.id == req.id) {
        Some(p) => p,
        None => return Ok(HttpResponse::NotFound().json(Empty {})),
    };

    let uuid = Ulid::new().to_string();
    let path = Path::new(&e.user_code_path).join(format!("{}_{}.py", p.id, uuid));

    let mut file = match File::create(&path) {
        Ok(f) => f,
        Err(_) => return Ok(HttpResponse::InternalServerError().json(Empty {})),
    };

    if file
        .write_all(format!("{}\n{}\n{}\n", p.header, req.code, p.footer).as_bytes())
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(Empty {}));
    }

    let path = path.into_os_string();

    JUDGE_MUTEX.get().unwrap().lock().await;

    return Ok(HttpResponse::Ok().json(
        p.testcases
            .iter()
            .map(|c| {
                let proc = Command::new("sudo")
                    .arg("-u")
                    .arg(&ENVIRONMENTS.get().unwrap().userid)
                    .arg("python3")
                    .arg(&path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn();

                let mut proc = match proc {
                    Ok(v) => v,
                    Err(_) => return JudgeResult::SystemError,
                };

                if proc
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(c.input.as_bytes())
                    .is_err()
                {
                    return JudgeResult::IllegalProcessExit;
                }

                let secs = Duration::from_millis(p.timeout_ms);
                let _status_code = match proc.wait_timeout(secs).unwrap() {
                    Some(status) => status.code(),
                    None => {
                        proc.kill().unwrap();
                        let _ = proc.wait().unwrap().code();
                        return JudgeResult::Timeout;
                    }
                };

                let mut output = String::new();

                if proc
                    .stdout
                    .as_mut()
                    .unwrap()
                    .read_to_string(&mut output)
                    .is_err()
                {
                    return JudgeResult::IllegalProcessExit;
                }

                if output == c.output {
                    JudgeResult::Ok
                } else {
                    JudgeResult::Invalid
                }
            })
            .collect::<Vec<JudgeResult>>(),
    ));
}

#[allow(clippy::unused_async)]
async fn validator(req: ServiceRequest, cred: BasicAuth) -> Result<ServiceRequest, Error> {
    let id = cred.user_id().to_string();
    let password = cred.password().map_or("", |p| p).to_string();
    let config = req.app_data::<AuthConfig>().cloned().unwrap_or_default();

    match USERS
        .get()
        .unwrap()
        .iter()
        .find(|u| id == u.name && verify(&password, &u.password_hash).unwrap())
    {
        Some(_) => Ok(req),
        None => Err(AuthenticationError::from(config).into()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ENVIRONMENTS.set(envy::from_env().unwrap()).unwrap_or(());
    let e = ENVIRONMENTS.get().unwrap();

    JUDGE_MUTEX.set(Arc::new(Mutex::new(1))).unwrap_or(());

    PRACTICES
        .set(serde_json::from_reader(BufReader::new(File::open(&e.practices).unwrap())).unwrap())
        .unwrap_or(());

    USERS
        .set(serde_json::from_reader(BufReader::new(File::open(&e.users).unwrap())).unwrap())
        .unwrap_or(());

    HttpServer::new(|| {
        let auth = HttpAuthentication::basic(validator);
        App::new()
            .wrap(auth)
            .service(judge)
            .service(list)
            .service(Embed::new("", &Assets).index_file("index.html"))
    })
    .bind(("0.0.0.0", e.port))?
    .run()
    .await
}
