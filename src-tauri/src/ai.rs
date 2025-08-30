use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiContext {
    pub working_dir: Option<String>,
    pub prompt: Option<String>,
    pub recent_commands: Vec<String>,
    pub tail_output: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiRequest {
    pub task: String,
    pub user_input: String,
    pub context: AiContext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiResponse {
    pub text: String,
}

pub struct AiClient {
    provider: AiProvider,
}

#[derive(Clone, Debug)]
pub enum AiProvider {
    Mock,
    OpenAICompatible { base_url: String, api_key: String, model: String },
}

impl AiClient {
    pub fn from_env() -> Self {
        let provider = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "mock".into());
        if provider.eq_ignore_ascii_case("openai") || provider.eq_ignore_ascii_case("openai-compatible") {
            let base_url = std::env::var("AI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());
            let api_key = std::env::var("AI_API_KEY").unwrap_or_else(|_| "".into());
            let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
            if api_key.is_empty() {
                Self { provider: AiProvider::Mock }
            } else {
                Self { provider: AiProvider::OpenAICompatible { base_url, api_key, model } }
            }
        } else {
            Self { provider: AiProvider::Mock }
        }
    }

    pub async fn generate(&self, req: AiRequest) -> Result<AiResponse, String> {
        match &self.provider {
            AiProvider::Mock => Ok(mock_response(req)),
            AiProvider::OpenAICompatible { base_url, api_key, model } => {
                call_openai_compatible(base_url, api_key, model, req).await
            }
        }
    }
}

fn mock_response(req: AiRequest) -> AiResponse {
    let txt = match req.task.as_str() {
        "generate_command" => format!("# Suggested command based on your input\n# task: {}\n# dir: {}\n{}",
            req.user_input,
            req.context.working_dir.clone().unwrap_or_default(),
            mock_guess_command(&req.user_input)
        ),
        "explain_error" => format!("# Explanation\nIt looks like the error is related to: {}\nTry: {}",
            trim_error(&req.user_input),
            mock_fix_suggestion(&req.user_input)
        ),
        "suggest_next" => {
            let last = req.context.recent_commands.last().cloned().unwrap_or_else(|| "(no recent commands)".into());
            format!("# Next steps\nYou recently ran: {}\nConsider: {}",
                last,
                mock_next_step(&last)
            )
        }
        _ => "Unsupported task".into(),
    };
    AiResponse { text: txt }
}

fn mock_guess_command(input: &str) -> String {
    let s = input.to_lowercase();
    if s.contains("list") { "ls -la".into() }
    else if s.contains("git") && s.contains("status") { "git status".into() }
    else if s.contains("find") && s.contains("large") { "du -sh * | sort -h | tail -n 20".into() }
    else { "echo TODO".into() }
}

fn mock_fix_suggestion(err: &str) -> String {
    let e = err.to_lowercase();
    if e.contains("permission") { "rerun with sudo or fix file permissions".into() }
    else if e.contains("not found") { "ensure the command is installed and on PATH".into() }
    else { "search logs and retry with verbose flags".into() }
}

fn mock_next_step(last: &str) -> String {
    if last.starts_with("git ") { "git add . && git commit -m \"WIP\"".into() } else { "run tests or lint".into() }
}

fn trim_error(s: &str) -> String { s.lines().take(6).collect::<Vec<_>>().join("\n") }

#[derive(Serialize)]
struct OpenAiChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice { message: OpenAiChoiceMessage }

#[derive(Deserialize)]
struct OpenAiChoiceMessage { content: String }

async fn call_openai_compatible(base: &str, key: &str, model: &str, req: AiRequest) -> Result<AiResponse, String> {
    let system = match req.task.as_str() {
        "generate_command" => "You are a helpful terminal AI. Respond with a single shell command and a short explanation if needed.",
        "explain_error" => "You explain terminal errors concisely and propose a fix.",
        "suggest_next" => "You propose next terminal commands based on context.",
        _ => "You are an assistant.",
    };
    let ctx = format!(
        "Working dir: {:?}\nRecent commands:\n{}\nTail output:\n{}",
        req.context.working_dir,
        req.context.recent_commands.join("\n"),
        req.context.tail_output.join("\n")
    );
    let user = format!("{}\n\nUser input:\n{}", ctx, req.user_input);

    let body = OpenAiChatRequest {
        model,
        temperature: 0.2,
        messages: vec![
            OpenAiMessage { role: "system", content: system.into() },
            OpenAiMessage { role: "user", content: user },
        ],
    };

    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("AI error: {}", resp.status()));
    }
    let parsed: OpenAiChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let text = parsed.choices.get(0).map(|c| c.message.content.clone()).unwrap_or_default();
    Ok(AiResponse { text })
}
