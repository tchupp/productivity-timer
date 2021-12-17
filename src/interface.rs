// https://monkeypatch.io/blog/2021/2021-05-31-rust-tui/
// https://github.com/ilaborie/plop-tui/blob/blog/step-1/src/app/ui.rs
use crate::database;

use std::convert::TryInto;
use std::io::{stdin, stdout, Error, Read};
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{BarChart, Block, Borders, Paragraph};
use tui::Terminal;

pub fn draw(session_tag: String) -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    // TODO: why do I need the lock?
    let stdin = stdin();
    let stdin = stdin.lock();

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let mut bytes = stdin.bytes();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(80)])
                .split(f.size());

            let overview = database::get_lifetime_overview(&session_tag).unwrap()[0].to_string();
            f.render_widget(draw_overview(overview), chunks[0]);

            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(chunks[1]);

            let mut total_times: Vec<(&str, u64)> = vec![];
            let times = database::get_total_time_as_seconds(&session_tag).unwrap();

            for time in times {
                total_times.push(("", time.total_time.try_into().unwrap()));
            }

            let durations_barchart = draw_barchart(&total_times);
            f.render_widget(durations_barchart, body_chunks[0]);

            let tags = database::get_tags_pane(&session_tag).unwrap();
            f.render_widget(draw_tags(tags), body_chunks[1]);
        })?;

        let b = bytes.next().unwrap().unwrap();

        // quit on q
        if b == b'q' {
            break;
        }
    }

    terminal.clear()?;
    Ok(())
}

fn draw_overview<'a>(overview: String) -> Paragraph<'a> {
    Paragraph::new(overview)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Overview")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        )
}

fn draw_tags<'a>(tags: String) -> Paragraph<'a> {
    Paragraph::new(tags)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        )
}

fn draw_barchart<'a>(data: &'a [(&'a str, u64)]) -> BarChart<'a> {
    BarChart::default()
        .block(Block::default().title("Durations").borders(Borders::ALL))
        .data(data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::LightCyan))
        .value_style(Style::default().fg(Color::Black).bg(Color::LightCyan))
}
