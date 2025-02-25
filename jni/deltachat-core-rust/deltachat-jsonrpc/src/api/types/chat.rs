use std::time::{Duration, SystemTime};

use anyhow::{bail, Context as _, Result};
use deltachat::chat::{self, get_chat_contacts, ChatVisibility};
use deltachat::chat::{Chat, ChatId};
use deltachat::constants::Chattype;
use deltachat::contact::{Contact, ContactId};
use deltachat::context::Context;
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use typescript_type_def::TypeDef;

use super::color_int_to_hex_string;
use super::contact::ContactObject;

#[derive(Serialize, TypeDef, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FullChat {
    id: u32,
    name: String,

    /// True if the chat is protected.
    ///
    /// UI should display a green checkmark
    /// in the chat title,
    /// in the chat profile title and
    /// in the chatlist item
    /// if chat protection is enabled.
    /// UI should also display a green checkmark
    /// in the contact profile
    /// if 1:1 chat with this contact exists and is protected.
    is_protected: bool,
    profile_image: Option<String>, //BLOBS ?
    archived: bool,
    pinned: bool,
    // subtitle  - will be moved to frontend because it uses translation functions
    chat_type: u32,
    is_unpromoted: bool,
    is_self_talk: bool,
    contacts: Vec<ContactObject>,
    contact_ids: Vec<u32>,
    color: String,
    fresh_message_counter: usize,
    // is_group - please check over chat.type in frontend instead
    is_contact_request: bool,
    is_protection_broken: bool,
    is_device_chat: bool,
    self_in_group: bool,
    is_muted: bool,
    ephemeral_timer: u32, //TODO look if there are more important properties in newer core versions
    can_send: bool,
    was_seen_recently: bool,
    mailing_list_address: Option<String>,
}

impl FullChat {
    pub async fn try_from_dc_chat_id(context: &Context, chat_id: u32) -> Result<Self> {
        let rust_chat_id = ChatId::new(chat_id);
        let chat = Chat::load_from_db(context, rust_chat_id).await?;

        let contact_ids = get_chat_contacts(context, rust_chat_id).await?;

        let mut contacts = Vec::with_capacity(contact_ids.len());

        for contact_id in &contact_ids {
            contacts.push(
                ContactObject::try_from_dc_contact(
                    context,
                    Contact::get_by_id(context, *contact_id)
                        .await
                        .context("failed to load contact")?,
                )
                .await?,
            )
        }

        let profile_image = match chat.get_profile_image(context).await? {
            Some(path_buf) => path_buf.to_str().map(|s| s.to_owned()),
            None => None,
        };

        let color = color_int_to_hex_string(chat.get_color(context).await?);
        let fresh_message_counter = rust_chat_id.get_fresh_msg_cnt(context).await?;
        let ephemeral_timer = rust_chat_id.get_ephemeral_timer(context).await?.to_u32();

        let can_send = chat.can_send(context).await?;

        let was_seen_recently = if chat.get_type() == Chattype::Single {
            match contact_ids.first() {
                Some(contact) => Contact::get_by_id(context, *contact)
                    .await
                    .context("failed to load contact for was_seen_recently")?
                    .was_seen_recently(),
                None => false,
            }
        } else {
            false
        };

        let mailing_list_address = chat.get_mailinglist_addr().map(|s| s.to_string());

        Ok(FullChat {
            id: chat_id,
            name: chat.name.clone(),
            is_protected: chat.is_protected(),
            profile_image, //BLOBS ?
            archived: chat.get_visibility() == chat::ChatVisibility::Archived,
            pinned: chat.get_visibility() == chat::ChatVisibility::Pinned,
            chat_type: chat.get_type().to_u32().context("unknown chat type id")?,
            is_unpromoted: chat.is_unpromoted(),
            is_self_talk: chat.is_self_talk(),
            contacts,
            contact_ids: contact_ids.iter().map(|id| id.to_u32()).collect(),
            color,
            fresh_message_counter,
            is_contact_request: chat.is_contact_request(),
            is_protection_broken: chat.is_protection_broken(),
            is_device_chat: chat.is_device_talk(),
            self_in_group: contact_ids.contains(&ContactId::SELF),
            is_muted: chat.is_muted(),
            ephemeral_timer,
            can_send,
            was_seen_recently,
            mailing_list_address,
        })
    }
}

/// cheaper version of fullchat, omits:
/// - contacts
/// - contact_ids
/// - fresh_message_counter
/// - ephemeral_timer
/// - self_in_group
/// - was_seen_recently
/// - can_send
///
/// used when you only need the basic metadata of a chat like type, name, profile picture
#[derive(Serialize, TypeDef, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BasicChat {
    id: u32,
    name: String,

    /// True if the chat is protected.
    ///
    /// UI should display a green checkmark
    /// in the chat title,
    /// in the chat profile title and
    /// in the chatlist item
    /// if chat protection is enabled.
    /// UI should also display a green checkmark
    /// in the contact profile
    /// if 1:1 chat with this contact exists and is protected.
    is_protected: bool,
    profile_image: Option<String>, //BLOBS ?
    archived: bool,
    pinned: bool,
    chat_type: u32,
    is_unpromoted: bool,
    is_self_talk: bool,
    color: String,
    is_contact_request: bool,
    is_protection_broken: bool,
    is_device_chat: bool,
    is_muted: bool,
}

impl BasicChat {
    pub async fn try_from_dc_chat_id(context: &Context, chat_id: u32) -> Result<Self> {
        let rust_chat_id = ChatId::new(chat_id);
        let chat = Chat::load_from_db(context, rust_chat_id).await?;

        let profile_image = match chat.get_profile_image(context).await? {
            Some(path_buf) => path_buf.to_str().map(|s| s.to_owned()),
            None => None,
        };
        let color = color_int_to_hex_string(chat.get_color(context).await?);

        Ok(BasicChat {
            id: chat_id,
            name: chat.name.clone(),
            is_protected: chat.is_protected(),
            profile_image, //BLOBS ?
            archived: chat.get_visibility() == chat::ChatVisibility::Archived,
            pinned: chat.get_visibility() == chat::ChatVisibility::Pinned,
            chat_type: chat.get_type().to_u32().context("unknown chat type id")?,
            is_unpromoted: chat.is_unpromoted(),
            is_self_talk: chat.is_self_talk(),
            color,
            is_contact_request: chat.is_contact_request(),
            is_protection_broken: chat.is_protection_broken(),
            is_device_chat: chat.is_device_talk(),
            is_muted: chat.is_muted(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize, TypeDef, schemars::JsonSchema)]
#[serde(tag = "kind")]
pub enum MuteDuration {
    NotMuted,
    Forever,
    Until { duration: i64 },
}

impl MuteDuration {
    pub fn try_into_core_type(self) -> Result<chat::MuteDuration> {
        match self {
            MuteDuration::NotMuted => Ok(chat::MuteDuration::NotMuted),
            MuteDuration::Forever => Ok(chat::MuteDuration::Forever),
            MuteDuration::Until { duration } => {
                if duration <= 0 {
                    bail!("failed to read mute duration")
                }

                Ok(SystemTime::now()
                    .checked_add(Duration::from_secs(duration as u64))
                    .map_or(chat::MuteDuration::Forever, chat::MuteDuration::Until))
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TypeDef, schemars::JsonSchema)]
#[serde(rename = "ChatVisibility")]
pub enum JSONRPCChatVisibility {
    Normal,
    Archived,
    Pinned,
}

impl JSONRPCChatVisibility {
    pub fn into_core_type(self) -> ChatVisibility {
        match self {
            JSONRPCChatVisibility::Normal => ChatVisibility::Normal,
            JSONRPCChatVisibility::Archived => ChatVisibility::Archived,
            JSONRPCChatVisibility::Pinned => ChatVisibility::Pinned,
        }
    }
}
