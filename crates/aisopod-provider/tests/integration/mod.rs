// Integration tests for aisopod-provider
// These tests require actual provider API keys and are gated behind environment variables

#[cfg(test)]
pub mod anthropic;

#[cfg(test)]
pub mod openai;

#[cfg(test)]
pub mod gemini;

#[cfg(test)]
pub mod bedrock;

#[cfg(test)]
pub mod ollama;
