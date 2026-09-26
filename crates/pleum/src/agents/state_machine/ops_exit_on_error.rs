//! Ends the turn when an error remains at the end of the conversation.

use anyhow::Result;
use async_trait::async_trait;

use crate::agents::state_machine::effects::PleumEffect;
use crate::agents::state_machine::{
    not_applicable, trailing_error, yielded, Emitter, Operation, OperationResult,
};
use crate::conversation::Conversation;
use crate::session::Session;

pub struct ExitOnErrorOperation;

#[async_trait]
impl Operation<Session, PleumEffect> for ExitOnErrorOperation {
    fn name(&self) -> &'static str {
        "exit_on_error"
    }

    async fn run(
        &self,
        _session: &Session,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> Result<OperationResult<PleumEffect>> {
        if trailing_error(conversation).is_none() {
            return not_applicable();
        }

        yielded()
    }
}
