use anyhow::Result;
use ai_shell::{AiShell, AiShellConfig};
use eventhd_neuromorph::{EventHdEncoder, NeuromorphicEvent};
use hd5d_core::{Identity5D, IdentityEncoder};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let id_encoder = IdentityEncoder::new();
    let event_encoder = EventHdEncoder::new(hd5d_core::DIM);

    // Example neuromorphic event window (in real use, read from .nstream.neuroaln).
    let events = vec![
        NeuromorphicEvent { x: 10, y: 20, polarity: true, timestamp_us: 1_000 },
        NeuromorphicEvent { x: 11, y: 21, polarity: false, timestamp_us: 1_050 },
    ];

    let id5d = Identity5D {
        biostate: "calm".into(),
        neurostate: "focus".into(),
        lifeforce: "normal".into(),
        context: "lab-chat".into(),
        sovereignty: "primary".into(),
    };

    let hv = event_encoder.encode_window(&events, &id5d, &id_encoder);
    let _similarity_self = hv.similarity(&hv);

    let cfg = AiShellConfig {
        api_url: "https://api.openai.com/v1/chat/completions".into(),
        api_key: std::env::var("OPENAI_API_KEY")?,
        subjectid: "bostrom18sd2u...".into(),
    };

    let policy_dir = Path::new("./policies");
    let shell = AiShell::new(cfg, policy_dir).await?;

    let response = shell
        .chat("Explain a safe neuromorphic encoding strategy respecting mental privacy.")
        .await?;

    println!("AI-shell response:\n{}", response);
    Ok(())
}
