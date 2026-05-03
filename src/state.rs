use crate::config::Config;
use crate::gemini::GeminiClient;

pub struct AppState {
    pub(crate) client: GeminiClient,
    pub(crate) config: Config,
}
