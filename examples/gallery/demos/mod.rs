use woodpecker_ui::prelude::WidgetChildren;

mod part1;
mod part2;
mod part3;
mod part4;

pub(crate) use part1::{PaginationDemo, StepperDemo, ToggleButtonDemo};
pub(crate) use part2::{ChipDemo, RatingDemo};
pub(crate) use part3::{DrawerDemo, ModalDemo, ToastDemo};
pub(crate) use part4::{ImageListDemo, MasonryDemo, PopoverDemo, SplitterDemo, TransferListDemo};

use crate::stories::Story;
use part1::*;
use part2::*;
use part3::*;
use part4::*;

pub(crate) fn story_demo(story: Story) -> WidgetChildren {
    match story {
        Story::Button => demo_button(),
        Story::IconButton => demo_icon_button(),
        Story::ToggleButton => demo_toggle_button(),
        Story::SpeedDial => demo_speed_dial(),
        Story::Link => demo_link(),
        Story::Checkbox => demo_checkbox(),
        Story::Radio => demo_radio(),
        Story::Toggle => demo_toggle(),
        Story::Slider => demo_slider(),
        Story::ComboBox => demo_combo_box(),
        Story::Dropdown => demo_dropdown(),
        Story::DatePicker => demo_date_picker(),
        Story::NumberInput => demo_number_input(),
        Story::TextBox => demo_text_box(),
        Story::Tabs => demo_tabs(),
        Story::BottomNavigation => demo_bottom_navigation(),
        Story::NavigationRail => demo_navigation_rail(),
        Story::Breadcrumbs => demo_breadcrumbs(),
        Story::Pagination => demo_pagination(),
        Story::Stepper => demo_stepper(),
        Story::Avatar => demo_avatar(),
        Story::Badge => demo_badge(),
        Story::Chip => demo_chip(),
        Story::List => demo_list(),
        Story::Table => demo_table(),
        Story::Chart => demo_chart(),
        Story::BarChart => demo_bar_chart(),
        Story::AreaChart => demo_area_chart(),
        Story::PieChart => demo_pie_chart(),
        Story::Timeline => demo_timeline(),
        Story::TreeView => demo_tree_view(),
        Story::Typography => demo_typography(),
        Story::Markdown => demo_markdown(),
        Story::Divider => demo_divider(),
        Story::Tooltip => demo_tooltip(),
        Story::Rating => demo_rating(),
        Story::Alert => demo_alert(),
        Story::ProgressBar => demo_progress_bar(),
        Story::Skeleton => demo_skeleton(),
        Story::Spinner => demo_spinner(),
        Story::Toast => demo_toast(),
        Story::Card => demo_card(),
        Story::Paper => demo_paper(),
        Story::Accordion => demo_accordion(),
        Story::AppBar => demo_app_bar(),
        Story::Drawer => demo_drawer(),
        Story::Modal => demo_modal(),
        Story::Popover => demo_popover(),
        Story::Menu => demo_menu(),
        Story::Masonry => demo_masonry(),
        Story::ImageList => demo_image_list(),
        Story::VirtualList => demo_virtual_list(),
        Story::TransferList => demo_transfer_list(),
        Story::Splitter => demo_splitter(),
        Story::ColorPicker => demo_color_picker(),
        Story::Window => demo_window(),
        Story::Dock => demo_dock(),
    }
}

