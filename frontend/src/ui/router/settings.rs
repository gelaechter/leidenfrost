pub mod endpoint;

use iced::{
    Element, Font,
    Length::{self, Fill, FillPortion, Shrink},
    Padding, Pixels, Task, Theme, font,
    widget::{self, row},
};

use backend::api::endpoint_api::{EndpointKind, EndpointManager, EndpointType};
use macros::Receiver;

use crate::ui::{
    Receiver,
    components::{icons, style::header_text},
    router::settings::endpoint::{EndpointMsg, EndpointSettings},
};

#[derive(Receiver)]
#[message(Cmd)]
pub struct Settings {
    /// How much the application is zoomed in
    /// TODO: currently unimplemented
    pub zoom_factor: f32,
    /// remnant from early debugging; might be useful to reimplement this and
    /// show metrics, etc.
    pub debug_overlay: bool,
    /// The currently selected theme
    pub theme: Option<Theme>,
    /// The currently selected font
    font_family: font::Family,
    /// Fonts fetched by the system
    available_font_families: Vec<font::Family>,
    /// The configured music endpoints
    pub endpoints: Vec<EndpointSettings>,
    /// The selected settings tab
    pub tab: Route,
}

#[derive(Debug, Clone, Default)]
pub enum Route {
    #[default]
    General,
    LookAndFeel,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Some(Theme::Light),
            debug_overlay: false,
            zoom_factor: 1.0,
            tab: Route::default(),
            font_family: font::Family::SansSerif,
            // All the built in fonts are available by default
            available_font_families: font::Family::VARIANTS.to_vec(),
            endpoints: vec![EndpointSettings {
                id: 0,
                expanded: true,
                selected: false,
                index: true,
                name: "JF".to_string(),
                endpoint_type: Some(EndpointType::Jellyfin),
                endpoint: None,
                url: "http://***REMOVED***".to_string(),
                username: "***REMOVED***".to_string(),
                password: "***REMOVED***".to_string(),
                error: None,
            }],
        }
    }
}

#[derive(Clone, Debug)]
pub enum Cmd {
    Show,
    ScaleChanged(String),
    /// A different theme has been selected
    ThemeChanged(Theme),
    /// A different font family has been selected
    FontChanged(font::Family),
    /// The system has returned all possible fonts
    FontsListed(Vec<font::Family>),
    /// Creates a new endpoint
    CreateEndpoint,
    /// The driver for the endpoint gui
    EndpointDriver(usize, EndpointMsg),
    SyncEndpoints,
    RemoveEndpoint(usize),
}

#[derive(Clone, Debug)]
pub enum Out {
    ThemeChanged(Theme),
}

pub type SettingsMsg = Cmd;

impl Settings {
    pub fn view(&self) -> Element<'_, SettingsMsg> {
        let view = widget::column([self.header(), self.tabs()]);

        widget::sensor(view).on_show(|_| Cmd::Show).into()
    }

    pub fn update(&mut self, message: SettingsMsg) -> Task<SettingsMsg> {
        match message {
            Cmd::Show => {
                if self.available_font_families.len() > font::Family::VARIANTS.len() {
                    return Task::none();
                }

                font::list()
                    .map(|c| match c {
                        Ok(f) => Some(f),
                        Err(e) => {
                            log::warn!("Couldn't get system fonts: {e:?}");
                            None
                        }
                    })
                    .and_then(Task::done)
                    .map(Cmd::FontsListed)
            }
            Cmd::ScaleChanged(_) => todo!(),
            Cmd::ThemeChanged(theme) => {
                self.theme = Some(theme.clone());
                Task::none()
            }
            Cmd::FontChanged(family) => {
                self.font_family = family;
                let font = Font::with_family(family);
                font::set_defaults(
                    font,
                    // Default as specified in [`iced_core::renderer::Settings::default`]
                    Pixels(16.0),
                )
            }
            Cmd::FontsListed(families) => {
                self.available_font_families = families
                    .into_iter()
                    .chain(font::Family::VARIANTS.iter().copied())
                    .collect();

                Task::none()
            }
            Cmd::EndpointDriver(idx, msg) => self.endpoints[idx]
                .update(msg)
                .map(move |m| Cmd::EndpointDriver(idx, m)),
            Cmd::CreateEndpoint => {
                self.endpoints.push(EndpointSettings::default());
                Task::none()
            }
            Cmd::SyncEndpoints => {
                let local_endpoints: Vec<EndpointKind> = self
                    .endpoints
                    .clone()
                    .into_iter()
                    .filter_map(|e| e.selected.then_some(e.endpoint).flatten())
                    .collect();

                let set_endpoint_task: Task<()> = Task::future(async move {
                    log::info!("set_endpoint_task");
                    EndpointManager::update_endpoints(local_endpoints)
                        .await
                        .unwrap();
                })
                .discard();

                let index_task = Task::future(async move {
                    log::info!("index_task");
                    EndpointManager::index_all_endpoints().await.unwrap();
                });

                log::info!("starting both tasks");
                set_endpoint_task.chain(index_task).discard()
            }
            Cmd::RemoveEndpoint(idx) => {
                self.endpoints.remove(idx);
                Task::done(Cmd::SyncEndpoints)
            }
        }
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
            Route::General => self.general(),
            Route::LookAndFeel => self.look_and_feel(),
        })
        .center_x(Fill)
        .padding(12)
        .into()
    }

    /// The general settings tab
    fn general(&self) -> Element<'_, SettingsMsg> {
        widget::column([
            widget::text("General").size(22).into(),
            widget::column([
                self.settings_entry(
                    "Endpoints",
                    "Where Leidenfrost gets its music. Create a new endpoint using the button, then configure and test it. Finally check the checkbox to use it.",
                    {
                        widget::container(
                            widget::column([
                                widget::column(self.endpoints.iter().enumerate().map(
                                    |(idx, e)| {
                                        e.view().map(move |m| Cmd::EndpointDriver(idx, m))
                                    },
                                ))
                                .spacing(6)
                                .into(),
                                widget::button("Add Endpoint")
                                    .on_press(Cmd::CreateEndpoint)
                                    .width(Length::Fill)
                                    .into(),
                            ])
                            .spacing(6),
                        )
                        .width(FillPortion(3))
                    },
                ),
                self.settings_entry(
                    "Theme",
                    "The application theme",
                    widget::pick_list(self.theme.as_ref(), Theme::ALL, Theme::to_string)
                        .on_select(Cmd::ThemeChanged)
                        .width(FillPortion(3)),
                ),
                self.settings_entry(
                    "Font",
                    "The application font",
                    widget::pick_list(
                        Some(self.font_family),
                        self.available_font_families.as_slice(),
                        font::Family::to_string,
                    )
                    .on_select(Cmd::FontChanged)
                    .width(FillPortion(3)),
                ),
            ])
            .spacing(24)
            .into(),
        ])
        .max_width(900)
        .into()
    }

    /// The look and feel settings tab
    fn look_and_feel(&self) -> Element<'_, SettingsMsg> {
        widget::container(widget::column([
            widget::text("Look and Feel").size(22).into(),
            widget::column([todo!()]).spacing(8).into(),
        ]))
        .padding(16)
        .max_width(900)
        .into()
    }

    fn settings_entry<'a>(
        &self,
        name: &'a str,
        description: &'a str,
        control: impl Into<Element<'a, SettingsMsg>>,
    ) -> Element<'a, SettingsMsg> {
        widget::row([
            widget::column([
                widget::text(name)
                    .font(Font::with_family(self.font_family).weight(font::Weight::Semibold))
                    .into(),
                widget::text(description).into(),
            ])
            .width(Length::FillPortion(2))
            .into(),
            control.into(),
        ])
        .padding(32)
        .into()
    }
}
