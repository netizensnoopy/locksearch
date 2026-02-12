use crate::config::Config;
use crate::indexer::ProgramIndex;
use crate::search::SearchEngine;
use iced::keyboard;
use iced::widget::{button, column, container, image, mouse_area, row, scrollable, svg, text, text_input, Column, Space};
use iced::{theme, window, Application, Color, Command, Element, Length, Subscription, Theme};
use std::path::PathBuf;
use std::sync::Arc;

// Embedded SVG icons for window controls
const ICON_MINIMIZE: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\"><line x1=\"2\" y1=\"6\" x2=\"10\" y2=\"6\" stroke=\"#7b8394\" stroke-width=\"1.5\" stroke-linecap=\"round\"/></svg>";
const ICON_MAXIMIZE: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\"><rect x=\"2\" y=\"2\" width=\"8\" height=\"8\" rx=\"1\" fill=\"none\" stroke=\"#7b8394\" stroke-width=\"1.3\"/></svg>";
const ICON_CLOSE: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\"><line x1=\"3\" y1=\"3\" x2=\"9\" y2=\"9\" stroke=\"#7b8394\" stroke-width=\"1.5\" stroke-linecap=\"round\"/><line x1=\"9\" y1=\"3\" x2=\"3\" y2=\"9\" stroke=\"#7b8394\" stroke-width=\"1.5\" stroke-linecap=\"round\"/></svg>";

// =============== COLOR PALETTE ===============

/// Outer window background — matches panel so no black gap
const BG_OUTER: Color = Color::from_rgba(0.08, 0.09, 0.13, 1.0);

/// Main panel background — dark navy
const BG_PANEL: Color = Color::from_rgba(0.08, 0.09, 0.13, 0.92);

/// Search bar background — slightly lighter than panel
const BG_SEARCH: Color = Color::from_rgba(0.11, 0.13, 0.18, 0.95);

/// Selected result row background
const BG_SELECTED: Color = Color::from_rgba(0.12, 0.16, 0.22, 0.90);

/// Search bar border glow — purple/indigo accent
const BORDER_GLOW: Color = Color::from_rgb(0.38, 0.30, 0.72);

/// Selected item border — cool blue
const BORDER_SELECTED: Color = Color::from_rgb(0.22, 0.42, 0.68);

/// Panel outer border — subtle gray
const BORDER_PANEL: Color = Color::from_rgba(0.25, 0.28, 0.36, 0.45);

/// Primary text — near white
const TEXT_WHITE: Color = Color::from_rgb(0.92, 0.93, 0.96);

/// Secondary text — muted gray
const TEXT_GRAY: Color = Color::from_rgb(0.48, 0.52, 0.60);

/// Highlighted path text on selected items
const TEXT_BLUE: Color = Color::from_rgb(0.32, 0.58, 0.84);

/// Letter-placeholder icon background
const ICON_BG: Color = Color::from_rgb(0.25, 0.28, 0.38);

pub struct App {
    config: Config,
    program_index: Arc<ProgramIndex>,
    search_query: String,
    search_results: Vec<ProgramResult>,
    selected_index: usize,
    is_indexing: bool,
    indexed_count: usize,
}

#[derive(Clone, Debug)]
pub struct ProgramResult {
    pub path: PathBuf,
    pub display_name: String,
    pub icon_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    SearchCompleted(Vec<ProgramResult>),
    LaunchSelected,
    KeyPressed(keyboard::Key),
    IndexingProgress(bool, usize),
    StartIndexing,
    CacheLoaded(bool),
    WindowMinimize,
    WindowMaximize,
    WindowClose,
    WindowDrag,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Command<Message>) {
        let index = Arc::new(ProgramIndex::new());
        let enable_cache = config.enable_cache;
        let cache_index = Arc::clone(&index);

        (
            Self {
                config,
                program_index: index,
                search_query: String::new(),
                search_results: Vec::new(),
                selected_index: 0,
                is_indexing: false,
                indexed_count: 0,
            },
            if enable_cache {
                // Try loading cache first, then start indexing in background
                Command::perform(
                    async move { cache_index.load_cache().await },
                    Message::CacheLoaded,
                )
            } else {
                Command::perform(async {}, |_| Message::StartIndexing)
            },
        )
    }

    fn title(&self) -> String {
        "LockSearch".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.selected_index = 0;
                return self.perform_search();
            }
            Message::SearchCompleted(results) => {
                self.search_results = results;
                if self.selected_index >= self.search_results.len() {
                    self.selected_index = 0;
                }
            }
            Message::LaunchSelected => {
                if let Some(result) = self.search_results.get(self.selected_index) {
                    let _ = open::that(&result.path);
                }
            }
            Message::CacheLoaded(loaded) => {
                if loaded {
                    // Cache loaded — show programs immediately
                    let search_cmd = self.perform_search();
                    // Also start re-indexing in background
                    let start_cmd = Command::perform(async {}, |_| Message::StartIndexing);
                    return Command::batch(vec![search_cmd, start_cmd]);
                } else {
                    // No cache — just start indexing
                    return Command::perform(async {}, |_| Message::StartIndexing);
                }
            }
            Message::WindowMinimize => {
                return window::minimize(window::Id::MAIN, true);
            }
            Message::WindowMaximize => {
                return window::toggle_maximize(window::Id::MAIN);
            }
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowDrag => {
                return window::drag(window::Id::MAIN);
            }
            Message::KeyPressed(key) => match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                    if !self.search_results.is_empty() {
                        self.selected_index = (self.selected_index + 1) % self.search_results.len();
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                    if !self.search_results.is_empty() {
                        self.selected_index = if self.selected_index == 0 {
                            self.search_results.len() - 1
                        } else {
                            self.selected_index - 1
                        };
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    if let Some(result) = self.search_results.get(self.selected_index) {
                        let _ = open::that(&result.path);
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    self.search_query.clear();
                    self.selected_index = 0;
                    return self.perform_search();
                }
                _ => {}
            },
            Message::StartIndexing => {
                if !self.is_indexing {
                    self.is_indexing = true;
                    let index = Arc::clone(&self.program_index);
                    return Command::perform(
                        async move {
                            // start_indexing spawns a blocking task and returns immediately
                            index.start_indexing().await;
                            // Signal that indexing has started — we'll poll for completion
                            (true, 0usize)
                        },
                        |(is_idx, count)| Message::IndexingProgress(is_idx, count),
                    );
                }
            }
            Message::IndexingProgress(is_indexing, count) => {
                self.is_indexing = is_indexing;
                self.indexed_count = count;
                if is_indexing {
                    // Poll every 100ms until indexing completes
                    let index = Arc::clone(&self.program_index);
                    return Command::perform(
                        async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            (index.is_indexing().await, index.indexed_count().await)
                        },
                        |(is_idx, count)| Message::IndexingProgress(is_idx, count),
                    );
                } else {
                    // Indexing finished — refresh search results
                    return self.perform_search();
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Search icon — bold magnifying glass
        let search_icon_svg: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" height=\"24px\" viewBox=\"0 0 24 24\" width=\"24px\" fill=\"none\" stroke=\"#8890a4\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"11\" cy=\"11\" r=\"7\"/><line x1=\"16.5\" y1=\"16.5\" x2=\"21\" y2=\"21\"/></svg>";
        let search_icon = container(
            svg(svg::Handle::from_memory(search_icon_svg))
                .width(self.config.search_icon_size)
                .height(self.config.search_icon_size),
        )
        .padding([0, 4, 0, 0]);

        // Search input
        let search_input = text_input("Search apps, files, and settings...", &self.search_query)
            .on_input(Message::SearchChanged)
            .on_submit(Message::LaunchSelected)
            .padding([14, 8])
            .size(16)
            .width(Length::Fill);

        let search_row = row![search_icon, search_input]
            .spacing(10)
            .align_items(iced::Alignment::Center)
            .padding([6, 18]);

        let search_bar = container(search_row)
            .width(Length::Fill)
            .style(theme::Container::Custom(Box::new(SearchBarStyle)));

        // Results area
        let results_content: Element<Message> = if self.search_results.is_empty() {
            if !self.search_query.is_empty() {
                container(text("No results").size(13).style(theme::Text::Color(TEXT_GRAY)))
                    .width(Length::Fill)
                    .padding([40, 0])
                    .center_x()
                    .into()
            } else {
                Space::with_height(0).into()
            }
        } else {
            let mut col: Column<Message> = column![].spacing(2);
            for (idx, result) in self.search_results.iter().enumerate() {
                let is_selected = idx == self.selected_index;
                col = col.push(self.result_row(result, is_selected));
            }
            scrollable(col).height(Length::Fill).width(Length::Fill).into()
        };

        // Window control buttons
        let btn_minimize = button(
            svg(svg::Handle::from_memory(ICON_MINIMIZE)).width(14).height(14)
        )
            .on_press(Message::WindowMinimize)
            .padding([6, 10])
            .style(theme::Button::Custom(Box::new(TitleBarButtonStyle)));

        let btn_maximize = button(
            svg(svg::Handle::from_memory(ICON_MAXIMIZE)).width(14).height(14)
        )
            .on_press(Message::WindowMaximize)
            .padding([6, 10])
            .style(theme::Button::Custom(Box::new(TitleBarButtonStyle)));

        let btn_close = button(
            svg(svg::Handle::from_memory(ICON_CLOSE)).width(14).height(14)
        )
            .on_press(Message::WindowClose)
            .padding([6, 10])
            .style(theme::Button::Custom(Box::new(CloseButtonStyle)));

        // Draggable title bar
        let title_label = mouse_area(
            container(
                text("LockSearch")
                    .size(12)
                    .style(theme::Text::Color(TEXT_GRAY))
            )
            .width(Length::Fill)
            .padding([8, 8])
        )
        .on_press(Message::WindowDrag);

        let title_bar = row![
            title_label,
            btn_minimize,
            btn_maximize,
            btn_close,
        ]
        .align_items(iced::Alignment::Center)
        .padding([0, 4, 0, 4]);

        // Main panel
        let panel = container(
            column![
                title_bar,
                Space::with_height(4),
                search_bar,
                Space::with_height(12),
                results_content,
                Space::with_height(8),
            ]
            .padding([0, 24]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::Container::Custom(Box::new(PanelStyle)));

        // Outer container
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .style(theme::Container::Custom(Box::new(OuterStyle)))
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _modifiers| Some(Message::KeyPressed(key)))
    }
}

impl App {
    fn result_row(&self, result: &ProgramResult, is_selected: bool) -> Element<'_, Message> {
        let icon_size = self.config.program_icon_size;

        let icon_element: Element<Message> = if let Some(ref icon_path) = result.icon_path {
            let use_real_icon = icon_path.exists()
                && std::fs::metadata(icon_path)
                    .map(|m| m.len() > 500)
                    .unwrap_or(false);

            if use_real_icon {
                let handle = image::Handle::from_path(icon_path);
                container(
                    image(handle)
                        .width(icon_size)
                        .height(icon_size),
                )
                .style(theme::Container::Custom(Box::new(IconContainerStyle)))
                .into()
            } else {
                self.letter_placeholder(&result.display_name)
            }
        } else {
            self.letter_placeholder(&result.display_name)
        };

        let name = text(&result.display_name)
            .size(15)
            .style(theme::Text::Color(TEXT_WHITE));

        let path_str = result.path.to_string_lossy();
        let path_color = if is_selected { TEXT_BLUE } else { TEXT_GRAY };
        let path = text(path_str.to_string())
            .size(11)
            .style(theme::Text::Color(path_color));

        let text_col = column![name, path].spacing(3);

        let content_row = row![icon_element, text_col]
            .spacing(16)
            .align_items(iced::Alignment::Center)
            .padding([10, 14]);

        container(content_row)
            .width(Length::Fill)
            .style(theme::Container::Custom(Box::new(ResultItemStyle { is_selected })))
            .into()
    }

    fn letter_placeholder(&self, name: &str) -> Element<'_, Message> {
        let icon_size = self.config.program_icon_size;

        let first_char = name
            .chars()
            .find(|c| c.is_alphanumeric())
            .unwrap_or('?')
            .to_uppercase()
            .to_string();

        let letter = text(first_char)
            .size((icon_size as f32 * 0.5) as u16)
            .style(theme::Text::Color(TEXT_WHITE));

        container(letter)
            .width(icon_size)
            .height(icon_size)
            .center_x()
            .center_y()
            .style(theme::Container::Custom(Box::new(LetterPlaceholderStyle)))
            .into()
    }

    fn perform_search(&self) -> Command<Message> {
        let query = self.search_query.clone();
        let index = Arc::clone(&self.program_index);
        let max_results = self.config.max_results;

        Command::perform(
            async move {
                let entries = index.get_entries().await;
                let engine = SearchEngine::new();
                let results = engine.search(&query, &entries);

                results
                    .into_iter()
                    .take(max_results)
                    .map(|r| ProgramResult {
                        path: r.entry.path,
                        display_name: r.entry.display_name,
                        icon_path: r.entry.icon_path,
                    })
                    .collect()
            },
            Message::SearchCompleted,
        )
    }
}

// =============== STYLES ===============

struct OuterStyle;
impl container::StyleSheet for OuterStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(BG_OUTER)),
            ..Default::default()
        }
    }
}

struct PanelStyle;
impl container::StyleSheet for PanelStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(BG_PANEL)),
            border: iced::Border {
                color: BORDER_PANEL,
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        }
    }
}

struct SearchBarStyle;
impl container::StyleSheet for SearchBarStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(BG_SEARCH)),
            border: iced::Border {
                color: BORDER_GLOW,
                width: 1.5,
                radius: 10.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.38, 0.30, 0.72, 0.25),
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        }
    }
}

struct ResultItemStyle {
    is_selected: bool,
}
impl container::StyleSheet for ResultItemStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        if self.is_selected {
            container::Appearance {
                background: Some(iced::Background::Color(BG_SELECTED)),
                border: iced::Border {
                    color: BORDER_SELECTED,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        } else {
            container::Appearance {
                background: None,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        }
    }
}

struct LetterPlaceholderStyle;
impl container::StyleSheet for LetterPlaceholderStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(ICON_BG)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

struct IconContainerStyle;
impl container::StyleSheet for IconContainerStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: None,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct TitleBarButtonStyle;
impl button::StyleSheet for TitleBarButtonStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: TEXT_GRAY,
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: TEXT_WHITE,
            ..Default::default()
        }
    }
}

struct CloseButtonStyle;
impl button::StyleSheet for CloseButtonStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: TEXT_GRAY,
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.80, 0.20, 0.20))),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: TEXT_WHITE,
            ..Default::default()
        }
    }
}
