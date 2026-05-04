//! Grounding contract enforcement.
//!
//! The grounding contract ensures that LLM generators may only rephrase or
//! reorder whitelisted source strings — they must never invent facts, add
//! roles, or introduce skills absent from the original data. This invariant
//! is enforced at the prompt-assembly layer so every adapter inherits it
//! automatically.
//!
//! See `plans/cross-cutting/llm-policy.md` for the operational policy and
//! ADR-015 for the architectural rationale.

use crate::error::LlmError;
use crate::types::{ChatMessage, LlmRequest, MessageRole};

/// What to do when the model produces output that appears to violate the
/// grounding contract.
#[derive(Debug, Clone)]
pub enum RefusalPolicy {
    /// Log the potential violation but return the response anyway.
    /// Post-generation verification is the caller's responsibility.
    WarnAndLog,
}

/// A whitelist of source strings the generator is allowed to draw from.
///
/// Prompt-assembly helpers use this to inject the grounding instruction and
/// the allowed source text into the prompt before handing it to the adapter.
#[derive(Debug, Clone)]
pub struct GroundingContract {
    /// Exact text strings (e.g. `job_details.detail_text` rows) that the model
    /// is permitted to rephrase or reorder.
    pub allowed_source_text: Vec<String>,
    pub refusal_policy: RefusalPolicy,
}

/// Inject the grounding contract into a request, returning a new request with
/// the grounding instruction prepended to the system prompt and the allowed
/// source text embedded.
///
/// Adapters should call this before constructing their API payload when
/// `LlmRequest::grounding` is `Some`.
///
/// # Errors
///
/// Returns [`LlmError::GroundingViolation`] if `allowed_source_text` is empty
/// (a contract with no whitelisted text would block all generation).
pub fn assemble_grounded_prompt(mut req: LlmRequest) -> Result<LlmRequest, LlmError> {
    let contract = match req.grounding.take() {
        Some(c) => c,
        None => return Ok(req),
    };

    if contract.allowed_source_text.is_empty() {
        return Err(LlmError::GroundingViolation {
            reason: "GroundingContract.allowed_source_text must not be empty".to_owned(),
        });
    }

    let source_block = contract
        .allowed_source_text
        .iter()
        .enumerate()
        .map(|(i, s)| format!("[source {}] {}", i + 1, s))
        .collect::<Vec<_>>()
        .join("\n");

    let grounding_instruction = format!(
        "You must only rephrase or reorder the following source strings. \
         Do not invent facts, add skills, or introduce content not present below. \
         Cite each claim as [source N].\n\n{source_block}"
    );

    let system = match req.system.take() {
        Some(existing) => format!("{existing}\n\n{grounding_instruction}"),
        None => grounding_instruction,
    };
    req.system = Some(system);

    // Also prepend a user-turn reminder so the contract is salient in the
    // conversation context, not just in the system turn.
    req.messages.insert(
        0,
        ChatMessage::text(
            MessageRole::User,
            "Remember: only rephrase the provided [source N] items. Do not invent.",
        ),
    );

    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GenerationConfig, LlmRequest};

    fn make_req(grounding: Option<GroundingContract>) -> LlmRequest {
        LlmRequest {
            model: String::new(),
            messages: vec![ChatMessage::text(MessageRole::User, "Rewrite my bullets.")],
            system: None,
            tools: vec![],
            grounding,
            config: GenerationConfig {
                max_tokens: 500,
                temperature: 0.3,
                prompt_version: "test-v1",
            },
        }
    }

    #[test]
    fn no_grounding_passthrough() {
        let req = make_req(None);
        let out = assemble_grounded_prompt(req).unwrap();
        assert!(out.system.is_none());
        assert_eq!(out.messages.len(), 1);
    }

    #[test]
    fn grounding_injects_system_and_reminder() {
        let contract = GroundingContract {
            allowed_source_text: vec![
                "Led migration to Rust".to_owned(),
                "Reduced latency by 40%".to_owned(),
            ],
            refusal_policy: RefusalPolicy::WarnAndLog,
        };
        let req = make_req(Some(contract));
        let out = assemble_grounded_prompt(req).unwrap();

        let system = out.system.unwrap();
        assert!(system.contains("[source 1] Led migration to Rust"));
        assert!(system.contains("[source 2] Reduced latency by 40%"));
        // Reminder message injected at position 0
        assert_eq!(out.messages[0].role, MessageRole::User);
        assert!(out.messages[0].text_content().contains("Do not invent"));
        // Original user message still present
        assert_eq!(out.messages[1].text_content(), "Rewrite my bullets.");
    }

    #[test]
    fn empty_source_text_is_an_error() {
        let contract = GroundingContract {
            allowed_source_text: vec![],
            refusal_policy: RefusalPolicy::WarnAndLog,
        };
        let req = make_req(Some(contract));
        let err = assemble_grounded_prompt(req).unwrap_err();
        assert!(matches!(err, LlmError::GroundingViolation { .. }));
    }
}
