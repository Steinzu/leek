use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, LineGauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, FileType};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(4), // Footer (Progress + Volume)
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title_text = format!("LEEK 🎵 - {}", app.current_directory.to_string_lossy());
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ) // Miku Cyan
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue)),
        ) // Miku Blue
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(title, area);
}

fn draw_main(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Browser
            Constraint::Percentage(40), // Info
        ])
        .split(area);

    draw_browser(f, app, chunks[0]);
    draw_info(f, app, chunks[1]);
}

fn draw_browser(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .browser_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let icon = match item.file_type {
                FileType::Directory => "📁 ",
                FileType::AudioFile => "🎵 ",
                FileType::Other => "📄 ",
            };

            let style = if i == app.browser_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(icon, style),
                Span::styled(item.name.clone(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.browser_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" File Browser ")
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_info(f: &mut Frame, app: &App, area: Rect) {
    let current_song = if !app.queue.is_empty() && app.queue_index < app.queue.len() {
        app.queue[app.queue_index]
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    } else {
        "No song playing".into()
    };

    let status = if app.is_playing {
        "Playing ▶"
    } else {
        "Paused ⏸"
    };
    // let _vol = app.volume; // unused

    let info_text = vec![
        Line::from(vec![Span::styled(
            "Now Playing:",
            Style::default().fg(Color::LightBlue),
        )]),
        Line::from(vec![Span::styled(
            current_song,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Status: {} ", status),
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "Queue Position: {}/{}",
                app.queue_index + 1,
                app.queue.len()
            ),
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Controls:",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(Span::styled(
            "Enter: Enter Dir / Play File",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Tab: Play Whole Folder",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "Backspace: Go Up",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Space: Play/Pause",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Left/Right: Prev/Next Track",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "PgUp/PgDn: Volume",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Info ")
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(info, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Progress
            Constraint::Length(1), // Volume
        ])
        .split(area);

    // Progress Bar
    let (elapsed_sec, duration_sec, ratio) = if let Some(d) = app.duration {
        let e = app.elapsed.as_secs_f64();
        let t = d.as_secs_f64();
        (e, t, (e / t).min(1.0))
    } else {
        (0.0, 0.0, 0.0)
    };

    let label = format!(
        "{:02}:{:02} / {:02}:{:02}",
        (elapsed_sec / 60.0) as u64,
        (elapsed_sec % 60.0) as u64,
        (duration_sec / 60.0) as u64,
        (duration_sec % 60.0) as u64
    );

    let progress = LineGauge::default()
        .block(Block::default())
        .filled_style(Style::default().fg(Color::Cyan))
        .unfilled_style(Style::default().fg(Color::DarkGray))
        .filled_symbol("▬")
        .ratio(ratio)
        .label(Line::from(Span::styled(
            label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));

    f.render_widget(progress, chunks[0]);

    // Volume Bar (Slim)
    let volume_ratio = app.volume as f64 / 100.0;
    let vol_label = format!("VOL: {}%", app.volume);

    let vol_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::LightBlue).bg(Color::Black))
        .ratio(volume_ratio)
        .label(vol_label)
        .use_unicode(true);

    f.render_widget(vol_gauge, chunks[1]);
}
