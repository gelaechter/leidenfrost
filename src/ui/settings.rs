use std::rc::Rc;

use iced::{
    Element,
    Length::{Fill, Shrink},
    Padding, Task, Theme,
    widget::{self, row},
};

use crate::{
    backend::api::endpoint_api::Endpoint,
    ui::{ICMsg, ToOutMsg, components::{icons, style::header_text}},
};

pub struct Settings {
    pub selected_apis: Vec<Endpoint>,
    pub zoom_factor: f32,
    pub debug_overlay: bool,
    pub theme: Option<Theme>,
    pub tab: Route,
}

#[derive(Debug, Clone, Default)]
pub enum Route {
    #[default]
    LookAndFeel,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_apis: Default::default(),
            theme: Some(Theme::Light),
            debug_overlay: Default::default(),
            zoom_factor: 1.0,
            tab: Route::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Cmd {
    ScaleChanged(String),
    ThemeChanged(Theme),
}

#[derive(Clone, Debug)]
pub enum Out {
    ThemeChanged(Theme),
}

pub type SettingsMsg = ICMsg<Cmd, Out>;

impl Settings {
    pub fn view(&self) -> Element<'_, SettingsMsg> {
        widget::column([self.header(), self.tabs()]).into()
    }

    pub fn update(&mut self, message: SettingsMsg) -> Task<SettingsMsg> {
        message.cmd(|c| match c {
            Cmd::ScaleChanged(_) => todo!(),
            Cmd::ThemeChanged(theme) => {
                self.theme = Some(theme.clone());
                // Emit event
                Task::done(Out::ThemeChanged(theme).out_msg())
            }
        })
    }

    pub fn header(&self) -> Element<'_, SettingsMsg> {
        let settings_icon = icons::settings().size(36);

        let header = widget::container(row![
            settings_icon,
            widget::space().width(12),
            widget::container(header_text("Settings")).center_y(Fill),
        ])
        .height(Shrink)
        .padding(Padding::new(12.0));

        header.into()
    }

    pub fn tabs(&self) -> Element<'_, SettingsMsg> {
        widget::container(match self.tab {
            Route::LookAndFeel => self.look_and_feel(),
        })
        .center_x(Fill)
        .into()
    }

    fn look_and_feel(&self) -> Element<'_, SettingsMsg> {
        widget::container(widget::column([
            widget::text("Look and Feel").size(22).into(),
            widget::column([Self::settings_entry(
                "Theme",
                "The application theme",
                widget::pick_list(self.theme.as_ref(), Theme::ALL, Theme::to_string)
                    .on_select(|s| Cmd::ThemeChanged(s).into()),
            )])
            .into(),
        ]))
        .padding(16)
        .max_width(900)
        .into()
    }

    fn settings_entry<'a>(
        name: &'a str,
        description: &'a str,
        control: impl Into<Element<'a, SettingsMsg>>,
    ) -> Element<'a, SettingsMsg> {
        widget::row([
            widget::column([widget::text(name).into(), widget::text(description).into()]).into(),
            widget::space().width(Fill).into(),
            control.into(),
        ])
        .into()
    }
}
