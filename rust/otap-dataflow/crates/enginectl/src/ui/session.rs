// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Terminal session lifecycle and event-thread helpers for the TUI.

use super::*;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug)]
pub(super) enum UiEvent {
    Terminal(Event),
    TerminalError(String),
    RefreshComplete(Box<AppState>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum EventOutcome {
    Continue,
    Refresh,
    OpenPipelineEditor {
        group_id: String,
        pipeline_id: String,
    },
    Execute(UiAction),
    Quit,
}

pub(super) struct TerminalSession {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    pub(super) fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub(super) fn draw(&mut self, app: &mut AppState) -> Result<(), CliError> {
        let _ = self.terminal.draw(|frame| {
            let area = frame.area();
            app.set_terminal_size(area.width, area.height);
            draw_ui(frame, app);
        })?;
        Ok(())
    }

    pub(super) fn suspend(&mut self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub(super) fn resume(&mut self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = self.terminal.show_cursor();
    }
}

pub(super) fn spawn_event_thread(
    stop: Arc<AtomicBool>,
    tx: mpsc::Sender<UiEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => match event::read() {
                    Ok(event) => match tx.try_send(UiEvent::Terminal(event)) {
                        Ok(()) | Err(TrySendError::Full(_)) => {}
                        Err(TrySendError::Closed(_)) => break,
                    },
                    Err(err) => {
                        let _ = tx.try_send(UiEvent::TerminalError(err.to_string()));
                        break;
                    }
                },
                Ok(false) => {}
                Err(err) => {
                    let _ = tx.try_send(UiEvent::TerminalError(err.to_string()));
                    break;
                }
            }
        }
    })
}

pub(super) fn stop_event_thread(
    stop: &Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
) {
    stop.store(true, Ordering::Relaxed);
    if let Some(handle) = event_thread.take() {
        let _ = handle.join();
    }
}

pub(super) fn restart_event_thread(
    stop: &mut Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
    tx: &mpsc::Sender<UiEvent>,
) {
    *stop = Arc::new(AtomicBool::new(false));
    *event_thread = Some(spawn_event_thread(stop.clone(), tx.clone()));
}
