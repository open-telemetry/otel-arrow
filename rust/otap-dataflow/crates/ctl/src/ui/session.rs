// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Terminal session lifecycle and event-thread helpers for the TUI.
//!
//! Session handling owns raw-terminal setup, teardown, event polling, and
//! temporary suspension when external tools such as editors run. This separation
//! keeps terminal-mode safety and cross-thread event delivery out of the
//! business logic that decides what the UI should display or mutate.

use super::*;
use tokio::sync::mpsc::error::TrySendError;

/// Cross-thread event sent from terminal input and refresh tasks to the UI loop.
#[derive(Debug)]
pub(super) enum UiEvent {
    Terminal(Event),
    TerminalError(String),
    RefreshComplete(Box<AppState>),
}

/// Result of handling one terminal event.
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

/// Owns terminal raw mode, alternate screen state, and ratatui drawing.
pub(super) struct TerminalSession {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    /// Enters raw-mode alternate-screen rendering for the TUI.
    pub(super) fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Draws one TUI frame and records the frame size in app state.
    pub(super) fn draw(&mut self, app: &mut AppState) -> Result<(), CliError> {
        let _ = self.terminal.draw(|frame| {
            let area = frame.area();
            app.set_terminal_size(area.width, area.height);
            draw_ui(frame, app);
        })?;
        Ok(())
    }

    /// Temporarily restores the terminal before launching an external editor.
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

    /// Re-enters the alternate screen after an external editor exits.
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

/// Spawns the blocking crossterm event reader used by the async UI loop.
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

/// Stops and joins the crossterm event reader thread.
pub(super) fn stop_event_thread(
    stop: &Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
) {
    stop.store(true, Ordering::Relaxed);
    if let Some(handle) = event_thread.take() {
        let _ = handle.join();
    }
}

/// Restarts terminal event reading after a temporary terminal suspension.
pub(super) fn restart_event_thread(
    stop: &mut Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
    tx: &mpsc::Sender<UiEvent>,
) {
    *stop = Arc::new(AtomicBool::new(false));
    *event_thread = Some(spawn_event_thread(stop.clone(), tx.clone()));
}
