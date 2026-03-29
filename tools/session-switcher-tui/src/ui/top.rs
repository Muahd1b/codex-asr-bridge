use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, FocusPane};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.focus == FocusPane::Top {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let outer = Block::default()
        .title("Top: Control Panel")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(inner);

    draw_system(frame, cols[0], app);
    draw_daemon(frame, cols[1], app);
    draw_controls(frame, cols[2], app);
}

fn draw_system(frame: &mut Frame, area: Rect, app: &App) {
    let last_target = app
        .last_injected_app
        .clone()
        .unwrap_or_else(|| "<none>".to_string());

    let text = format!(
        "Model: voxtral\nVoxtral procs: {}\nMic: {}\nLang: {}\nLast target: {}",
        app.voxtral_instances(),
        app.profile.mic_device_index,
        app.profile.asr_language,
        last_target
    );

    let widget = Paragraph::new(text)
        .block(Block::default().title("System").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_daemon(frame: &mut Frame, area: Rect, app: &App) {
    let daemon_state = if let Some(ms) = app.daemon_recording_elapsed_ms() {
        format!("RECORDING ({} ms)", ms)
    } else if app.daemon_transcribing {
        "TRANSCRIBING".to_string()
    } else if app.global_ptt_running {
        "IDLE".to_string()
    } else {
        "OFF".to_string()
    };

    let text = format!(
        "Status: {}\nState: {}\nHotkey: RIGHT_SHIFT\nMode: press once start\n      press again stop",
        if app.global_ptt_running { "ON" } else { "OFF" },
        daemon_state
    );

    let widget = Paragraph::new(text)
        .block(Block::default().title("Daemon").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_controls(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "Rewrite: {}\nInject: {}\nKeymap: p rewrite, i inject\n        c cmd-mode, g daemon\n        r reload, v validate\n        Tab pane, q quit",
        app.profile.rewrite_mode.label(),
        app.profile.inject_app.label()
    );

    let widget = Paragraph::new(text)
        .block(Block::default().title("Controls").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(widget, area);
}
