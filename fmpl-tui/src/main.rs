use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fmpl_core::{Vm, eval};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;
use std::time::Duration;

struct App {
    research_content: String,
    planning_content: String,
    code_input: String,
    output: String,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        App {
            research_content: String::from("Research view - Problem space analysis"),
            planning_content: String::from("Planning view - Collaborative scope definition"),
            code_input: String::new(),
            output: String::from("FMPL output will appear here"),
            should_quit: false,
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(c) => {
                self.code_input.push(c);
            }
            KeyCode::Backspace => {
                self.code_input.pop();
            }
            KeyCode::Enter => {
                // Execute FMPL code
                self.execute_code();
            }
            _ => {}
        }
    }

    fn execute_code(&mut self) {
        if self.code_input.trim().is_empty() {
            return;
        }

        // Create VM and execute code
        let mut vm = Vm::new();

        match eval(&mut vm, &self.code_input) {
            Ok(result) => {
                self.output = format!(">>> {}\nResult: {:?}", self.code_input, result);
            }
            Err(e) => {
                self.output = format!(">>> {}\nError: {}", self.code_input, e);
            }
        }

        // Clear input after execution
        self.code_input.clear();
    }
}

fn draw_ui(f: &mut Frame, app: &App) {
    // Main layout: split into horizontal sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(33), // Research
                Constraint::Percentage(33), // Planning
                Constraint::Percentage(34), // Execution (bottom)
            ]
            .as_ref(),
        )
        .split(f.area());

    // Research panel
    let research_panel = Paragraph::new(app.research_content.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Research View")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    // Planning panel
    let planning_panel = Paragraph::new(app.planning_content.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Planning View")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    // Execution panel - split horizontally into code input and output
    let execution_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(50), // Code input
                Constraint::Percentage(50), // Output
            ]
            .as_ref(),
        )
        .split(chunks[2]);

    // Code input panel
    let code_text = Text::from(vec![
        Line::from("FMPL Code (Enter to execute, q to quit):"),
        Line::from(vec![
            Span::raw("> "),
            Span::styled(app.code_input.as_str(), Style::default().fg(Color::Yellow)),
        ]),
    ]);

    let code_panel = Paragraph::new(code_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Code Editor")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    // Output panel
    let output_panel = Paragraph::new(app.output.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Execution Output")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    // Render all panels
    f.render_widget(research_panel, chunks[0]);
    f.render_widget(planning_panel, chunks[1]);
    f.render_widget(code_panel, execution_chunks[0]);
    f.render_widget(output_panel, execution_chunks[1]);
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| draw_ui(f, &app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key.code);
            }
        }

        // Check for quit
        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
