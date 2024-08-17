use crate::api::{get_completion, GptMessage};
use crate::state::State;
use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;

const API_ERROR_FEEDBACK: &str = "An error occured, see debug logs.";
const API_ERROR_SYSTEM_MESSAGE: &str = "Failed to get a response from the assistant.";

pub fn get_message_text_from_editor(state: &State, initial_text: &str) -> Result<String> {
    std::fs::write(&state.paths.message_file, initial_text)
        .context("write initial text to message file")?;
    let mut editor_command_iter = state.config.commands.editor.iter();
    let editor_process_output =
        Command::new(editor_command_iter.next().context("editor command empty")?)
            .args(editor_command_iter.collect::<Vec<&String>>())
            .arg(&state.paths.message_file)
            .output()
            .context("run editor")?;
    if !editor_process_output.status.success() {
        anyhow::bail!(format!(
            "editor process failed: {}",
            editor_process_output.status
        ));
    }
    let message_text = std::fs::read_to_string(&state.paths.message_file)
        .context("read text from message file")?
        .trim()
        .to_owned();
    Ok(message_text)
}

pub async fn do_prompt(state: &mut State) -> Result<()> {
    let client = reqwest::Client::new();
    let raw_response =
        get_completion(&client, &state.config, state.get_active_conversation()?).await;
    match raw_response {
        Ok(response) => {
            let message = &response
                .choices
                .first()
                .context("missing response choices")?
                .message;
            state
                .get_active_conversation_mut()?
                .add_message(message.clone());
            state.set_status_bar_text(format!("AI responded. {}", response.usage));
            state.add_debug_log(response.usage.to_string());
        }
        Err(error) => {
            state.set_status_bar_text(API_ERROR_FEEDBACK);
            state
                .get_active_conversation_mut()?
                .add_message(GptMessage::new_system_message(
                    API_ERROR_SYSTEM_MESSAGE.to_owned(),
                ));
            state.add_debug_log(format!("{error}"));
        }
    }
    Ok(())
}

pub fn export_to_clipboard(state: &State, text: &str) -> Result<()> {
    let mut command_iter = state.config.commands.copy.iter();
    let mut child_process = Command::new(command_iter.next().context("clipboard command empty")?)
        .args(command_iter.collect::<Vec<&String>>())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawn clipboard command")?;
    let child_stdin = child_process
        .stdin
        .as_mut()
        .context("clipboard command stdin handle")?;
    child_stdin
        .write_all(text.as_bytes())
        .context("write to clipboard command pipe")?;
    child_process
        .wait()
        .context("wait on clipboard command process")?;
    Ok(())
}
