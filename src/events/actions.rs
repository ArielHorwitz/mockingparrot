use crate::api::{get_completion, GptMessage};
use crate::state::State;
use anyhow::{Context, Result};
use std::process::Command;

const API_ERROR_FEEDBACK: &str = "An error occured, see debug logs.";
const API_ERROR_SYSTEM_MESSAGE: &str = "Failed to get a response from the assistant.";
const MESSAGE_FILE: &str = ".local/share/hummingparrot/message_text";

pub fn get_message_text_from_editor(
    config: &crate::config::Config,
    initial_text: &str,
) -> Result<String> {
    let user_home_dir = std::env::var("HOME").context("get HOME environment variable")?;
    let message_file = std::path::Path::new(&user_home_dir).join(MESSAGE_FILE);
    let message_dir = message_file
        .parent()
        .expect("get message file parent directory");
    std::fs::create_dir_all(message_dir).context("create directory for message file")?;
    std::fs::write(&message_file, initial_text).context("write initial text to message file")?;
    let mut editor_command_iter = config.ui.editor_command.iter();
    let editor_process_output =
        Command::new(editor_command_iter.next().context("editor command empty")?)
            .args(editor_command_iter.collect::<Vec<&String>>())
            .arg(&message_file)
            .output()
            .context("run editor")?;
    if !editor_process_output.status.success() {
        anyhow::bail!(format!(
            "editor process failed: {}",
            editor_process_output.status
        ));
    }
    let message_text = std::fs::read_to_string(message_file)
        .context("read text from message file")?
        .trim()
        .to_owned();
    Ok(message_text)
}

pub async fn do_prompt(state: &mut State) -> Result<()> {
    let client = reqwest::Client::new();
    let raw_response = get_completion(&client, &state.config, &state.conversation).await;
    match raw_response {
        Ok(response) => {
            let message = &response
                .choices
                .first()
                .context("missing response choices")?
                .message;
            state.conversation.add_message(message.clone());
            state.set_status_bar_text(format!("AI responded. {}", response.usage));
        }
        Err(error) => {
            state.set_status_bar_text(API_ERROR_FEEDBACK);
            state
                .conversation
                .add_message(GptMessage::new_system_message(
                    API_ERROR_SYSTEM_MESSAGE.to_owned(),
                ));
            state.add_debug_log(format!("{error}"));
        }
    }
    Ok(())
}
