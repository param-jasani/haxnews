pub mod app;
pub mod ui;
pub mod handler;

pub use app::App;
pub use handler::handle_events;

use anyhow::Result;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use crate::tui::ui::draw;

pub async fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize image picker (query terminal for graphics capability and font size)
    let mut picker = match ratatui_image::picker::Picker::from_query_stdio() {
        Ok(picker) => picker,
        Err(_) => ratatui_image::picker::Picker::halfblocks(),
    };

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, &mut picker).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App, picker: &mut ratatui_image::picker::Picker) -> Result<()> {
    loop {
        // Poll for image fetch results
        if let Ok(img) = app.image_rx.try_recv() {
            if let Ok(dyn_img) = image::load_from_memory(&img) {
                app.current_image = Some(picker.new_resize_protocol(dyn_img));
            }
        }

        terminal.draw(|f| {
            draw(f, app);
        })?;

        if !app.running {
            break;
        }

        handle_events(app)?;
        app.process_pending_action().await;
    }

    Ok(())
}
