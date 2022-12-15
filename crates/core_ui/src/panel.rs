// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use protos::spelldawn::game_command::Command;
use protos::spelldawn::toggle_panel_command::ToggleCommand;
use protos::spelldawn::{
    interface_panel_address, ClientPanelAddress, Dimension, FlexAlign, FlexJustify, FlexPosition,
    ImageScaleMode, InterfacePanel, InterfacePanelAddress, TextAlign, TogglePanelCommand,
    UpdatePanelsCommand,
};

use crate::button::IconButton;
use crate::component::{ComponentObject, EmptyComponent};
use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;
use crate::text::Text;
use crate::{icons, style};

/// Converts a [ClientPanelAddress] into an [InterfacePanelAddress].
pub fn client(address: ClientPanelAddress) -> InterfacePanelAddress {
    InterfacePanelAddress {
        address_type: Some(interface_panel_address::AddressType::ClientPanel(address as i32)),
    }
}

/// Set the indicated panel as the only open panel
pub fn set(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::SetPanel(address.into())),
    })
}

/// Add the indicated panel to the end of the stack of open views if
/// it is not already present.
pub fn open(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::OpenPanel(address.into())),
    })
}

/// Opens a new bottom sheet with the indicated panel.
///
/// Closes any existing bottom sheet.
pub fn open_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::OpenBottomSheetAddress(address.into())),
    })
}

/// Pushes the indicated panel as a new bottom sheet page.
///
/// If no bottom sheet is currently open, the behavior is identical to
/// [open_bottom_sheet].
pub fn push_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::PushBottomSheetAddress(address.into())),
    })
}

/// Removes the indicated panel from the stack of open views.
pub fn close(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::ClosePanel(address.into())),
    })
}

/// Closes all open panels
pub fn close_all() -> Command {
    Command::TogglePanel(TogglePanelCommand { toggle_command: Some(ToggleCommand::CloseAll(())) })
}

/// Closes the currently-open bottom sheet.
pub fn close_bottom_sheet() -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::CloseBottomSheet(())),
    })
}

/// Pops the currently-open bottom sheet page, displaying 'address' as the *new*
/// sheet contents.
pub fn pop_to_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::PopToBottomSheetAddress(address.into())),
    })
}

/// Command to update the contents of a panel
pub fn update(address: impl Into<InterfacePanelAddress>, node: Option<Node>) -> Command {
    Command::UpdatePanels(UpdatePanelsCommand {
        panels: vec![InterfacePanel { address: Some(address.into()), node }],
    })
}

/// A rectangular interface element that displays content centered on-screen,
/// optionally including a title or close button.
pub struct Panel {
    address: InterfacePanelAddress,
    width: Dimension,
    height: Dimension,
    layout: Layout,
    content: Box<dyn ComponentObject>,
    title: Option<String>,
    show_close_button: bool,
}

impl Panel {
    pub fn new(
        address: impl Into<InterfacePanelAddress>,
        width: impl Into<Dimension>,
        height: impl Into<Dimension>,
    ) -> Self {
        Self {
            address: address.into(),
            width: width.into(),
            height: height.into(),
            layout: Layout::default(),
            content: Box::new(EmptyComponent),
            title: None,
            show_close_button: false,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn show_close_button(mut self, show_close_button: bool) -> Self {
        self.show_close_button = show_close_button;
        self
    }
}

impl Component for Panel {
    fn build(self) -> Option<Node> {
        let background = style::sprite("Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_window_big_recolored");
        Row::new(self.title.clone().unwrap_or_else(|| "Panel".to_string()))
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Left, 50.pct())
                    .position(Edge::Top, 50.pct())
                    .translate((-50).pct(), (-50).pct())
                    .width(self.width)
                    .height(self.height)
                    .padding(Edge::Horizontal, 32.px())
                    .padding(Edge::Bottom, 32.px())
                    .padding(Edge::Top, 48.px())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .background_image(background)
                    .background_image_scale_mode(ImageScaleMode::StretchToFill)
                    .image_slice(Edge::All, 128.px()),
            )
            .child(self.title.map(TitleBar::new))
            .child(self.show_close_button.then(|| {
                IconButton::new(icons::CLOSE).action(close(self.address)).show_frame(true).layout(
                    Layout::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::Right, (-20).px())
                        .position(Edge::Top, (-20).px()),
                )
            }))
            .child_boxed(self.content)
            .build()
    }
}

#[derive(Debug)]
pub struct TitleBar {
    title: String,
}

impl TitleBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }
}

impl Component for TitleBar {
    fn build(self) -> Option<Node> {
        let background = style::sprite(
            "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_big_bar_512",
        );
        Row::new(format!("TitleBar {}", self.title))
            .style(Style::new().position_type(FlexPosition::Absolute).position(Edge::All, 0.px()))
            .child(
                Row::new("TitleBarContent")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Left, 64.px())
                            .position(Edge::Right, 64.px())
                            .translate(0.px(), (-50).pct())
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center)
                            .padding(Edge::Vertical, 16.px())
                            .padding(Edge::Horizontal, 32.px())
                            .background_image(background)
                            .background_image_scale_mode(ImageScaleMode::StretchToFill)
                            .image_slice(Edge::All, 64.px()),
                    )
                    .child(
                        Text::new(self.title)
                            .font_size(FontSize::PanelTitle)
                            .color(FontColor::PanelTitle)
                            .font(Font::PanelTitle)
                            .text_align(TextAlign::MiddleCenter),
                    ),
            )
            .build()
    }
}
