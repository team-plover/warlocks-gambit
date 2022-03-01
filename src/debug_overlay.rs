//! Overlay display for debugging
use std::sync::{Arc, RwLock};

use bevy::prelude::{Plugin as BevyPlugin, *};
use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DBG_TEXT: Arc<DebugText> = Arc::new(DebugText::default());
}

/// Display text in top left corner of screen, for `timeout` seconds with the
/// file name and line number where the macro was called.
///
/// If `timeout == 0`, only display it for a single frame. If timeout is not
/// specified, the text lingers for 7 seconds.
///
/// # Usage
///
/// ```rust,ignore
/// add_dbg_text!("Debug text");
/// add_dbg_text!("Debug text", timeout);
/// ```
#[macro_export]
macro_rules! add_dbg_text {
    ($text:expr) => {
        add_dbg_text!($text, 7.0)
    };
    ($text:expr, $timeout:expr) => {{
        use $crate::debug_overlay::{DebugTextKey, DBG_TEXT};
        let key = DebugTextKey { file: file!(), line: line!(), column: column!() };
        DBG_TEXT.add_text(key, ($text).to_string(), $timeout)
    }};
}

struct DebugLine {
    text: String,
    expiration_time: f64,
}

#[derive(Hash, PartialEq, Eq)]
pub struct DebugTextKey {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

/// Queue text to display on the screen
#[derive(Default)]
pub struct DebugText {
    lines: DashMap<DebugTextKey, DebugLine>,
    last_timestamp: RwLock<f64>,
}
impl DebugText {
    /// Display text in top left corner of screen, for `timeout` seconds
    ///
    /// If `timeout == 0`, only display it for a single frame
    pub fn add_text(&self, key: DebugTextKey, text: String, timeout: f64) {
        let last_timestamp = self.last_timestamp.read().unwrap();
        let line = DebugLine { text, expiration_time: timeout + *last_timestamp };
        self.lines.insert(key, line);
    }
}

#[derive(Component)]
struct DebugTextEntity;

fn debug_align() -> TextAlignment {
    TextAlignment {
        horizontal: HorizontalAlign::Left,
        ..Default::default()
    }
}
fn debug_style(asset_server: &AssetServer) -> TextStyle {
    TextStyle {
        color: Color::YELLOW,
        font: asset_server.load("Boogaloo-Regular.otf"),
        font_size: 13.0,
    }
}
fn debug_overlay_setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let position = Rect {
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        ..Default::default()
    };
    cmds.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position,
            ..Default::default()
        },
        text: Text::with_section("", debug_style(&asset_server), debug_align()),
        ..Default::default()
    })
    .insert(DebugTextEntity);
}

fn update_debug_overlay(mut debug_texts: Query<&mut Text, With<DebugTextEntity>>, time: Res<Time>) {
    let texts = &DBG_TEXT;
    let mut to_modify = debug_texts.get_single_mut().unwrap();
    let last_timestamp = time.seconds_since_startup();
    to_modify.sections[0].value = texts
        .lines
        .iter()
        .filter(|line| line.value().expiration_time > last_timestamp)
        .map(|kv| {
            format!(
                "[{}:{}:{}] {}\n",
                kv.key().file,
                kv.key().line,
                kv.key().column,
                kv.value().text
            )
        })
        .fold("".to_string(), |a, b| a + &b);
    let mut timestamp = texts.last_timestamp.write().unwrap();
    *timestamp = last_timestamp;
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(debug_overlay_setup)
            .add_system(update_debug_overlay);
    }
}
