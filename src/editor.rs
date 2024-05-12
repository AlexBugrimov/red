use std::io::{stdout, Stdout, Write};
use crossterm::{cursor, event, event::read, ExecutableCommand, QueueableCommand, style, terminal};

enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    EnterMode(Mode),

    AddChar(char),
    NewLine,
}

enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    size: (u16, u16),
    cx: u16,
    cy: u16,
    stdout: Stdout,
    mode: Mode,
}

impl Drop for Editor {

    fn drop(&mut self) {
        _ = self.stdout.flush();
        _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        _ = terminal::disable_raw_mode();
    }
}

impl Editor {

    pub fn new() -> anyhow::Result<Self> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(Editor {
            size: terminal::size()?,
            cx: 0,
            cy: 0,
            stdout,
            mode: Mode::Normal,
        })
    }

    pub fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_status_line()?;
        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn draw_status_line(&mut self) -> anyhow::Result<()> {
        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(style::Print("Status line"))?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.draw()?;
            if let Some(action) = self.handle_event(read()?)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.cy = self.cy.saturating_sub(1)
                    }
                    Action::MoveDown => {
                        self.cy += 1;
                    }
                    Action::MoveLeft => {
                        self.cx = self.cx.saturating_sub(1)
                    }
                    Action::MoveRight => {
                        self.cx += 1;
                    }
                    Action::EnterMode(new_mode) => {
                        self.mode = new_mode
                    }
                    Action::AddChar(c) => {
                        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
                        self.stdout.queue(style::Print(c))?;
                        self.cx += 1;
                    }
                    Action::NewLine => {
                        self.cx = 0;
                        self.cy += 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        match self.mode {
            Mode::Normal => self.handle_normal_event(ev),
            Mode::Insert => self.handle_insert_event(ev)
        }
    }

    fn handle_normal_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => Some(Action::Quit),
                event::KeyCode::Up | event::KeyCode::Char('k') => Some(Action::MoveUp),
                event::KeyCode::Down | event::KeyCode::Char('j') => Some(Action::MoveDown),
                event::KeyCode::Left | event::KeyCode::Char('h') => Some(Action::MoveLeft),
                event::KeyCode::Right | event::KeyCode::Char('l') => Some(Action::MoveRight),
                event::KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
                _ => None
            }
            _ => None
        };
        Ok(action)
    }

    fn handle_insert_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
                event::KeyCode::Enter => Some(Action::NewLine),
                event::KeyCode::Char(c) => Some(Action::AddChar(c)),
                _ => None,
            }
            _ => None
        };
        Ok(action)
    }
}

