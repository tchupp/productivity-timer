// https://monkeypatch.io/blog/2021/2021-05-31-rust-tui/

use std::io::{stdin, stdout, Error, Read};
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;

pub fn hello_world() -> Result<(), Error> {
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
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ])
                .split(f.size());

            let title = draw_title();
            f.render_widget(title, chunks[0]);

            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        })?;

        let b = bytes.next().unwrap().unwrap();
        // quit on q
        if b == b'q' {
            break;
        }
    }
    Ok(())
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Hello, world")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        )
}
