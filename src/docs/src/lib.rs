#![allow(clippy::result_large_err)]

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use gix::bstr::ByteSlice;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use skil_core::skills::{Skill, discover_skills};
use skil_core::{Result, SkilError};

mod markdown;

#[derive(Args, Clone)]
#[command(about = "Build and serve static docs for discovered skills")]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: DocsCommand,
}

#[derive(Subcommand, Clone)]
pub enum DocsCommand {
    #[command(about = "Build a static website from repository skills")]
    Build(DocsBuildArgs),
    #[command(about = "Build and serve skill docs locally")]
    Serve(DocsServeArgs),
}

#[derive(Args, Clone)]
pub struct DocsBuildArgs {
    #[arg(long = "source", default_value = ".")]
    pub source: PathBuf,
    #[arg(long = "output", default_value = "site")]
    pub output: PathBuf,
    #[arg(long = "full-depth")]
    pub full_depth: bool,
}

#[derive(Args, Clone)]
pub struct DocsServeArgs {
    #[arg(long = "source", default_value = ".")]
    pub source: PathBuf,
    #[arg(long = "output", default_value = "site")]
    pub output: PathBuf,
    #[arg(long = "host", default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long = "port", default_value_t = 4173)]
    pub port: u16,
    #[arg(long = "full-depth")]
    pub full_depth: bool,
}

pub fn run_docs(args: DocsArgs) -> Result<()> {
    match args.command {
        DocsCommand::Build(args) => run_build(args),
        DocsCommand::Serve(args) => run_serve(args),
    }
}

pub fn run_build(args: DocsBuildArgs) -> Result<()> {
    let source = fs::canonicalize(&args.source)?;
    let output = args.output;
    let install_source = install_source_for(&source);

    let mut skills = discover_skills(&source, None, args.full_depth)?;
    if skills.is_empty() {
        return Err(SkilError::Message(format!(
            "No skills found in {}",
            source.display()
        )));
    }

    skills.sort_by_key(|a| a.name.to_lowercase());

    if output.exists() {
        fs::remove_dir_all(&output)?;
    }
    fs::create_dir_all(output.join("skills"))?;

    write_styles(&output)?;
    write_index(&output, &skills)?;
    for skill in &skills {
        write_skill_page(&output, &source, &install_source, skill)?;
    }

    println!(
        "Built docs for {} skill(s) in {}",
        skills.len(),
        output.display()
    );
    Ok(())
}

pub fn run_serve(args: DocsServeArgs) -> Result<()> {
    run_build(DocsBuildArgs {
        source: args.source,
        output: args.output.clone(),
        full_depth: args.full_depth,
    })?;

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr)?;
    let root = fs::canonicalize(&args.output)?;
    let docs_url = format!("http://{}", addr);

    println!("Serving docs at {}", docs_url);
    if let Err(err) = open::that(&docs_url) {
        eprintln!("Failed to open docs in browser: {err}");
    }

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Connection error: {err}");
                continue;
            }
        };

        if let Err(err) = serve_request(&mut stream, &root) {
            eprintln!("Request failed: {err}");
        }
    }

    Ok(())
}

fn write_styles(output: &Path) -> Result<()> {
    fs::write(output.join("styles.css"), STYLES)?;
    Ok(())
}

fn write_index(output: &Path, skills: &[Skill]) -> Result<()> {
    let page = page_shell(
        "Skill Docs",
        html! {
            h1 { "Skill Docs" }
            p class="lead" { "Discovered skills in this repository." }
            ul class="skills" {
                @for skill in skills {
                    li {
                        a href={ "/skills/" (slugify(&skill.name)) "/" } { (&skill.name) }
                        p { (&skill.description) }
                    }
                }
            }
        },
    );
    fs::write(output.join("index.html"), page.into_string())?;
    Ok(())
}

fn write_skill_page(
    output: &Path,
    source_root: &Path,
    install_source: &str,
    skill: &Skill,
) -> Result<()> {
    let slug = slugify(&skill.name);
    let dir = output.join("skills").join(slug);
    fs::create_dir_all(&dir)?;

    let content = markdown::markdown_to_html(markdown::strip_frontmatter(&skill.raw_content));
    let location = skill
        .path
        .strip_prefix(source_root)
        .unwrap_or(&skill.path)
        .display()
        .to_string();
    let install_cmd = format!(
        "skil add {} --skill {}",
        shell_escape_single_arg(install_source),
        shell_escape_single_arg(&skill.name)
    );

    let title = format!("{} | Skill Docs", skill.name);
    let page = page_shell(
        &title,
        html! {
            p { a href="/" { "← All skills" } }
            h1 { (&skill.name) }
            p class="lead" { (&skill.description) }
            p class="meta" { "Path: " (&location) }
            h2 { "Install" }
            pre { code { (&install_cmd) } }
            article class="content" { (PreEscaped(content)) }
        },
    );

    fs::write(dir.join("index.html"), page.into_string())?;
    Ok(())
}

fn page_shell(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                title { (title) }
                link rel="stylesheet" href="/styles.css";
            }
            body {
                main { (body) }
            }
        }
    }
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    let trimmed = out.trim_matches(&['-', '.'][..]);
    if trimmed.is_empty() {
        "skill".to_string()
    } else {
        trimmed.to_string()
    }
}

fn shell_escape_single_arg(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn install_source_for(source_root: &Path) -> String {
    detect_repo_install_source(source_root).unwrap_or_else(|| source_root.display().to_string())
}

fn detect_repo_install_source(source_root: &Path) -> Option<String> {
    let repo = gix::discover(source_root).ok()?;
    let repo_root = repo.workdir().or_else(|| repo.path().parent())?;
    let repo_root = fs::canonicalize(repo_root).ok()?;
    let origin = repo_origin_url(&repo)?;
    let normalized_origin = normalize_origin_source(&origin);
    let rel = source_root.strip_prefix(&repo_root).ok()?;

    if rel.as_os_str().is_empty() {
        return Some(normalized_origin);
    }

    let rel = rel.to_string_lossy().replace('\\', "/");
    let branch = repo_branch(&repo);

    if let Some(branch) = branch
        && let Some(url) = hosted_tree_url(&normalized_origin, &branch, &rel)
    {
        return Some(url);
    }

    Some(normalized_origin)
}

fn repo_origin_url(repo: &gix::Repository) -> Option<String> {
    let remote = repo.find_remote("origin".as_bytes().as_bstr()).ok()?;
    let url = remote.url(gix::remote::Direction::Fetch)?;
    let url = url.to_string();
    if url.is_empty() { None } else { Some(url) }
}

fn repo_branch(repo: &gix::Repository) -> Option<String> {
    let head = repo.head_name().ok().flatten()?;
    let short = head.shorten().to_str().ok()?;
    if short.is_empty() {
        None
    } else {
        Some(short.to_string())
    }
}

fn hosted_tree_url(origin: &str, branch: &str, rel: &str) -> Option<String> {
    let (host, owner, repo) = parse_hosted_origin(origin)?;
    match host {
        "github.com" => Some(format!("https://{host}/{owner}/{repo}/tree/{branch}/{rel}")),
        "gitlab.com" => Some(format!(
            "https://{host}/{owner}/{repo}/-/tree/{branch}/{rel}"
        )),
        "codeberg.org" => Some(format!(
            "https://{host}/{owner}/{repo}/src/branch/{branch}/{rel}"
        )),
        _ => None,
    }
}

fn parse_hosted_origin(origin: &str) -> Option<(&'static str, String, String)> {
    for host in ["github.com", "gitlab.com", "codeberg.org"] {
        if let Some(rest) = origin.strip_prefix(&format!("https://{host}/")) {
            return parse_owner_repo(host, rest);
        }
        if let Some(rest) = origin.strip_prefix(&format!("http://{host}/")) {
            return parse_owner_repo(host, rest);
        }
        if let Some(rest) = origin.strip_prefix(&format!("ssh://git@{host}/")) {
            return parse_owner_repo(host, rest);
        }
        if let Some(rest) = origin.strip_prefix(&format!("ssh://{host}/")) {
            return parse_owner_repo(host, rest);
        }
        if let Some(rest) = origin.strip_prefix(&format!("git@{host}:")) {
            return parse_owner_repo(host, rest);
        }
    }
    None
}

fn parse_owner_repo(host: &'static str, rest: &str) -> Option<(&'static str, String, String)> {
    let mut parts = rest
        .trim_end_matches('/')
        .split('/')
        .filter(|p| !p.is_empty());
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.trim_end_matches(".git").to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((host, owner, repo))
}

fn normalize_origin_source(origin: &str) -> String {
    if let Some((host, owner, repo)) = parse_hosted_origin(origin) {
        return format!("https://{host}/{owner}/{repo}");
    }
    origin.to_string()
}

fn serve_request(stream: &mut TcpStream, root: &Path) -> Result<()> {
    let mut buffer = [0u8; 8192];
    let bytes = stream.read(&mut buffer)?;
    if bytes == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let mut parts = request.lines().next().unwrap_or("").split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        return write_plain(
            stream,
            405,
            "Method Not Allowed",
            "Method not allowed",
            method == "HEAD",
        );
    }

    let mut relative = path.trim_start_matches('/').to_string();
    if relative.is_empty() {
        relative.push_str("index.html");
    }
    if relative.ends_with('/') {
        relative.push_str("index.html");
    }

    let mut requested = root.join(&relative);
    if requested.is_dir() {
        requested = requested.join("index.html");
    }

    if !requested.exists() {
        return write_plain(stream, 404, "Not Found", "Not found", method == "HEAD");
    }

    let canonical = fs::canonicalize(&requested)?;
    if !canonical.starts_with(root) {
        return write_plain(stream, 403, "Forbidden", "Forbidden", method == "HEAD");
    }

    let body = fs::read(&canonical)?;
    let content_type = content_type_for(&canonical);
    write_response(stream, 200, "OK", content_type, &body, method == "HEAD")
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn write_plain(
    stream: &mut TcpStream,
    status: u16,
    status_text: &str,
    body: &str,
    head_only: bool,
) -> Result<()> {
    write_response(
        stream,
        status,
        status_text,
        "text/plain; charset=utf-8",
        body.as_bytes(),
        head_only,
    )
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    stream.write_all(header.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()?;
    Ok(())
}

const STYLES: &str = include_str!("../assets/styles.css");
