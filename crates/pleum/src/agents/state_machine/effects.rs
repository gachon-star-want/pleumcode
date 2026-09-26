use crate::conversation::message::Message;
use crate::conversation::Conversation;
use crate::providers::base::ProviderUsage;
use crate::recipe::Recipe;
use crate::session::ExtensionData;
use pleum_agent::operation::{ConversationEffect, MachineEffect};

pub enum PleumEffect {
    Conversation(ConversationEffect),
    ReplaceConversation {
        conversation: Conversation,
        usage: Option<ProviderUsage>,
    },
    SetRecipe(Box<Option<Recipe>>),
    SetExtensionData(ExtensionData),
    RecordUsage(ProviderUsage),
}

impl MachineEffect for PleumEffect {
    fn ensure_message_ids(&mut self) {
        match self {
            PleumEffect::Conversation(effect) => effect.ensure_message_ids(),
            PleumEffect::ReplaceConversation { conversation, .. } => {
                for message in conversation.messages_mut() {
                    if message.id.is_none() {
                        message.id = Some(format!("msg_{}", uuid::Uuid::new_v4()));
                    }
                }
            }
            _ => {}
        }
    }
}

impl From<ConversationEffect> for PleumEffect {
    fn from(effect: ConversationEffect) -> Self {
        PleumEffect::Conversation(effect)
    }
}

impl From<Message> for PleumEffect {
    fn from(message: Message) -> Self {
        ConversationEffect::from(message).into()
    }
}

impl From<Conversation> for PleumEffect {
    fn from(conversation: Conversation) -> Self {
        ConversationEffect::from(conversation).into()
    }
}
