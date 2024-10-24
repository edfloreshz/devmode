// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::{fl, pages};
use cosmic::app::{Core, Task};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};

const REPOSITORY: &str = "https://github.com/edfloreshz/cosmic-app-template";

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct Devmode {
    core: Core,
    context_page: ContextPage,
    page: Page,
    clone: pages::clone::ClonePage,
    workspaces: pages::workspaces::WorkspacesPage,
    open: pages::open::OpenPage,
    config: pages::config::ConfigPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav: nav_bar::Model,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    Clone(pages::clone::Message),
    Workspaces(pages::workspaces::Message),
    Open(pages::open::Message),
    Config(pages::config::Message),
}

/// Identifies a page in the application.
#[derive(Debug, Clone, Copy)]
pub enum Page {
    Clone,
    Workspaces,
    Open,
    Config,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for Devmode {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.Devmode";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Instructs the cosmic runtime to use this model as the nav bar model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Clone")
            .data::<Page>(Page::Clone)
            .icon(icon::from_name("browser-download-symbolic"))
            .activate();

        nav.insert()
            .text("Workspaces")
            .data::<Page>(Page::Workspaces)
            .icon(icon::from_name("multitasking-symbolic"));

        nav.insert()
            .text("Open")
            .data::<Page>(Page::Open)
            .icon(icon::from_name("folder-open-symbolic"));

        nav.insert()
            .text("Config")
            .data::<Page>(Page::Config)
            .icon(icon::from_name("settings-symbolic"));

        let app = Devmode {
            core,
            context_page: ContextPage::default(),
            page: Page::Clone,
            clone: pages::clone::ClonePage::new(),
            workspaces: pages::workspaces::WorkspacesPage::new(),
            open: pages::open::OpenPage::new(),
            config: pages::config::ConfigPage::new(),
            key_binds: HashMap::new(),
            nav,
        };

        (app, Task::none())
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let spacing = theme::active().cosmic().spacing;

        let page: Element<Self::Message> = match self.page {
            Page::Clone => self.clone.view().map(Message::Clone),
            Page::Workspaces => self.workspaces.view().map(Message::Workspaces),
            Page::Open => self.open.view().map(Message::Open),
            Page::Config => self.config.view().map(Message::Config),
        };

        widget::container(page)
            .apply(widget::container)
            .padding(spacing.space_xxs)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Clone(message) => {
                for command in self.clone.update(message) {
                    match command {
                        pages::clone::Command::Clone(_repository, _workspace) => {
                            todo!("Implement cloning mechanism.")
                        }
                    }
                }
            }
            Message::Workspaces(message) => for command in self.workspaces.update(message) {},
            Message::Open(message) => for command in self.open.update(message) {},
            Message::Config(message) => for command in self.config.update(message) {},
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page.eq(&context_page) {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
        }
        Task::none()
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
        })
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        // Activate the page in the model.
        self.nav.activate(id);

        if let Some(page) = self.nav.active_data::<Page>() {
            self.page = *page;
        }

        Task::none()
    }
}

impl Devmode {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/scalable/apps/dev.edfloreshz.Devmode.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }
}
