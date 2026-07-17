use crate::{theming::ThemeRegisterExt, watched_resource::WatchedResource, WidgetRegisterExt};
use bevy::prelude::*;

mod accordion;
mod alert;
pub(crate) mod animation;
mod app;
mod app_bar;
mod avatar;
mod badge;
mod bottom_navigation;
mod breadcrumbs;
mod button;
mod card;
mod chart;
mod checkbox;
mod chip;
mod clip;
mod color_picker;
/// A set of default colors used by Woodpecker UI.
pub mod colors;
mod combo_box;
mod date_picker;
mod divider;
mod drawer;
mod dropdown;
mod element;
mod icon_button;
mod image_list;
mod link;
mod list;
mod masonry;
mod menu;
mod modal;
mod navigation_rail;
mod number_input;
mod overlay_root;
mod pagination;
mod paper;
pub(crate) mod popover;
mod progress_bar;
mod radio;
mod rating;
mod scroll;
mod skeleton;
mod slider;
mod speed_dial;
mod spinner;
mod splitter;
mod stepper;
mod tab;
mod table;
pub(crate) mod text_box;
mod timeline;
mod toast;
mod toggle;
mod toggle_button;
mod tooltip;
mod transfer_list;
pub(crate) mod transition;
mod tree_view;
mod typography;
mod virtual_list;
/// A pure, ECS-free helper for windowed/virtualized rendering of large uniform-item lists.
pub mod virtualization;
pub(crate) mod window;
mod windowing_context;

mod syntax_highlighting;
pub use syntax_highlighting::highlight;

pub use accordion::{
    Accordion, AccordionChanged, AccordionItem, AccordionItemStyles, AccordionMode,
};
pub use alert::{Alert, AlertDismissed, AlertStyles};
pub use animation::{Animating, AnimationGroup, AnimationTimeline, AnimationTrack, Spring, SpringStyle};
pub use app::WoodpeckerApp;
pub use app_bar::{AppBar, AppBarLeading, AppBarPosition, AppBarStyles, AppBarTrailing};
// use bevy_mod_picking::prelude::EventListenerPlugin;
pub use avatar::{Avatar, AvatarStyles};
pub use badge::{Badge, BadgeStyles, BadgeVariant, BadgeVariantColors};
pub use bottom_navigation::{
    BottomNavItem, BottomNavigation, BottomNavigationChanged, BottomNavigationStyles,
};
pub use breadcrumbs::{BreadcrumbClicked, Breadcrumbs, BreadcrumbsStyles};
pub use button::{ButtonStyles, WButton};
pub use card::{Card, CardActions, CardStyles};
pub use chart::{ChartSeries, ChartStyles, LineChart};
pub use checkbox::{
    Checkbox, CheckboxChanged, CheckboxState, CheckboxStyles, CheckboxWidgetStyles,
};
pub use chip::{Chip, ChipClicked, ChipDeleted, ChipLeading};
pub use clip::Clip;
pub use color_picker::{ColorPicker, ColorPickerChanged, ColorPickerStyles};
pub use combo_box::{ComboBox, ComboBoxChanged, ComboBoxMatchMode, ComboBoxStyles};
pub use date_picker::{CalendarDate, DateChanged, DatePicker, DatePickerStyles};
pub use divider::{Divider, DividerStyles};
pub use drawer::{Drawer, DrawerCloseRequested, DrawerPosition, DrawerStyles, DrawerVariant};
pub use dropdown::{Dropdown, DropdownChanged, DropdownStyles};
pub use element::Element;
pub use icon_button::{IconButton, IconButtonStyles};
pub use image_list::{ImageList, ImageListItem, ImageListStyles};
pub use link::{Link, LinkClicked, LinkStyles};
pub use list::{
    List, ListItem, ListItemClicked, ListItemLeading, ListItemStyles, ListItemTrailing,
};
pub use masonry::{pack_masonry, Masonry, MasonryItem, MasonryStyles};
pub use menu::{Menu, MenuItemSelected, MenuStyles};
pub use modal::{Modal, ModalStyles, TitleChildren};
pub use navigation_rail::{
    NavigationRail, NavigationRailChanged, NavigationRailItem, NavigationRailStyles,
};
pub use number_input::{NumberInput, NumberInputChanged, NumberInputState, NumberInputStyles};
pub use overlay_root::OverlayRootWidget;
pub use pagination::{Pagination, PaginationChanged};
pub use paper::{Paper, PaperStyles};
pub use popover::{Popover, PopoverBundle, PopoverContent, PopoverPlacement, PopoverStyles};
pub use progress_bar::{ProgressBar, ProgressBarStyles};
pub use radio::{RadioChanged, RadioGroup, RadioGroupStyles};
pub use rating::{Rating, RatingChanged, RatingStyles};
pub use scroll::content::ScrollContent;
pub use scroll::scroll_bar::{ScrollBar, ScrollBarStyles};
pub use scroll::scroll_box::ScrollBox;
pub use scroll::{ScrollContext, ScrollContextProvider, TaggedContext};
pub use skeleton::{Skeleton, SkeletonStyles, SkeletonVariant};
pub use slider::{Slider, SliderChanged, SliderState, SliderStyles};
pub use speed_dial::{SpeedDial, SpeedDialAction, SpeedDialActionClicked, SpeedDialStyles};
pub use spinner::{Spinner, SpinnerMode, SpinnerStyles};
pub use splitter::{Splitter, SplitterChanged, SplitterState, SplitterStyles};
pub use stepper::{Stepper, StepperChanged, StepperOrientation, StepperStyles};
pub use tab::*;
pub use table::{Table, TableColumn, TableRow, TableSort, TableStyles};
pub use text_box::{ApplyHighlighting, TextBox, TextBoxState, TextChanged, TextboxStyles};
pub use timeline::{Timeline, TimelineItem, TimelineStyles};
pub use toast::{ToastEntry, ToastQueue, ToastStyles, ToastViewport};
pub use toggle::{Toggle, ToggleChanged, ToggleState, ToggleStyles, ToggleWidgetStyles};
pub use toggle_button::{
    ButtonGroup, ButtonGroupChanged, ToggleButton, ToggleButtonChanged, ToggleButtonStyles,
};
pub use tooltip::{Tooltip, TooltipStyles};
pub use transfer_list::{TransferList, TransferListChanged, TransferListStyles};
pub use transition::*;
pub use tree_view::{TreeNode, TreeNodeSelected, TreeNodeToggled, TreeView, TreeViewStyles};
pub use typography::{Typography, TypographyVariant};
pub use virtual_list::{VirtualList, VirtualListItemContent};
pub use virtualization::{compute_virtual_window, dynamic_overscan, VirtualWindow};
pub use window::{WindowState, WindowStyles, WoodpeckerWindow};
pub use windowing_context::{WindowingContext, WindowingContextProvider};

/// A core set of UI widgets that Woodpecker UI provides.
// TODO: Make this optional? Expose publicly.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<WoodpeckerApp>()
            .register_widget::<Accordion>()
            .register_widget::<AccordionItem>()
            .register_widget::<Alert>()
            .register_widget::<AppBar>()
            .register_widget::<Element>()
            .register_widget::<WButton>()
            .register_widget::<LineChart>()
            .register_widget::<Avatar>()
            .register_widget::<Badge>()
            .register_widget::<BottomNavigation>()
            .register_widget::<Breadcrumbs>()
            .register_widget::<Card>()
            .register_widget::<Divider>()
            .register_widget::<Drawer>()
            .register_widget::<ProgressBar>()
            .register_widget::<Spinner>()
            .register_widget::<RadioGroup>()
            .register_widget::<Rating>()
            .register_widget::<Popover>()
            .register_widget::<Tooltip>()
            .register_widget::<TransferList>()
            .register_widget::<Clip>()
            .register_widget::<TextBox>()
            .register_widget::<Timeline>()
            .register_widget::<Menu>()
            .register_widget::<Modal>()
            .register_widget::<NavigationRail>()
            .register_widget::<NumberInput>()
            .register_widget::<ScrollContextProvider>()
            .register_widget::<ScrollContent>()
            .register_widget::<ScrollBox>()
            .register_widget::<ScrollBar>()
            .register_widget::<IconButton>()
            .register_widget::<ImageList>()
            .register_widget::<Link>()
            .register_widget::<List>()
            .register_widget::<ListItem>()
            .register_widget::<Masonry>()
            .register_widget::<Toggle>()
            .register_widget::<ToggleButton>()
            .register_widget::<ButtonGroup>()
            .register_widget::<Skeleton>()
            .register_widget::<Slider>()
            .register_widget::<SpeedDial>()
            .register_widget::<Splitter>()
            .register_widget::<Stepper>()
            .register_widget::<WoodpeckerWindow>()
            .register_widget::<WindowingContextProvider>()
            .register_widget::<Dropdown>()
            .register_widget::<TabButton>()
            .register_widget::<TabContextProvider>()
            .register_widget::<TabContent>()
            .register_widget::<Checkbox>()
            .register_widget::<Chip>()
            .register_widget::<ColorPicker>()
            .register_widget::<ComboBox>()
            .register_widget::<DatePicker>()
            .register_widget::<Table>()
            .register_widget::<VirtualList>()
            .register_widget::<TreeView>()
            .register_widget::<Typography>()
            .register_widget::<ToastViewport>()
            .register_widget::<OverlayRootWidget>()
            .register_widget::<Pagination>()
            .register_widget::<Paper>()
            .register_themed_style::<AccordionItemStyles>()
            .register_themed_style::<AlertStyles>()
            .register_themed_style::<AvatarStyles>()
            .register_themed_style::<AppBarStyles>()
            .register_themed_style::<BadgeStyles>()
            .register_themed_style::<BottomNavigationStyles>()
            .register_themed_style::<BreadcrumbsStyles>()
            .register_themed_style::<ButtonStyles>()
            .register_themed_style::<CardStyles>()
            .register_themed_style::<ChartStyles>()
            .register_themed_style::<CheckboxWidgetStyles>()
            .register_themed_style::<ColorPickerStyles>()
            .register_themed_style::<ComboBoxStyles>()
            .register_themed_style::<DatePickerStyles>()
            .register_themed_style::<DividerStyles>()
            .register_themed_style::<DrawerStyles>()
            .register_themed_style::<DropdownStyles>()
            .register_themed_style::<IconButtonStyles>()
            .register_themed_style::<ImageListStyles>()
            .register_themed_style::<LinkStyles>()
            .register_themed_style::<ListItemStyles>()
            .register_themed_style::<MasonryStyles>()
            .register_themed_style::<MenuStyles>()
            .register_themed_style::<ModalStyles>()
            .register_themed_style::<NavigationRailStyles>()
            .register_themed_style::<NumberInputStyles>()
            .register_themed_style::<PaperStyles>()
            .register_themed_style::<PopoverStyles>()
            .register_themed_style::<ProgressBarStyles>()
            .register_themed_style::<RadioGroupStyles>()
            .register_themed_style::<RatingStyles>()
            .register_themed_style::<ScrollBarStyles>()
            .register_themed_style::<SkeletonStyles>()
            .register_themed_style::<SliderStyles>()
            .register_themed_style::<SpeedDialStyles>()
            .register_themed_style::<SpinnerStyles>()
            .register_themed_style::<SplitterStyles>()
            .register_themed_style::<StepperStyles>()
            .register_themed_style::<TableStyles>()
            .register_themed_style::<TabButtonStyles>()
            .register_themed_style::<TabStyles>()
            .register_themed_style::<TextboxStyles>()
            .register_themed_style::<TimelineStyles>()
            .register_themed_style::<ToastStyles>()
            .register_themed_style::<TreeViewStyles>()
            .register_themed_style::<ToggleWidgetStyles>()
            .register_themed_style::<ToggleButtonStyles>()
            .register_themed_style::<TooltipStyles>()
            .register_themed_style::<TransferListStyles>()
            .register_themed_style::<WindowStyles>()
            .insert_resource(ToastQueue::default())
            .register_type::<WatchedResource<ToastQueue>>()
            .add_systems(
                PreUpdate,
                (
                    crate::watched_resource::sync_watched_resource::<ToastQueue>,
                    overlay_root::sync_overlay_root,
                ),
            )
            .add_systems(
                Update,
                (
                    text_box::cursor_animation_system,
                    transition::update_transitions,
                    animation::update_animation_timelines,
                    animation::update_springs,
                    animation::update_spring_styles.after(animation::update_springs),
                    toast::tick_toasts,
                ),
            );
    }
}
