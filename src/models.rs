use uuid::Uuid;
use std::sync::{
    Arc, 
    Mutex
};
use axum::http::StatusCode;
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Deserialize)]
pub struct PollInput {
    pub title: String,
    pub tags: PollTags,
    pub options: PollOptionsInput,
}

#[derive(Deserialize)]
pub struct PollOptInput {
    pub option: String,
    pub reference: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollOpt {
    pub opt_id: Uuid,
    pub option: String,
    pub reference: Option<String>,
    pub vote: i32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poll {
    pub id: Uuid,
    pub title: String,
    pub tags: PollTags,
    pub options: PollOptions,
}

pub type PollOptions = Vec<PollOpt>;
pub type PollOptionsInput = Vec<PollOptInput>;
pub type PollTags = Vec<String>;
pub type SharedState = Arc<Mutex<Vec<Poll>>>;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>
}

#[derive(Deserialize)]
pub struct PollVote {
    pub opt_id: Uuid
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub messages: Option<String>,
    pub data: Option<T>
}

#[derive(Deserialize)]
pub enum PollVoteAct {
    INCREMENT,
    DECREMENT,
    RESET
}

#[allow(dead_code)]
impl<T> ApiResponse<T> {
    pub fn success(code: StatusCode, messages: Option<&str>, data: Option<T>) -> Self {
        Self {
            code: code.as_u16(),
            messages: messages.map(|msg: &str| msg.to_string()), 
            data
        }
    }

    pub fn error(code: StatusCode, messages: Option<&str>) -> Self {
        Self {
            code: code.as_u16(),
            messages: messages.map(|msg: &str| msg.to_string()), 
            data: None
        }
    }
}

#[allow(dead_code)]
pub trait PollExtra<T> {
    fn new(title: String, tags: PollTags, options: PollOptionsInput) -> Self;

    fn title(&self) -> &str;
    fn tags(&self) -> &PollTags;
    fn options(&self) -> &T;
    fn title_mut(&mut self) -> &mut String;
    fn tags_mut(&mut self) -> &mut PollTags;
    fn options_mut(&mut self) -> &mut T;

    fn is_empty(&self) -> bool;
    
    fn set_title(&mut self, title: String) { *self.title_mut() = title }
    fn set_tags(&mut self, tags: PollTags) { *self.tags_mut() = tags }
    fn set_options(&mut self, options: T) { *self.options_mut() = options }
}

#[allow(dead_code)]
impl Poll {
    pub fn push_tag(&mut self, tag: String) { self.tags_mut().push(tag) }

    pub fn pop_tag_at(&mut self, tag: String) -> Option<String> {
        if let Some(index) = self.tags.iter().position(
            |t: &String| *t == tag
        ) { Some(self.tags.remove(index)) }
        else { None }
    }

    pub fn pop_tag(&mut self, index: usize) -> Option<String> {
        if index < self.tags.len() { Some(self.tags.remove(index)) }
        else { None }
    }

    pub fn set_all(&mut self, poll: PollInput) {
        if !poll.title.is_empty() { self.title = poll.title }
        if !poll.tags.is_empty() { self.tags = poll.tags }
        if !poll.options.is_empty() { 
            self.options = poll.options.into_iter().map(
                |input: PollOptInput| PollOpt::new(input.option, input.reference)
            ).collect()
        }
    }

    pub fn push_option(&mut self, option: PollOpt) { self.options.push(option) }

    pub fn pop_option_at(&mut self, option: PollOpt) -> Option<PollOpt> {
        if let Some(index) = self.options.iter().position(
            |opt: &PollOpt| opt.opt_id == option.opt_id
        ) { Some(self.options.remove(index)) }
        else { None }
    }

    pub fn pop_option(&mut self, index: usize) -> Option<PollOpt> {
        if index < self.options.len() { Some(self.options.remove(index)) }
        else { None }
    }
}

impl PollExtra<PollOptions> for Poll {
    fn new(title: String, tags: PollTags, options: PollOptionsInput) -> Self {
        Self { id: Uuid::new_v4(), title, tags, options: options.into_iter().map(
            |input: PollOptInput| PollOpt::new(input.option, input.reference)
        ).collect() }
    }

    fn title(&self) -> &str { &self.title }
    fn tags(&self) -> &PollTags { &self.tags }
    fn options(&self) -> &PollOptions { &self.options }
    fn title_mut(&mut self) -> &mut String { &mut self.title }
    fn tags_mut(&mut self) -> &mut PollTags { &mut self.tags }
    fn options_mut(&mut self) -> &mut PollOptions { &mut self.options }

    fn is_empty(&self) -> bool { 
        self.title().trim().is_empty() || self.tags().is_empty() || self.options().is_empty() 
    }
}

impl PollExtra<PollOptionsInput> for PollInput {
    fn new(title: String, tags: PollTags, options: PollOptionsInput) -> Self {
        Self { title, tags, options }
    }

    fn title(&self) -> &str { &self.title }
    fn tags(&self) -> &PollTags { &self.tags }
    fn options(&self) -> &PollOptionsInput { &self.options }
    fn title_mut(&mut self) -> &mut String { &mut self.title }
    fn tags_mut(&mut self) -> &mut PollTags { &mut self.tags }
    fn options_mut(&mut self) -> &mut PollOptionsInput { &mut self.options }

    fn is_empty(&self) -> bool { 
        self.title().trim().is_empty() || self.tags().is_empty() || self.options().is_empty() 
    }
}

#[allow(dead_code)]
pub trait PollOptExtra {
    fn new(option: String, reference: Option<String>) -> Self;

    fn option(&self) -> &str;
    fn reference(&self) -> &Option<String>;
    fn option_mut(&mut self) -> &mut String;
    fn reference_mut(&mut self) -> &mut Option<String>;

    fn is_invalid(&self) -> bool { self.option().trim().is_empty() }
    fn has_reference(&self) -> bool { self.reference().is_some() }

    fn set_option(&mut self, option: String) { 
        if !option.trim().is_empty() { *self.option_mut() = option } 
    }

    fn set_reference(&mut self, reference: Option<String>) { *self.reference_mut() = reference }
}

#[allow(dead_code)]
pub trait PollOptVote {
    fn vote(&self) -> &i32;
    fn vote_mut(&mut self) -> &mut i32;

    fn increment_vote(&mut self) { *self.vote_mut() += 1 }

    fn decrement_vote(&mut self) { 
        let vote: &mut i32 = self.vote_mut();
        if *vote > 0 { *vote -= 1 } 
    }
    
    fn reset_vote(&mut self) { *self.vote_mut() = 0 }
}

impl PollOptExtra for PollOpt {
    fn new(option: String, reference: Option<String>) -> Self {
        Self { opt_id: Uuid::new_v4(), option, reference, vote: 0 }
    }

    fn option(&self) -> &str { &self.option }
    fn reference(&self) -> &Option<String> { &self.reference }
    fn option_mut(&mut self) -> &mut String { &mut self.option }
    fn reference_mut(&mut self) -> &mut Option<String> { &mut self.reference }
}

impl PollOptVote for PollOpt {
    fn vote(&self) -> &i32 { &self.vote }
    fn vote_mut(&mut self) -> &mut i32 { &mut self.vote }
}

impl PollOptExtra for PollOptInput {
    fn new(option: String, reference: Option<String>) -> Self {
        Self { option, reference }
    }

    fn option(&self) -> &str { &self.option }
    fn reference(&self) -> &Option<String> { &self.reference }
    fn option_mut(&mut self) -> &mut String { &mut self.option }
    fn reference_mut(&mut self) -> &mut Option<String> { &mut self.reference }
}