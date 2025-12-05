//! Event processing from backend

use super::SlircApp;
use crate::events;
use crate::protocol::GuiEvent;
use crate::ui::dialogs::ChannelListItem;

impl SlircApp {
    pub fn process_events(&mut self) {
        // Collect channel list events separately
        let mut regular_events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                GuiEvent::ChannelListItem {
                    channel,
                    user_count,
                    topic,
                } => {
                    // Add to channel browser dialog if open
                    self.dialogs.add_channel_to_browser(ChannelListItem {
                        channel,
                        user_count,
                        topic,
                    });
                }
                GuiEvent::ChannelListEnd => {
                    // Mark loading complete and show dialog
                    self.dialogs.channel_browser_complete();
                }
                other => {
                    regular_events.push(other);
                }
            }
        }

        // Process regular events
        for event in regular_events {
            self.process_single_event(event);
        }
    }

    fn process_single_event(&mut self, event: GuiEvent) {
        // Process event and check if nick changed
        if let Some(new_nick) = events::process_single_event(&mut self.state, event) {
            // Update UI nickname field when server confirms nick change
            self.connection.nickname = new_nick;
        }
    }
}
