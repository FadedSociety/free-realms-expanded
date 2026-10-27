//! T543: multi-line quest talk conversations with player responses and retries.
//!
//! Wire: the server's 26/3 shows one line and up to five response rows. The client
//! `ShowDialogFromPacket` (`0x009f5ae0`) copies each row's first dword into the open
//! dialog's choice list (the same value `decode_dialogue_26_3` reads as a row's
//! first word), and `ChoiceResponseByIndex` (`0x009f6aa0`) writes the chosen one
//! into the 26/6 body (constructor `0x009f5890`; reader
//! `npc_interaction::decode_dialogue_response_26_6`). Rows are numbered from 1 in
//! content order so they cannot collide with the 26/3 Goodbye row (300) or a quest
//! offer row (301).
//!
//! The cursor is the fork's conversation behavior, reimplemented (R): a line with
//! choices is a question, a `retry` choice plays its `then` lines and then asks the
//! question again, and an answer the current question does not offer abandons the
//! conversation without finishing it. Nothing here decides when a quest step
//! completes; `QuestEngine::DialogueFinished` does that after a finished cursor.

use crate::npc_interaction::dialogue_choice_row;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// (R) A goal's whole conversation is at most this many lines: the fork's goals are
/// short, and the bound also caps a retry loop's queue.
pub const MAX_LINES: usize = 16;
/// The 26/3 reader `0x00a9ef10` lists at most five response nodes and
/// `ShowDialogFromPacket` stops at `v44 >= 5`.
pub const MAX_CHOICES: usize = 5;
/// (R) At most two `then` levels: the fork's quiz nests none, and the recursion is
/// content, so it is bounded at load rather than at runtime.
pub const MAX_DEPTH: usize = 2;
/// (R) Last-line button label when content gives no `response_text_id`; client
/// locale 103085 ("You got it!"). The fork uses the same fallback.
pub const YOU_GOT_IT_STRING_ID: u32 = 103_085;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueLine {
    /// `Global.Text` id of the dialog body (`0x00a9ef10` reads it as the first word).
    pub text_id: u32,
    /// Label of the one continue button when `choices` is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_text_id: Option<u32>,
    /// Question rows; empty means one continue button.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<DialogueChoice>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueChoice {
    /// `Global.Text` id of this row's label.
    pub response_text_id: u32,
    /// Ask the question again after `then` plays.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retry: bool,
    /// Lines shown after this answer, before the rest of the conversation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub then: Vec<DialogueLine>,
}

impl DialogueLine {
    /// The 26/3 response rows for this line, numbered from 1. A line without
    /// choices shows one continue button; its label falls back to "You got it!".
    pub fn response_rows(&self) -> Vec<[u32; 5]> {
        if self.choices.is_empty() {
            let label = self.response_text_id.unwrap_or(YOU_GOT_IT_STRING_ID);
            return vec![dialogue_choice_row(1, label)];
        }
        (1..)
            .zip(&self.choices)
            .map(|(id, choice)| dialogue_choice_row(id, choice.response_text_id))
            .collect()
    }
}

/// One talk step's authored conversation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TalkDialogue {
    pub quest_id: u32,
    pub objective_id: u32,
    /// The talk does not complete the step; finishing this conversation does.
    pub complete_after_dialogue: bool,
    pub lines: Vec<DialogueLine>,
}

/// A conversation one Talk started. Session-only state on `Gameplay`: a reconnect
/// has no cursor and the next Talk starts the step at its first line.
#[derive(Clone, Debug)]
pub struct OpenConversation {
    pub quest_id: u32,
    pub objective_id: u32,
    /// Content NPC key the Talk named (the quest event's npc id).
    pub npc_id: u64,
    /// Runtime actor GUID the client clicked; the 26/3 camera target.
    pub actor_id: u64,
    pub complete_after_dialogue: bool,
    pub conversation: Conversation,
}

/// A conversation in progress: the line on screen and the lines still to come.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conversation {
    current: Option<DialogueLine>,
    remaining: VecDeque<DialogueLine>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationStep {
    /// A next line is on screen.
    Next,
    /// The last line was answered; the conversation finished.
    Finished,
    /// The answer was not on the current question; nothing changed.
    Abandoned,
}

impl Conversation {
    /// Start at the first line. Empty lines are no conversation (`None`), which
    /// the load bounds refuse for content anyway.
    pub fn start(lines: &[DialogueLine]) -> Option<Self> {
        let mut remaining: VecDeque<DialogueLine> = lines.iter().cloned().collect();
        let current = remaining.pop_front()?;
        Some(Self {
            current: Some(current),
            remaining,
        })
    }

    pub fn line(&self) -> Option<&DialogueLine> {
        self.current.as_ref()
    }

    /// Move on after the player answers. A question validates its answer; a line
    /// without choices shows one continue button (id 1), so any other id abandons
    /// it. A retry whose queue would pass [`MAX_LINES`] abandons instead, so a
    /// client cannot grow the queue without bound.
    pub fn answer(&mut self, response_id: i32) -> ConversationStep {
        let Some(current) = self.current.as_ref() else {
            return ConversationStep::Finished;
        };
        if current.choices.is_empty() {
            if response_id != 1 {
                return ConversationStep::Abandoned;
            }
        } else {
            let Some(index) = response_id
                .checked_sub(1)
                .and_then(|index| usize::try_from(index).ok())
                .filter(|index| *index < current.choices.len())
            else {
                return ConversationStep::Abandoned;
            };
            let choice = &current.choices[index];
            let mut next: VecDeque<DialogueLine> = choice.then.iter().cloned().collect();
            if choice.retry {
                next.push_back(current.clone());
            }
            if next.len() + self.remaining.len() > MAX_LINES {
                return ConversationStep::Abandoned;
            }
            next.extend(self.remaining.iter().cloned());
            self.remaining = next;
        }
        self.current = self.remaining.pop_front();
        if self.current.is_some() {
            ConversationStep::Next
        } else {
            ConversationStep::Finished
        }
    }
}

/// Bound and shape-check one goal's lines (`Ok` for no conversation). The split
/// loader and `content::ContentDb` both call this, so a bad record is refused at
/// load on every path.
pub fn validate(lines: &[DialogueLine]) -> Result<(), &'static str> {
    let mut total = 0;
    for line in lines {
        validate_line(line, 0, &mut total)?;
    }
    Ok(())
}

fn validate_line(line: &DialogueLine, depth: usize, total: &mut usize) -> Result<(), &'static str> {
    *total += 1;
    if *total > MAX_LINES {
        return Err("dialogue has more than 16 lines");
    }
    if line.text_id == 0 || line.response_text_id == Some(0) {
        return Err("dialogue text ids must be positive");
    }
    if line.choices.len() > MAX_CHOICES {
        return Err("dialogue line has more than 5 choices");
    }
    for choice in &line.choices {
        if choice.response_text_id == 0 {
            return Err("dialogue response text ids must be positive");
        }
        if depth + 1 > MAX_DEPTH && !choice.then.is_empty() {
            return Err("dialogue then nesting is deeper than 2");
        }
        for then in &choice.then {
            validate_line(then, depth + 1, total)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text_id: u32, choices: Vec<DialogueChoice>) -> DialogueLine {
        DialogueLine {
            text_id,
            response_text_id: None,
            choices,
        }
    }

    fn choice(response_text_id: u32, retry: bool, then: Vec<DialogueLine>) -> DialogueChoice {
        DialogueChoice {
            response_text_id,
            retry,
            then,
        }
    }

    #[test]
    fn a_retry_reasks_the_question_after_its_then_line() {
        let question = line(10, vec![choice(11, true, vec![line(12, Vec::new())])]);
        let mut conversation = Conversation::start(&[question]).expect("one line");
        assert_eq!(conversation.line().unwrap().text_id, 10);
        assert_eq!(conversation.answer(1), ConversationStep::Next);
        assert_eq!(conversation.line().unwrap().text_id, 12);
        assert_eq!(conversation.answer(1), ConversationStep::Next);
        assert_eq!(
            conversation.line().unwrap().text_id,
            10,
            "the retry re-asks"
        );
        assert_eq!(conversation.answer(2), ConversationStep::Abandoned);
        assert_eq!(
            conversation.line().unwrap().text_id,
            10,
            "abandon changes nothing"
        );
    }

    #[test]
    fn a_retry_that_would_pass_the_line_bound_abandons() {
        let mut lines = vec![line(10, vec![choice(11, true, vec![line(12, Vec::new())])])];
        lines.extend((0..15).map(|index| line(100 + index, Vec::new())));
        // Sixteen validated lines plus a retry would queue seventeen; refuse it
        // rather than grow the queue on every answer.
        let mut conversation = Conversation::start(&lines).expect("one line");
        assert_eq!(conversation.answer(1), ConversationStep::Abandoned);
        assert_eq!(conversation.line().unwrap().text_id, 10);
    }

    #[test]
    fn bounds_refuse_six_choices_seventeen_lines_and_depth_three() {
        assert!(validate(&[]).is_ok());
        assert!(validate(&[line(1, vec![choice(2, false, Vec::new()); 5])]).is_ok());
        assert!(validate(&[line(1, vec![choice(2, false, Vec::new()); 6])]).is_err());
        assert!(validate(&vec![line(1, Vec::new()); 17]).is_err());
        let depth_three = line(
            1,
            vec![choice(
                2,
                false,
                vec![line(
                    3,
                    vec![choice(
                        4,
                        false,
                        vec![line(5, vec![choice(6, false, vec![line(7, Vec::new())])])],
                    )],
                )],
            )],
        );
        assert!(validate(&[depth_three]).is_err());
    }

    #[test]
    fn response_rows_are_numbered_from_one() {
        let question = line(10, vec![choice(20, false, Vec::new()); 3]);
        assert_eq!(
            question
                .response_rows()
                .iter()
                .map(|row| row[0])
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
        let continue_line = line(10, Vec::new());
        assert_eq!(continue_line.response_rows()[0][2], YOU_GOT_IT_STRING_ID);
    }
}
