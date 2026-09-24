//! Drives the built binary against an in-process mock of the Spaceship API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-with-query, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/api/v1/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("").to_string();
                    let path = raw_path.trim_start_matches("/api/v1/").to_string();
                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap_or(Value::Null));
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && *p == path)
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"code": "not_found", "message": "no route"})));
                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "spaceship-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_spaceship"))
            .args(args)
            .env("SPACESHIP_CONFIG_DIR", &self.dir)
            .env("SPACESHIP_SECRET_STORE", "plaintext")
            .env("SPACESHIP_ALLOW_PLAINTEXT_STORE", "1")
            .env("SPACESHIP_API_URL", &self.api)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self) -> Env {
        let (code, out, err) =
            self.json(&["accounts", "add", "work", "--api-key", "sp_key", "--api-secret", "sp_secret"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn probe_route() -> Route {
    ("GET", "domains?take=1", 200, json!({"items": [], "total": 0}))
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![probe_route(), probe_route(), probe_route()]);
    let env = Env::new(&mock).with_account();

    let last_req = mock.last("GET");
    assert_eq!(last_req.headers.iter().find(|(k, _)| k == "x-api-key").unwrap().1, "sp_key");
    assert_eq!(last_req.headers.iter().find(|(k, _)| k == "x-api-secret").unwrap().1, "sp_secret");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (_, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    // Duplicate account needs --force
    let (code, _, err) = env.json(&["accounts", "add", "WORK", "--api-key", "x", "--api-secret", "y"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    let (code, out, _) = env.json(&["accounts", "test", "Work"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");

    // Remove without --yes and non-interactive fails
    let (code, _, err) = env.json(&["accounts", "remove", "work"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--yes"));

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn api_credentials_from_stdin() {
    let mock = Mock::start(vec![probe_route()]);
    let env = Env::new(&mock);

    let mut child = Command::new(env!("CARGO_BIN_EXE_spaceship"))
        .args(["accounts", "add", "piped", "--api-key-stdin", "--api-secret", "sec_pipe", "--json"])
        .env("SPACESHIP_CONFIG_DIR", &env.dir)
        .env("SPACESHIP_SECRET_STORE", "plaintext")
        .env("SPACESHIP_ALLOW_PLAINTEXT_STORE", "1")
        .env("SPACESHIP_API_URL", &env.api)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(b"key_from_stdin\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));

    let key = mock.last("GET").headers.into_iter().find(|(k, _)| k == "x-api-key").unwrap().1;
    assert_eq!(key, "key_from_stdin");
}

#[test]
fn login_command_lifecycle() {
    let mock = Mock::start(vec![probe_route(), probe_route(), probe_route()]);
    let env = Env::new(&mock);

    // Default account name "default"
    let (code, out, err) = env.json(&["login", "--api-key", "key_def", "--api-secret", "sec_def"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "logged_in");
    assert_eq!(out["name"], "default");

    // Re-login without --force fails
    let (code, _, err) = env.json(&["login", "--api-key", "k2", "--api-secret", "s2"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    // Re-login with --force succeeds
    let (code, out, _) = env.json(&["login", "--api-key", "k2", "--api-secret", "s2", "--force"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "logged_in");

    // Login with named account
    let (code, out, _) = env.json(&["login", "staging", "--api-key", "k_stg", "--api-secret", "s_stg"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "staging");
}

#[test]
fn account_is_required_or_env_or_flags() {
    let mock = Mock::start(vec![
        probe_route(),
        ("GET", "domains?take=20&skip=0", 200, json!({"items": [{"name": "example.com"}]})),
        ("GET", "domains?take=20&skip=0", 200, json!({"items": [{"name": "example.com"}]})),
    ]);
    let env = Env::new(&mock).with_account();

    // 1. Without account, env vars, or flags: returns code 7 (NoAccount)
    let (code, _, err) = env.json(&["domains", "list"]);
    assert_eq!(code, 7);
    assert_eq!(err["code"], "no_account");
    assert!(err["detail"].as_str().unwrap().contains("work"));

    // 2. With flag credentials: works directly
    let (code, out, _) = env.json(&["domains", "list", "--api-key", "k", "--api-secret", "s"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["name"], "example.com");

    // 3. With explicit account: works
    let (code, out, _) = env.json(&["domains", "list", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["name"], "example.com");
}

#[test]
fn domains_operations() {
    let mock = Mock::start(vec![
        probe_route(),
        ("GET", "domains/example.com", 200, json!({"name": "example.com", "status": "active"})),
        ("GET", "domains/example.com/available", 200, json!({"available": true})),
        ("POST", "domains/available", 200, json!([{"domain": "example.com", "available": true}])),
        ("POST", "domains/example.com", 201, json!({"success": true, "asyncOperationId": "op_reg"})),
        ("PUT", "domains/example.com/autorenew", 200, json!({"success": true})),
        ("PUT", "domains/example.com/nameservers", 200, json!({"success": true})),
        ("GET", "domains/example.com/personal-nameservers", 200, json!([{"host": "ns1", "ips": ["192.0.2.1"]}])),
        ("PUT", "domains/example.com/personal-nameservers/ns1", 200, json!({"success": true})),
        ("DELETE", "domains/example.com/personal-nameservers/ns1", 204, json!(null)),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, _) = env.json(&["domains", "get", "example.com", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "example.com");

    let (code, out, _) = env.json(&["domains", "check", "example.com", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["available"], true);

    let (code, out, _) = env.json(&["domains", "check-batch", "example.com,test.com", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["domain"], "example.com");

    let (code, out, _) = env.json(&[
        "domains",
        "register",
        "example.com",
        "--registrant",
        "c_123",
        "--years",
        "2",
        "--auto-renew",
        "-a",
        "work",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out["asyncOperationId"], "op_reg");

    let (code, _, _) = env.json(&["domains", "autorenew", "example.com", "--enable", "-a", "work"]);
    assert_eq!(code, 0);
    let req = mock.last("PUT");
    assert_eq!(req.body.unwrap()["isEnabled"], true);

    let (code, _, _) = env.json(&[
        "domains",
        "nameservers",
        "example.com",
        "--provider",
        "custom",
        "--hosts",
        "ns1.example.com,ns2.example.com",
        "-a",
        "work",
    ]);
    assert_eq!(code, 0);
    let req = mock.last("PUT");
    assert_eq!(req.body.unwrap()["hosts"], json!(["ns1.example.com", "ns2.example.com"]));

    let (code, out, _) = env.json(&["domains", "personal-ns", "list", "example.com", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["host"], "ns1");

    let (code, _, _) = env.json(&[
        "domains",
        "personal-ns",
        "save",
        "example.com",
        "ns1",
        "--ips",
        "192.0.2.1,2001:db8::1",
        "-a",
        "work",
    ]);
    assert_eq!(code, 0);
    assert_eq!(mock.last("PUT").path, "domains/example.com/personal-nameservers/ns1");

    let (code, _, _) = env.json(&["domains", "personal-ns", "delete", "example.com", "ns1", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(mock.last("DELETE").path, "domains/example.com/personal-nameservers/ns1");
}

#[test]
fn dns_operations_and_filtering() {
    let mock = Mock::start(vec![
        probe_route(),
        (
            "GET",
            "dns/records/example.com?take=500&skip=0",
            200,
            json!({
                "items": [
                    {"name": "@", "type": "A", "address": "192.0.2.1", "ttl": 300},
                    {"name": "www", "type": "CNAME", "cname": "example.com", "ttl": 300},
                    {"name": "mail", "type": "MX", "exchange": "mail.example.com", "preference": 10}
                ],
                "total": 3
            }),
        ),
        ("PUT", "dns/records/example.com", 200, json!({"success": true})),
        ("DELETE", "dns/records/example.com", 200, json!({"success": true})),
    ]);
    let env = Env::new(&mock).with_account();

    // Filter by type
    let (code, out, _) = env.json(&["dns", "list", "example.com", "--type", "CNAME", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["total"], 1);
    assert_eq!(out["items"][0]["name"], "www");

    // Save with inline records via tempfile
    let records_file = env.dir.join("records.json");
    std::fs::write(&records_file, r#"[{"name":"www","type":"CNAME","cname":"example.com","ttl":300}]"#).unwrap();

    let (code, _, err) =
        env.json(&["dns", "save", "example.com", "--file", records_file.to_str().unwrap(), "--force", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    let put_req = mock.last("PUT");
    assert_eq!(put_req.body.unwrap()["force"], true);

    // Delete with items envelope via tempfile
    let del_file = env.dir.join("del.json");
    std::fs::write(&del_file, r#"{"items":[{"name":"www","type":"CNAME","cname":"example.com"}]}"#).unwrap();

    let (code, _, err) =
        env.json(&["dns", "delete", "example.com", "--file", del_file.to_str().unwrap(), "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    let del_req = mock.last("DELETE");
    assert!(del_req.body.unwrap().is_array());
}

#[test]
fn contacts_and_sellerhub_and_hyperlift() {
    let mock = Mock::start(vec![
        probe_route(),
        ("PUT", "contacts", 200, json!({"id": "contact_123"})),
        ("GET", "contacts/c1", 200, json!({"id": "c1", "firstName": "John"})),
        ("GET", "sellerhub/domains?take=20&skip=0", 200, json!({"items": []})),
        ("GET", "sellerhub/verification-records", 200, json!({"options": ["TXT", "CNAME"]})),
        ("GET", "hyperlift/applications?take=20&skip=0", 200, json!({"items": []})),
        ("PUT", "hyperlift/applications/app1/scale", 200, json!({"status": "ok"})),
        ("GET", "async-operations/op_99", 200, json!({"id": "op_99", "status": "completed"})),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, _) = env.json(&[
        "contacts",
        "save",
        "--first-name",
        "John",
        "--last-name",
        "Doe",
        "--email",
        "john@example.com",
        "--address",
        "123 Main St",
        "--city",
        "City",
        "--country",
        "US",
        "--phone",
        "+1.2125551234",
        "-a",
        "work",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out["id"], "contact_123");

    let (code, out, _) = env.json(&["contacts", "get", "c1", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["firstName"], "John");

    let (code, _, _) = env.json(&["sellerhub", "list", "-a", "work"]);
    assert_eq!(code, 0);

    let (code, out, _) = env.json(&["sellerhub", "verification", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["options"][0], "TXT");

    let (code, _, _) = env.json(&["hyperlift", "list", "-a", "work"]);
    assert_eq!(code, 0);

    let (code, out, _) = env.json(&["hyperlift", "scale", "app1", "--scale", "1", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "ok");

    let (code, out, _) = env.json(&["operations", "get", "op_99", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "completed");
}

#[test]
fn http_errors_mapping() {
    let mock = Mock::start(vec![
        probe_route(),
        ("GET", "domains/forbidden.com", 403, json!({"message": "Insufficient permissions"})),
        ("GET", "domains/missing.com", 404, json!({"message": "Domain not found"})),
        ("GET", "domains/busy.com", 429, json!({"message": "Slow down"})),
        ("GET", "domains/invalid.com", 422, json!({"message": "Invalid domain name"})),
        ("GET", "domains/broken.com", 500, json!({"message": "Internal error"})),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, _, err) = env.json(&["domains", "get", "forbidden.com", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    let (code, _, err) = env.json(&["domains", "get", "missing.com", "-a", "work"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");

    let (code, _, err) = env.json(&["domains", "get", "busy.com", "-a", "work"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");

    let (code, _, err) = env.json(&["domains", "get", "invalid.com", "-a", "work"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");

    let (code, _, err) = env.json(&["domains", "get", "broken.com", "-a", "work"]);
    assert_eq!(code, 2);
    assert_eq!(err["code"], "network");
}

#[test]
fn agent_readme_command() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "spaceship");
    assert_eq!(out["apiVersion"], "v1");
    assert!(out["rules"].is_array());
    assert_eq!(out["exitCodes"]["7"], "no_account - run spaceship accounts list or pass credentials");

    let out = env.run(&["agent-readme"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("# spaceship - agent operating manual"));
}

#[test]
fn yaml_default_and_json_modes() {
    let mock = Mock::start(vec![
        probe_route(),
        ("GET", "domains/example.com", 200, json!({"name": "example.com", "status": "active"})),
        ("GET", "domains/example.com", 200, json!({"name": "example.com", "status": "active"})),
    ]);
    let env = Env::new(&mock).with_account();

    // Default is YAML
    let out = env.run(&["domains", "get", "example.com", "-a", "work"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("name: example.com"), "stdout should be YAML: {stdout}");

    // --format json produces JSON
    let out = env.run(&["domains", "get", "example.com", "--format", "json", "-a", "work"]);
    assert_eq!(out.status.code(), Some(0));
    let val: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(val["name"], "example.com");
}
