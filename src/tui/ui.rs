//! Vibrant TUI rendering with rich colors, animations, spinners, and badges.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use cargo_scrub::report::format_size;

use crate::tui::app::{App, CrateStatus, Screen};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(6),    // Main Body (Table or State Screen)
            Constraint::Length(2), // Status bar & hints
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
    let spinner_span = if matches!(app.screen, Screen::Scanning | Screen::Running) {
        Span::styled(
            format!(" {} ", app.spinner()),
            Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " 🧹 ",
            Style::new().fg(Color::Cyan),
        )
    };

    let title_badge = Span::styled(
        " cargo-scrub ",
        Style::new()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let path_span = Span::styled(
        format!(" {} ", app.options.root.display()),
        Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
    );

    let dry_run_badge = if app.options.dry_run {
        Span::styled(
            " DRY-RUN ",
            Style::new()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let mode_badge = Span::styled(
        format!(" {:?} ", app.options.workspace_mode),
        Style::new()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    );

    let stats_spans: Vec<Span> = match app.screen {
        Screen::Scanning => vec![
            Span::styled(" [Scanning] ", Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("discovered {} project(s)...", app.scan_visited),
                Style::new().fg(Color::LightYellow),
            ),
        ],
        Screen::Empty => vec![Span::styled(
            " [No Projects Found] ",
            Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        )],
        Screen::Review => {
            let count = app.crates.len();
            let selected = app.selected_count();
            let size = format_size(app.reclaimable_bytes());
            vec![
                Span::styled(format!(" {} ", count), Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("crates  ", Style::new().fg(Color::DarkGray)),
                Span::styled(format!(" {} ", selected), Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("selected  ", Style::new().fg(Color::DarkGray)),
                Span::styled(format!(" {} ", size), Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("reclaimable  ", Style::new().fg(Color::DarkGray)),
                Span::styled(format!(" {} jobs ", app.options.jobs), Style::new().fg(Color::Blue)),
            ]
        }
        Screen::Running => {
            let elapsed = app
                .elapsed()
                .map(|d| format!("{:.1}s", d.as_secs_f64()))
                .unwrap_or_else(|| "0.0s".to_string());
            let cleaned = app.cleaned_count();
            let active = app.active_count();
            let total = app.selected_count();
            vec![
                Span::styled(" [Cleaning] ", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}/{} done", cleaned, total), Style::new().fg(Color::LightGreen)),
                Span::raw(" | "),
                Span::styled(format!("{} active", active), Style::new().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(elapsed, Style::new().fg(Color::LightYellow)),
            ]
        }
        Screen::Summary => {
            if let Some(ref report) = app.summary {
                vec![
                    Span::styled(" [Done] ", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} cleaned", report.cleaned), Style::new().fg(Color::Green)),
                    Span::raw("  "),
                    Span::styled(format!("{} skipped", report.skipped), Style::new().fg(Color::Yellow)),
                    Span::raw("  "),
                    Span::styled(format!("{} errors", report.errors), Style::new().fg(if report.errors > 0 { Color::Red } else { Color::DarkGray })),
                    Span::raw("  "),
                    Span::styled(format!("Reclaimed {}", format_size(app.reclaimed_bytes)), Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                ]
            } else {
                vec![]
            }
        }
    };

    let mut line_spans = vec![
        spinner_span,
        title_badge,
        Span::raw(" "),
        path_span,
        Span::raw(" "),
        mode_badge,
        Span::raw(" "),
    ];

    if app.options.dry_run {
        line_spans.push(dry_run_badge);
        line_spans.push(Span::raw(" "));
    }

    line_spans.push(Span::raw(" │ "));
    line_spans.extend(stats_spans);

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Cyan));

    let header_paragraph = Paragraph::new(Line::from(line_spans)).block(header_block);
    f.render_widget(header_paragraph, area);
}

fn draw_body(f: &mut Frame, app: &mut App, area: Rect) {
    match app.screen {
        Screen::Scanning => {
            let inner_area = centered_rect(60, 40, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::LightCyan))
                .title(Span::styled(" Scanning Filesystem ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

            let text = Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!(" {} ", app.spinner()), Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    Span::styled("Searching for Rust cargo crates...", Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Directory: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(app.options.root.display().to_string(), Style::new().fg(Color::LightYellow)),
                ]),
                Line::from(vec![
                    Span::styled("  Projects found: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(format!("{}", app.scan_visited), Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(Span::styled("  Press 'q' to abort scan", Style::new().fg(Color::Gray))),
            ]);

            let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);
            f.render_widget(paragraph, inner_area);
        }
        Screen::Empty => {
            let inner_area = centered_rect(60, 35, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Yellow))
                .title(Span::styled(" No Crates Found ", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("🔍 No Rust projects matched your search criteria.", Style::new().fg(Color::White))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Target Root: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(app.options.root.display().to_string(), Style::new().fg(Color::LightYellow)),
                ]),
                Line::from(""),
                Line::from(Span::styled("Press 'q' or 'Esc' to exit.", Style::new().fg(Color::Cyan))),
            ]);

            let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);
            f.render_widget(paragraph, inner_area);
        }
        Screen::Review => draw_table(f, app, area),
        Screen::Summary => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(7), Constraint::Min(4)])
                .split(area);

            let report = app.summary.as_ref();
            let cleaned = report.map(|r| r.cleaned).unwrap_or(0);
            let skipped = report.map(|r| r.skipped).unwrap_or(0);
            let errors = report.map(|r| r.errors).unwrap_or(0);
            let duration = report.map(|r| r.duration).unwrap_or_default();

            let summary_card = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::LightGreen))
                .title(Span::styled(" 🎉 Scrub Complete - Summary ", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));

            let stats_text = Text::from(vec![
                Line::from(vec![
                    Span::styled("  Total Cleaned: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(format!("{} projects", cleaned), Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("   │   Skipped: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(format!("{}", skipped), Style::new().fg(Color::Yellow)),
                    Span::styled("   │   Errors: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(format!("{}", errors), Style::new().fg(if errors > 0 { Color::Red } else { Color::Green })),
                ]),
                Line::from(vec![
                    Span::styled("  Reclaimed Disk Space: ", Style::new().fg(Color::DarkGray)),
                    Span::styled(format_size(app.reclaimed_bytes), Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  (in {:.2}s)", duration.as_secs_f64()), Style::new().fg(Color::Gray)),
                ]),
                Line::from(""),
                Line::from(Span::styled("  Press 'q' or 'Esc' to exit, or scroll down to review crate details.", Style::new().fg(Color::Gray))),
            ]);

            let paragraph = Paragraph::new(stats_text).block(summary_card);
            f.render_widget(paragraph, chunks[0]);

            draw_table(f, app, chunks[1]);
        }
        Screen::Running => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(4)])
                .split(area);

            let selected = app.selected_count();
            let done = app.cleaned_count();
            let ratio = if selected > 0 {
                (done as f64 / selected as f64).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let gauge_title = format!(" Cleaning Progress: {} of {} crates ({:.0}%) ", done, selected, ratio * 100.0);
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Green))
                        .title(Span::styled(gauge_title, Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD))),
                )
                .gauge_style(Style::new().fg(Color::Green).bg(Color::DarkGray))
                .ratio(ratio);
            f.render_widget(gauge, chunks[0]);

            draw_table(f, app, chunks[1]);
        }
    }
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let visible = app.visible_indices();
    let header_cells = ["", "SEL", "PROJECT PATH", "TARGET SIZE", "TYPE", "STATUS"]
        .into_iter()
        .map(|h| Cell::from(Span::styled(h, Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD))));
    let header = Row::new(header_cells)
        .style(Style::new().bg(Color::Rgb(20, 25, 35)))
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = visible
        .iter()
        .map(|&idx| {
            let row = &app.crates[idx];
            let (sel_icon, sel_style) = if row.info.selected {
                (" ● ", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD))
            } else {
                (" ○ ", Style::new().fg(Color::DarkGray))
            };

            let (kind_badge, kind_style) = if row.info.is_workspace_root {
                ("WORKSPACE", Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            } else {
                ("CRATE", Style::new().fg(Color::DarkGray))
            };

            let (status_text, status_style) = match &row.status {
                CrateStatus::Pending => {
                    if row.info.selected {
                        ("⏳ ready", Style::new().fg(Color::LightBlue))
                    } else {
                        ("— idle", Style::new().fg(Color::DarkGray))
                    }
                }
                CrateStatus::Cleaning => {
                    (format!("{} cleaning", app.spinner()).leak() as &str, Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
                }
                CrateStatus::Done => ("✓ cleaned", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
                CrateStatus::Skipped => ("⊘ skipped", Style::new().fg(Color::Yellow)),
                CrateStatus::Error(_) => ("✗ failed", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
            };

            let size_style = if row.info.target_size > 500 * 1024 * 1024 {
                Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD)
            } else if row.info.target_size > 50 * 1024 * 1024 {
                Style::new().fg(Color::LightYellow)
            } else {
                Style::new().fg(Color::Green)
            };

            let marker = if row.info.selected { "▌" } else { " " };
            let marker_style = if row.info.selected { Style::new().fg(Color::LightGreen) } else { Style::new() };

            Row::new(vec![
                Cell::from(Span::styled(marker, marker_style)),
                Cell::from(Span::styled(sel_icon, sel_style)),
                Cell::from(Span::styled(truncate_path(&row.info.path.display().to_string(), 50), Style::new().fg(Color::White))),
                Cell::from(Span::styled(format_size(row.info.target_size), size_style)),
                Cell::from(Span::styled(kind_badge, kind_style)),
                Cell::from(Span::styled(status_text, status_style)),
            ])
            .height(1)
        })
        .collect();

    let (title, border_color) = match app.screen {
        Screen::Review => (" Crate Discovery & Selection (Space: toggle, a: select all, c: clean) ", Color::Cyan),
        Screen::Running => (" Active Scrub Tasks ", Color::LightGreen),
        Screen::Summary => (" Summary of Cleaned Crates ", Color::Magenta),
        _ => (" Crates ", Color::Gray),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(25),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(border_color))
            .title(Span::styled(title, Style::new().fg(border_color).add_modifier(Modifier::BOLD))),
    )
    .row_highlight_style(
        Style::new()
            .bg(Color::Rgb(40, 50, 75))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

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
    let key_badge = |k: &'static str, desc: &'static str| -> Vec<Span> {
        vec![
            Span::styled(format!(" {} ", k), Style::new().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", desc), Style::new().fg(Color::White)),
            Span::raw(" "),
        ]
    };

    let key_hints: Vec<Span> = match app.screen {
        Screen::Scanning | Screen::Empty => key_badge("q", "Quit"),
        Screen::Review => {
            let mut v = Vec::new();
            v.extend(key_badge("Up/Down", "Navigate"));
            v.extend(key_badge("Space", "Toggle"));
            v.extend(key_badge("a", "All"));
            v.extend(key_badge("A", "None"));
            v.extend(key_badge("/", "Filter"));
            v.extend(key_badge("d", "Dry-run"));
            v.extend(key_badge("c / Enter", "Start Clean"));
            v.extend(key_badge("?", "Help"));
            v.extend(key_badge("q", "Quit"));
            v
        }
        Screen::Running => {
            let mut v = Vec::new();
            v.extend(key_badge("q / Esc", "Cancel/Quit"));
            v.extend(key_badge("?", "Help"));
            v
        }
        Screen::Summary => {
            let mut v = Vec::new();
            v.extend(key_badge("Up/Down", "Scroll"));
            v.extend(key_badge("q / Esc", "Exit"));
            v.extend(key_badge("?", "Help"));
            v
        }
    };

    let status_text = if app.status_line.is_empty() {
        "Ready"
    } else {
        &app.status_line
    };

    let mut footer_spans = vec![
        Span::styled(" STATUS: ", Style::new().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(truncate_path(status_text, 45), Style::new().fg(Color::White)),
        Span::raw("  │  "),
    ];
    footer_spans.extend(key_hints);

    let block = Block::default().borders(Borders::NONE);
    let paragraph = Paragraph::new(Line::from(footer_spans)).block(block);
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(65, 65, f.area());
    f.render_widget(Clear, area);

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(" ✨ cargo-scrub Keyboard Shortcuts ✨ ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation & Selection:", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Up / Down, j / k   Move selection up / down"),
        Line::from("  g / G              Jump directly to top / bottom"),
        Line::from("  Space              Toggle selection on focused project"),
        Line::from("  a / A              Select all / Deselect all projects"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions & Filtering:", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  /                  Live path filter (Enter to apply, Esc to cancel)"),
        Line::from("  d                  Toggle Dry-Run mode on/off"),
        Line::from("  c / Enter          Start parallel cleaning of selected projects"),
        Line::from(""),
        Line::from(vec![
            Span::styled("General:", Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ?                  Toggle this help popup"),
        Line::from("  q / Esc            Quit cargo-scrub"),
        Line::from(""),
        Line::from(Span::styled("Press '?' or 'Esc' to dismiss help", Style::new().fg(Color::DarkGray))),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(Span::styled(" Help & Controls ", Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD)))
        .style(Style::new().bg(Color::Rgb(15, 18, 28)).fg(Color::White));

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_quit_confirm(f: &mut Frame) {
    let area = centered_rect(50, 25, f.area());
    f.render_widget(Clear, area);

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled("⚠️  Cleaning is currently in progress!", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Active cargo clean jobs are running in background tasks."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Quit anyway? [", Style::new().fg(Color::White)),
            Span::styled("y", Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled("es / ", Style::new().fg(Color::White)),
            Span::styled("N", Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            Span::styled("o]", Style::new().fg(Color::White)),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(Span::styled(" Confirm Abort ", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)))
        .style(Style::new().bg(Color::Rgb(25, 15, 15)).fg(Color::White));

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn draw_filter_input(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 22, f.area());
    f.render_widget(Clear, area);

    let cursor = if (app.tick_count / 10) % 2 == 0 { "█" } else { " " };
    let input_line = format!("🔎 Filter query: {}{}", app.filter_input, cursor);

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(input_line, Style::new().fg(Color::White).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Press Enter to apply filter, Esc to cancel", Style::new().fg(Color::DarkGray))),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(" Filter by Path Substring ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .style(Style::new().bg(Color::Rgb(15, 25, 35)).fg(Color::White));

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);
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
