use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Cache, Config, Llama, LlamaConfig};
use log::info;
use tokenizers::Tokenizer;

pub struct AiBot {
    model: Llama,
    tokenizer: Tokenizer,
    device: Device,
    config: Config,
}

impl AiBot {
    pub async fn new() -> Result<Self> {
        let device = Device::Cpu;
        let api = hf_hub::api::tokio::Api::new()?;
        let repo = api.repo(hf_hub::Repo::with_revision(
            "HuggingFaceTB/SmolLM-360M-Instruct".to_string(),
            hf_hub::RepoType::Model,
            "main".to_string(),
        ));

        info!("Downloading/Finding tokenizer...");
        let tokenizer_filename = repo.get("tokenizer.json").await?;

        info!("Downloading/Finding config...");
        let config_filename = repo.get("config.json").await?;

        info!("Downloading/Finding model weights...");
        let weights_filename = repo.get("model.safetensors").await?;

        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        let llama_config: LlamaConfig =
            serde_json::from_reader(std::fs::File::open(config_filename)?)?;
        let config: Config = llama_config.into_config(false);

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)?
        };

        let model = Llama::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
        })
    }

    pub fn generate(&mut self, prompt: String) -> Result<String> {
        let mut cache = Cache::new(true, DType::F32, &self.config, &self.device)?;

        // Ensure we don't exceed token limits by encoding first, then truncating if needed
        // Since we are passing pre-formatted prompt string now (which contains history + current message),
        // we encode it without special tokens logic here and just truncate from the start (oldest context).
        let mut tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Failed to encode: {}", e))?
            .get_ids()
            .to_vec();

        // Max context length for SmolLM is 2048. We'll leave 150 for generation.
        if tokens.len() > 1850 {
            let offset = tokens.len() - 1850;
            tokens = tokens[offset..].to_vec();
        }

        let mut generated_tokens = vec![];

        let max_tokens = 150;
        let mut current_pos = 0;

        for index in 0..max_tokens {
            let context_size = if index == 0 { tokens.len() } else { 1 };
            let start_pos = tokens.len().saturating_sub(context_size);

            let input = Tensor::new(&tokens[start_pos..], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, current_pos, &mut cache)?;

            // Extract the logits for the last token in the sequence to get the next token
            let logits = logits.squeeze(0)?;
            let seq_len = logits.dim(0)?;
            let logits = logits.get(seq_len - 1)?;

            // Greedy search
            let next_token = logits.argmax(0)?.to_scalar::<u32>()?;

            if next_token == self.tokenizer.token_to_id("<|im_end|>").unwrap_or(0) {
                break;
            }

            tokens.push(next_token);
            generated_tokens.push(next_token);
            current_pos += context_size;
        }

        let result = self
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;

        Ok(result.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Can take a while to download weights
    async fn test_ai_bot_initialization_and_generation() {
        let mut bot = AiBot::new().await.expect("Failed to initialize AiBot");
        let response = bot
            .generate("Hi".to_string())
            .expect("Failed to generate response");
        assert!(!response.is_empty(), "Response should not be empty");
    }
}
