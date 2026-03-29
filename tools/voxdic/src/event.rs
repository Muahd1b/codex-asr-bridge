use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::App;

pub enum LoopControl {
    Continue,
    Quit,
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<LoopControl> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') if is_press_like(key.kind) => {
            return Ok(LoopControl::Quit)
        }
        KeyCode::Tab if is_press_like(key.kind) => {
            app.focus = app.focus.next();
        }
        KeyCode::Char('p') | KeyCode::Char('P') if is_press_like(key.kind) => {
            app.profile.rewrite_mode = app.profile.rewrite_mode.next();
            app.save_profile()?;
            app.push_runtime(format!(
                "Rewrite mode set to {}",
                app.profile.rewrite_mode.label()
            ));
        }
        KeyCode::Char('i') | KeyCode::Char('I') if is_press_like(key.kind) => {
            app.profile.inject_app = app.profile.inject_app.next();
            app.save_profile()?;
            app.push_runtime(format!(
                "Inject mode set to {}",
                app.profile.inject_app.label()
            ));
        }
        KeyCode::Char('l') | KeyCode::Char('L') if is_press_like(key.kind) => {
            app.profile.live_inject = !app.profile.live_inject;
            app.save_profile()?;
            app.push_runtime(format!(
                "Live inject {}",
                if app.profile.live_inject {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
        }
        KeyCode::Char('r') | KeyCode::Char('R') if is_press_like(key.kind) => {
            app.reload_profile()?;
        }
        KeyCode::Char('v') | KeyCode::Char('V') if is_press_like(key.kind) => {
            match app.voxtral.validate() {
                Ok(_) => app.push_runtime("Voxtral validation OK"),
                Err(err) => app.push_runtime(format!("Voxtral validation failed: {}", err)),
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') if is_press_like(key.kind) => {
            if let Err(err) = app.toggle_global_ptt() {
                app.push_runtime(format!("Global PTT toggle failed: {}", err));
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') if is_press_like(key.kind) => {
            if let Err(err) = app.command_mode_rewrite_selected() {
                app.push_talk(format!("ERROR: {}", err));
            }
        }
        _ => {}
    }
    Ok(LoopControl::Continue)
}

fn is_press_like(kind: KeyEventKind) -> bool {
    matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat)
}
