//! TUI rendering.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use cargo_scrub::report::format_size;

use crate::tui::app::{App, CrateStatus, Screen};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_body(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_help {
        draw_help(f);
    }

    if app.quit_confirm {
        draw_quit_confirm(f);
    }

    if app.filter_active {
        draw_filter_input(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let dry_run = if app.options.dry_run {
        Span::styled(" DRY-RUN ", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let mode = Span::styled(
        format!(" {:?} ", app.options.workspace_mode),
        Style::new().fg(Color::Cyan),
    );

    let stats = match app.screen {
        Screen::Scanning => format!("scanning... visited {}", app.scan_visited),
        Screen::Empty => "no crates found".to_string(),
        Screen::Review => format!(
            "{} crates | {} selected | {} reclaimable | {} jobs",
            app.crates.len(),
            app.selected_count(),
            format_size(app.reclaimable_bytes()),
            app.options.jobs
        ),
        Screen::Running => {
            let elapsed = app
                .elapsed()
                .map(|d| format!("{:.1}s", d.as_secs_f64()))
                .unwrap_or_else(|| "0.0s".to_string());
            format!("cleaning... {} | {}", elapsed, app.status_line)
        }
        Screen::Summary => {
            if let Some(ref report) = app.summary {
                format!(
                    "cleaned {} | skipped {} | errors {} | reclaimed {}",
                    report.cleaned,
                    report.skipped,
                    report.errors,
                    format_size(app.reclaimed_bytes)
                )
            } else {
                String::new()
            }
        }
    };

    let title = Line::from(vec![
        Span::styled(" cargo-scrub ", Style::new().add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(app.options.root.display().to_string(), Style::new().fg(Color::White)),
        dry_run,
        mode,
        Span::raw(" "),
        Span::styled(stats, Style::new().fg(Color::Gray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::DarkGray));
    let paragraph = Paragraph::new(title).block(block);
    f.render_widget(paragraph, area);
}

fn draw_body(f: &mut Frame, app: &mut App, area: Rect) {
    match app.screen {
        Screen::Scanning => {
            let paragraph = Paragraph::new(Text::from(vec![
                Line::from("Scanning directory tree for Rust projects..."),
                Line::from(""),
                Line::from(Span::styled(
                    "Press q to quit",
                    Style::new().fg(Color::DarkGray),
                )),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Scanning "));
            f.render_widget(paragraph, area);
        }
        Screen::Empty => {
            let paragraph = Paragraph::new(Text::from(vec![
                Line::from("No Rust projects matched the current filters."),
                Line::from(""),
                Line::from(Span::styled("Press q to quit", Style::new().fg(Color::DarkGray))),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Empty "));
            f.render_widget(paragraph, area);
        }
        Screen::Review | Screen::Running | Screen::Summary => draw_table(f, app, area),
    }
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let visible = app.visible_indices();
    let header = Row::new(vec!["Sel", "Path", "Size", "Kind", "Status"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = visible
        .iter()
        .map(|&idx| {
            let row = &app.crates[idx];
            let sel = if row.info.selected { "[x]" } else { "[ ]" };
            let kind = if row.info.is_workspace_root {
                "workspace"
            } else {
                ""
            };
            let status = match &row.status {
                CrateStatus::Pending => "pending".gray(),
                CrateStatus::Cleaning => "cleaning".cyan(),
                CrateStatus::Done => "done".green(),
                CrateStatus::Skipped => "skipped".yellow(),
                CrateStatus::Error(_) => "error".red(),
            };
            Row::new(vec![
                Cell::from(sel),
                Cell::from(truncate_path(&row.info.path.display().to_string(), 48)),
                Cell::from(format_size(row.info.target_size)),
                Cell::from(kind),
                Cell::from(status),
            ])
        })
        .collect();

    let title = match app.screen {
        Screen::Review => " Review ",
        Screen::Running => " Running ",
        Screen::Summary => " Summary ",
        _ => " Crates ",
    };

    let table = Table::new(rows, [
        Constraint::Length(4),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED));

    let mut state = app.table_state.clone();
    if let Some(sel) = state.selected() {
        if !visible.contains(&sel) {
            state.select(visible.first().copied());
        }
    } else if let Some(&first) = visible.first() {
        state.select(Some(first));
    }

    f.render_stateful_widget(table, area, &mut state);
    app.table_state = state;
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let hints = match app.screen {
        Screen::Scanning | Screen::Empty => "q quit",
        Screen::Review => {
            "↑↓/jk move  Space toggle  a all  A none  / filter  d dry-run  c clean  ? help  q quit"
        }
        Screen::Running => "q quit (confirm)  ? help",
        Screen::Summary => "↑↓ scroll  q quit  ? help",
    };

    let status = if app.status_line.is_empty() {
        hints.to_string()
    } else {
        format!("{} | {}", truncate_path(&app.status_line, 60), hints)
    };

    let paragraph = Paragraph::new(status).style(Style::new().fg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);

    let text = Text::from(vec![
        Line::from(Span::styled("Help", Style::new().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Review screen:"),
        Line::from("  ↑/↓, j/k     Move selection"),
        Line::from("  g / G        Jump to top / bottom"),
        Line::from("  Space        Toggle crate selection"),
        Line::from("  a / A        Select all / none"),
        Line::from("  /            Filter by path substring"),
        Line::from("  d            Toggle dry-run mode"),
        Line::from("  c / Enter    Start cleaning selected crates"),
        Line::from("  ?            Toggle this help"),
        Line::from("  q / Esc      Quit"),
        Line::from(""),
        Line::from("Running: q asks for confirmation before quitting."),
        Line::from(""),
        Line::from("Press ? or Esc to close"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::new().bg(Color::Black));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_quit_confirm(f: &mut Frame) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    let paragraph = Paragraph::new(Text::from(vec![
        Line::from("Cleaning is still running."),
        Line::from(""),
        Line::from("Quit anyway?  y/N"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Quit ")
            .style(Style::new().fg(Color::Yellow)),
    );
    f.render_widget(paragraph, area);
}

fn draw_filter_input(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    let paragraph = Paragraph::new(format!("Filter path: {}", app.filter_input))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Filter ")
                .style(Style::new().fg(Color::Cyan)),
        );
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate_path(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len() - max + 3..])
    }
}
