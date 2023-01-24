use std::fmt::Debug;
use std::fs::read_to_string;
use std::fs::OpenOptions;
use std::io::Write;

use noulith::{lex, Token};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

const TOKEN_TYPES: [SemanticTokenType; 7] = [
    SemanticTokenType::NUMBER,
    SemanticTokenType::STRING,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::COMMENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::STRUCT,
];

fn token_type_id(t: SemanticTokenType) -> u32 {
    (TOKEN_TYPES.iter()).position(|tt| *tt == t).unwrap() as _
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let mut result = InitializeResult::default();
        result.capabilities.semantic_tokens_provider = Some(
            SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: Some(false),
                },
                legend: SemanticTokensLegend {
                    token_types: TOKEN_TYPES.into_iter().collect(),
                    token_modifiers: vec![],
                },
                range: None,
                full: Some(SemanticTokensFullOptions::Bool(true)),
            }),
        );
        Ok(result)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(MessageType::INFO, format!("{:?}", params))
            .await;

        let code = read_to_string(params.text_document.uri.path()).unwrap();
        let tokens = lex(&code);

        let mut prev_line = 0;
        let mut prev_col = 0;
        let semantic_tokens = tokens
            .iter()
            .filter_map(|token| match token.token {
                Token::IntLit(_)
                | Token::RatLit(_)
                | Token::FloatLit(_)
                | Token::ImaginaryFloatLit(_) => Some((token, SemanticTokenType::NUMBER)),
                Token::StringLit(_) | Token::BytesLit(_) | Token::FormatString(_) => {
                    Some((token, SemanticTokenType::STRING))
                }
                Token::Ident(_) => Some((token, SemanticTokenType::FUNCTION)),
                Token::And
                | Token::Or
                | Token::Bang
                | Token::QuestionMark
                | Token::Colon
                | Token::LeftArrow
                | Token::RightArrow
                | Token::DoubleLeftArrow
                | Token::DoubleColon
                | Token::Ellipsis => Some((token, SemanticTokenType::OPERATOR)),
                Token::While
                | Token::For
                | Token::If
                | Token::Consume
                | Token::Pop
                | Token::Remove
                | Token::Swap
                | Token::Every
                | Token::Freeze
                | Token::Import
                | Token::Literally
                | Token::Else
                | Token::Switch
                | Token::Case
                | Token::Null
                | Token::Coalesce
                | Token::Yield
                | Token::Into
                | Token::Break
                | Token::Continue
                | Token::Return
                | Token::Throw
                | Token::Try
                | Token::Catch => Some((token, SemanticTokenType::KEYWORD)),
                Token::Comment(_) => Some((token, SemanticTokenType::COMMENT)),
                Token::Struct => Some((token, SemanticTokenType::STRUCT)),
                Token::LeftParen
                | Token::VLeftParen
                | Token::RightParen
                | Token::LeftBracket
                | Token::RightBracket
                | Token::LeftBrace
                | Token::RightBrace
                | Token::Semicolon
                | Token::Lambda
                | Token::Comma
                | Token::Assign
                | Token::Underscore
                | Token::Invalid(_) => None,
            })
            .map(|(loctoken, token_type)| {
                let delta_line = loctoken.start.line - 1 - prev_line;
                prev_line = loctoken.start.line - 1;
                let delta_start = if delta_line == 0 {
                    loctoken.start.col - 1 - prev_col
                } else {
                    loctoken.start.col - 1
                };
                prev_col = loctoken.start.col - 1;
                SemanticToken {
                    delta_line: delta_line as _,
                    delta_start: delta_start as _,
                    length: (loctoken.end.index - loctoken.start.index) as _,
                    token_type: token_type_id(token_type),
                    token_modifiers_bitset: 0,
                }
            })
            .collect();

        let result = Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })));
        result
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[allow(unused)]
fn dbg_log(arg: impl Debug) {
    let mut log_file = OpenOptions::new()
        .append(true)
        .open("/tmp/nlsp.log")
        .unwrap();
    write!(log_file, "{:?}\n", arg).unwrap();
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
