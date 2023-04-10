use super::content::{user_joined_event_content::UserJoinedEventContent, initial_roster_event_content::InitialRosterEventContent, typing_user_event_content::TypingUserEventContent, message_event_content::MessageEventContent, file_upload_event_content::FileUploadEventContent};

#[derive(Clone, Debug)]

pub enum SwitchboardEvent {
    UserJoinedEvent(UserJoinedEventContent),
    InitialRosterEvent(InitialRosterEventContent),
    MessageEvent(MessageEventContent),
    TypingUserEvent(TypingUserEventContent),
    FileUploadEvent(FileUploadEventContent)
}

impl From<TypingUserEventContent> for SwitchboardEvent {
    fn from(v: TypingUserEventContent) -> Self {
        Self::TypingUserEvent(v)
    }
}


impl From<InitialRosterEventContent> for SwitchboardEvent {
    fn from(v: InitialRosterEventContent) -> Self {
        Self::InitialRosterEvent(v)
    }
}

impl From<UserJoinedEventContent> for SwitchboardEvent {
    fn from(v: UserJoinedEventContent) -> Self {
        Self::UserJoinedEvent(v)
    }
}

impl From<MessageEventContent> for SwitchboardEvent {
    fn from(v: MessageEventContent) -> Self {
        Self::MessageEvent(v)
    }
}

impl From<FileUploadEventContent> for SwitchboardEvent {
    fn from(v: FileUploadEventContent) -> Self {
        Self::FileUploadEvent(v)
    }
}


