use bevy::prelude::Reflect;

use crate::props::PropDoc;

/// One entry per widget shown in the gallery. Grouped into categories via `STORIES` below,
/// not as separate enum variants per category -- keeps the category list in exactly one
/// place instead of two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub(crate) enum Story {
    Button,
    IconButton,
    ToggleButton,
    SpeedDial,
    Link,
    Checkbox,
    Radio,
    Toggle,
    Slider,
    ComboBox,
    Dropdown,
    DatePicker,
    NumberInput,
    TextBox,
    Tabs,
    BottomNavigation,
    NavigationRail,
    Breadcrumbs,
    Pagination,
    Stepper,
    Avatar,
    Badge,
    Chip,
    List,
    Table,
    Chart,
    BarChart,
    AreaChart,
    PieChart,
    Timeline,
    TreeView,
    Typography,
    Markdown,
    Divider,
    Tooltip,
    Rating,
    Alert,
    ProgressBar,
    Skeleton,
    Spinner,
    Toast,
    Card,
    Paper,
    Accordion,
    AppBar,
    Drawer,
    Modal,
    Popover,
    Menu,
    Masonry,
    ImageList,
    VirtualList,
    TransferList,
    Splitter,
    ColorPicker,
    Window,
    Dock,
}

/// `(category, label, story)` -- the sidebar groups consecutive entries sharing the same
/// category under one header, so entries for the same category must stay adjacent here.
pub(crate) const STORIES: &[(&str, &str, Story)] = &[
    ("Buttons & Actions", "Button", Story::Button),
    ("Buttons & Actions", "Icon Button", Story::IconButton),
    ("Buttons & Actions", "Toggle Button", Story::ToggleButton),
    ("Buttons & Actions", "Speed Dial", Story::SpeedDial),
    ("Buttons & Actions", "Link", Story::Link),
    ("Selection Controls", "Checkbox", Story::Checkbox),
    ("Selection Controls", "Radio", Story::Radio),
    ("Selection Controls", "Toggle", Story::Toggle),
    ("Selection Controls", "Slider", Story::Slider),
    ("Selection Controls", "Combo Box", Story::ComboBox),
    ("Selection Controls", "Dropdown", Story::Dropdown),
    ("Selection Controls", "Date Picker", Story::DatePicker),
    ("Selection Controls", "Number Input", Story::NumberInput),
    ("Selection Controls", "Text Box", Story::TextBox),
    ("Navigation", "Tabs", Story::Tabs),
    ("Navigation", "Bottom Navigation", Story::BottomNavigation),
    ("Navigation", "Navigation Rail", Story::NavigationRail),
    ("Navigation", "Breadcrumbs", Story::Breadcrumbs),
    ("Navigation", "Pagination", Story::Pagination),
    ("Navigation", "Stepper", Story::Stepper),
    ("Data Display", "Avatar", Story::Avatar),
    ("Data Display", "Badge", Story::Badge),
    ("Data Display", "Chip", Story::Chip),
    ("Data Display", "List", Story::List),
    ("Data Display", "Table", Story::Table),
    ("Data Display", "Line Chart", Story::Chart),
    ("Data Display", "Bar Chart", Story::BarChart),
    ("Data Display", "Area Chart", Story::AreaChart),
    ("Data Display", "Pie Chart", Story::PieChart),
    ("Data Display", "Timeline", Story::Timeline),
    ("Data Display", "Tree View", Story::TreeView),
    ("Data Display", "Typography", Story::Typography),
    ("Data Display", "Markdown", Story::Markdown),
    ("Data Display", "Divider", Story::Divider),
    ("Data Display", "Tooltip", Story::Tooltip),
    ("Data Display", "Rating", Story::Rating),
    ("Feedback", "Alert", Story::Alert),
    ("Feedback", "Progress Bar", Story::ProgressBar),
    ("Feedback", "Skeleton", Story::Skeleton),
    ("Feedback", "Spinner", Story::Spinner),
    ("Feedback", "Toast", Story::Toast),
    ("Surfaces", "Card", Story::Card),
    ("Surfaces", "Paper", Story::Paper),
    ("Surfaces", "Accordion", Story::Accordion),
    ("Surfaces", "App Bar", Story::AppBar),
    ("Surfaces", "Drawer", Story::Drawer),
    ("Surfaces", "Modal", Story::Modal),
    ("Surfaces", "Popover", Story::Popover),
    ("Surfaces", "Menu", Story::Menu),
    ("Layout", "Masonry", Story::Masonry),
    ("Layout", "Image List", Story::ImageList),
    ("Layout", "Virtual List", Story::VirtualList),
    ("Layout", "Transfer List", Story::TransferList),
    ("Layout", "Splitter", Story::Splitter),
    ("Color", "Color Picker", Story::ColorPicker),
    ("Windowing", "Window", Story::Window),
    ("Windowing", "Dock", Story::Dock),
];

/// A one-sentence description shown above each story's live demo -- the "Docs" half of the
/// Storybook-style gallery, alongside the "Canvas" half `story_demo` builds.
pub(crate) fn story_doc(story: Story) -> &'static str {
    match story {
        Story::Button => "A clickable button with a text label. The default building block for actions.",
        Story::IconButton => "A compact, icon-only button -- for toolbars and actions that don't need a label.",
        Story::ToggleButton => "A button with a persistent on/off visual state, toggled by the caller in response to a click.",
        Story::SpeedDial => "A floating action button that expands into a fan of secondary actions on click.",
        Story::Link => "An inline, clickable text link that underlines on hover.",
        Story::Checkbox => "A tri-free boolean checkbox with its own internal checked state.",
        Story::Radio => "A set of mutually-exclusive labeled options, rendered inline rather than in a popup.",
        Story::Toggle => "An on/off switch -- the same boolean concept as `Checkbox`, styled as a sliding switch.",
        Story::Slider => "A draggable handle along a track for picking a numeric value in a range.",
        Story::ComboBox => "A text field with a filterable dropdown list -- type to narrow the options, or pick one directly.",
        Story::Dropdown => "A closed picker that expands into a scrollable list of options on click.",
        Story::DatePicker => "A text field that opens a calendar popup for picking a single date.",
        Story::NumberInput => "A numeric field with increment/decrement step buttons and drag-to-scrub support.",
        Story::TextBox => "A single- or multi-line editable text field with cursor and selection support.",
        Story::Tabs => "A row of labeled tab buttons that switch which panel of content is visible.",
        Story::BottomNavigation => "A fixed-height row of mutually-exclusive, evenly-spaced icon+label destinations.",
        Story::NavigationRail => "A narrow, icon-only, vertical single-select navigation column.",
        Story::Breadcrumbs => "A row of `/`-separated links showing the current location within a hierarchy.",
        Story::Pagination => "A row of page-number buttons for paging through a large result set.",
        Story::Stepper => "A horizontal sequence of numbered steps showing progress through a multi-step flow.",
        Story::Avatar => "A small circular (or square) image or initials badge representing a user.",
        Story::Badge => "A small colored pill for a status word or count, often overlaid on another element.",
        Story::Chip => "A compact, pill-shaped tag -- optionally selectable or deletable.",
        Story::List => "A vertical list of rows, each with optional leading/trailing content and hover/selected states.",
        Story::Table => "A header row plus zebra-striped, divided data rows -- optionally virtualized for large datasets.",
        Story::Chart => "A multi-series line chart with labeled axes, gradient fill, and hover values.",
        Story::BarChart => "A grouped or stacked bar chart for comparing values across categories, with per-bar hover values.",
        Story::AreaChart => "A stacked area chart showing how multiple series accumulate over a shared axis, with hover values.",
        Story::PieChart => "A pie or donut chart (set `inner_radius` > 0) for showing proportions of a whole, with per-slice hover values.",
        Story::Timeline => "A vertical sequence of dated events, each with a marker and connecting line.",
        Story::TreeView => "A hierarchical, expand/collapse tree of labeled rows -- a file browser, a scene graph.",
        Story::Typography => "The shared heading/body/caption text scale used throughout the crate's own widgets.",
        Story::Markdown => "Renders a block of markdown source as headings, paragraphs, fenced code blocks, lists, block quotes, and rules.",
        Story::Divider => "A thin horizontal or vertical rule for separating sections of content.",
        Story::Tooltip => "A small label that appears near an element on hover, after a short delay.",
        Story::Rating => "A row of clickable stars (or similar icons) for picking or displaying a rating.",
        Story::Alert => "An inline banner for a success/info/warning/error message, optionally dismissible.",
        Story::ProgressBar => "A determinate horizontal bar showing completion progress from 0 to 1.",
        Story::Skeleton => "A pulsing placeholder shape shown while real content is still loading.",
        Story::Spinner => "An indeterminate rotating loading indicator.",
        Story::Toast => "A short-lived notification card that appears, waits, then dismisses itself.",
        Story::Card => "A raised surface for grouping related content, usually with a title and body.",
        Story::Paper => "The base raised-surface primitive `Card`/`Accordion`/other surfaces compose from.",
        Story::Accordion => "A vertically-stacked set of expand/collapse sections, one (or more) open at a time.",
        Story::AppBar => "A fixed-height chrome bar with leading/title/trailing slots, for the top or bottom of a page.",
        Story::Drawer => "A panel that slides in from an edge of the screen, or sits permanently in the layout.",
        Story::Modal => "A centered dialog with a scrim, blocking interaction with the rest of the page until dismissed.",
        Story::Popover => "A small floating panel anchored to a trigger element, for menus or extra detail.",
        Story::Menu => "A right-click context menu that pops up a list of items at the pointer.",
        Story::Masonry => "A Pinterest-style column layout that packs variable-height items to minimize gaps.",
        Story::ImageList => "A uniform grid of images, optionally with captions.",
        Story::VirtualList => "A single-column virtualized list -- only the visible window's rows are actually mounted.",
        Story::TransferList => "Two side-by-side lists with buttons to move selected items between them.",
        Story::Splitter => "A draggable divider between two panes that resizes them proportionally.",
        Story::ColorPicker => "A hue/saturation/value color picker panel with a hex readout.",
        Story::Window => "A draggable, resizable, titled window -- for an in-app floating panel or dialog chrome.",
        Story::Dock => "A resizable, tabbed, redockable panel layout -- drag a tab to a panel's edge to split, or to its center to join; dropping it anywhere else reverts it back to where it started.",
    }
}

/// The public constructor fields of the struct each story's own `demo_x()` actually builds --
/// a Storybook-style "Controls" table. Marker structs with all behavior on a separate
/// `XStyles`/`XState` companion component (e.g. `Checkbox`, `Toggle`) genuinely have zero
/// fields here; that's not an omission.
pub(crate) fn story_props(story: Story) -> &'static [PropDoc] {
    match story {
        Story::Button => &[],
        Story::IconButton => &[],
        Story::ToggleButton => &[PropDoc {
            name: "selected",
            type_str: "bool",
            doc: "Whether this button is currently \"on\"",
        }],
        Story::SpeedDial => &[
            PropDoc {
                name: "icon",
                type_str: "String",
                doc: "The closed-state FAB glyph, swapped for an X while expanded",
            },
            PropDoc {
                name: "actions",
                type_str: "Vec<SpeedDialAction>",
                doc: "The actions shown when expanded",
            },
            PropDoc {
                name: "placement",
                type_str: "PopoverPlacement",
                doc: "Which side the action column expands toward",
            },
        ],
        Story::Link => &[PropDoc {
            name: "label",
            type_str: "String",
            doc: "The link's text",
        }],
        Story::Checkbox => &[],
        Story::Radio => &[
            PropDoc {
                name: "options",
                type_str: "Vec<String>",
                doc: "The option labels",
            },
            PropDoc {
                name: "selected",
                type_str: "usize",
                doc: "Which option is selected initially",
            },
            PropDoc {
                name: "horizontal",
                type_str: "bool",
                doc: "Lays out options in a row instead of the default column",
            },
        ],
        Story::Toggle => &[],
        Story::Slider => &[
            PropDoc {
                name: "start",
                type_str: "f32",
                doc: "Start value",
            },
            PropDoc {
                name: "end",
                type_str: "f32",
                doc: "End value",
            },
            PropDoc {
                name: "value",
                type_str: "f32",
                doc: "Initial value",
            },
        ],
        Story::ComboBox => &[
            PropDoc {
                name: "current_value",
                type_str: "String",
                doc: "The current value shown/typed in the trigger",
            },
            PropDoc {
                name: "list",
                type_str: "Vec<String>",
                doc: "The full, unfiltered list of choices",
            },
            PropDoc {
                name: "match_mode",
                type_str: "ComboBoxMatchMode",
                doc: "How typed text filters `list`",
            },
        ],
        Story::Dropdown => &[
            PropDoc {
                name: "current_value",
                type_str: "String",
                doc: "The current value",
            },
            PropDoc {
                name: "list",
                type_str: "Vec<String>",
                doc: "A list of items in the dropdown",
            },
        ],
        Story::DatePicker => &[
            PropDoc {
                name: "selected_date",
                type_str: "Option<CalendarDate>",
                doc: "The currently selected date, if any",
            },
            PropDoc {
                name: "min_date",
                type_str: "Option<CalendarDate>",
                doc: "The earliest selectable date, if any",
            },
            PropDoc {
                name: "max_date",
                type_str: "Option<CalendarDate>",
                doc: "The latest selectable date, if any",
            },
            PropDoc {
                name: "initial_view",
                type_str: "CalendarDate",
                doc: "Which month the calendar opens showing, when `selected_date` is `None`",
            },
            PropDoc {
                name: "placeholder",
                type_str: "String",
                doc: "Shown in the trigger when nothing is selected yet",
            },
        ],
        Story::NumberInput => &[
            PropDoc {
                name: "value",
                type_str: "f32",
                doc: "The initial value -- only read on first mount",
            },
            PropDoc {
                name: "min",
                type_str: "f32",
                doc: "The minimum allowed value",
            },
            PropDoc {
                name: "max",
                type_str: "f32",
                doc: "The maximum allowed value",
            },
            PropDoc {
                name: "step",
                type_str: "f32",
                doc: "The amount each step-button click changes the value by",
            },
            PropDoc {
                name: "drag_pixels_per_step",
                type_str: "f32",
                doc: "Pixels of Alt+drag per `step`; lower is more sensitive",
            },
        ],
        Story::TextBox => &[
            PropDoc {
                name: "initial_value",
                type_str: "String",
                doc: "An initial value",
            },
            PropDoc {
                name: "multi_line",
                type_str: "bool",
                doc: "Indicates this is a multi-line text editor",
            },
            PropDoc {
                name: "text_highlighting",
                type_str: "ApplyHighlighting",
                doc: "Optional syntax/text-color highlighting",
            },
            PropDoc {
                name: "tab_mode",
                type_str: "TabMode",
                doc: "The tab behavior. Defaults to 4 spaces",
            },
        ],
        Story::Tabs => &[
            PropDoc {
                name: "tab buttons",
                type_str: "TabButton",
                doc: "One clickable tab header per tab, keyed by index",
            },
            PropDoc {
                name: "tab content",
                type_str: "TabContent",
                doc: "The panel shown when its matching index is active",
            },
        ],
        Story::BottomNavigation => &[
            PropDoc {
                name: "items",
                type_str: "Vec<BottomNavItem>",
                doc: "The destinations, in display order",
            },
            PropDoc {
                name: "selected",
                type_str: "usize",
                doc: "Which item is selected initially",
            },
            PropDoc {
                name: "height",
                type_str: "f32",
                doc: "Bar height, in pixels",
            },
        ],
        Story::NavigationRail => &[
            PropDoc {
                name: "items",
                type_str: "Vec<NavigationRailItem>",
                doc: "The destinations, in display order",
            },
            PropDoc {
                name: "selected",
                type_str: "usize",
                doc: "Which item is selected initially",
            },
            PropDoc {
                name: "width",
                type_str: "f32",
                doc: "Rail width, in pixels",
            },
        ],
        Story::Breadcrumbs => &[PropDoc {
            name: "items",
            type_str: "Vec<String>",
            doc: "The trail, root-first -- the last entry is the current page",
        }],
        Story::Pagination => &[
            PropDoc {
                name: "page",
                type_str: "usize",
                doc: "The current page, 0-indexed",
            },
            PropDoc {
                name: "page_count",
                type_str: "usize",
                doc: "Total number of pages",
            },
        ],
        Story::Stepper => &[
            PropDoc {
                name: "steps",
                type_str: "Vec<String>",
                doc: "Step labels, in order",
            },
            PropDoc {
                name: "active",
                type_str: "usize",
                doc: "The current step, 0-indexed",
            },
            PropDoc {
                name: "orientation",
                type_str: "StepperOrientation",
                doc: "Horizontal (default) or vertical layout",
            },
            PropDoc {
                name: "clickable",
                type_str: "bool",
                doc: "Whether steps fire `Change<StepperChanged>` on click",
            },
        ],
        Story::Avatar => &[
            PropDoc {
                name: "initials",
                type_str: "String",
                doc: "The initials to display (typically 1-2 characters)",
            },
            PropDoc {
                name: "color",
                type_str: "Option<Color>",
                doc: "The circle's background color; `None` follows the active theme",
            },
            PropDoc {
                name: "size",
                type_str: "f32",
                doc: "The diameter of the circle, in logical pixels",
            },
        ],
        Story::Badge => &[
            PropDoc {
                name: "label",
                type_str: "String",
                doc: "The text shown inside the badge",
            },
            PropDoc {
                name: "variant",
                type_str: "BadgeVariant",
                doc: "The semantic color variant",
            },
        ],
        Story::Chip => &[
            PropDoc {
                name: "label",
                type_str: "String",
                doc: "The text shown inside the chip",
            },
            PropDoc {
                name: "variant",
                type_str: "BadgeVariant",
                doc: "The semantic color variant",
            },
            PropDoc {
                name: "deletable",
                type_str: "bool",
                doc: "Whether to show a trailing delete glyph",
            },
            PropDoc {
                name: "selected",
                type_str: "bool",
                doc: "Whether this chip is in its \"selected\" state -- caller-owned",
            },
        ],
        Story::List => &[PropDoc {
            name: "children",
            type_str: "ListItem",
            doc: "Each row, added as a normal child",
        }],
        Story::Table => &[
            PropDoc {
                name: "columns",
                type_str: "Vec<TableColumn>",
                doc: "Column definitions",
            },
            PropDoc {
                name: "rows",
                type_str: "TableRows",
                doc: "Data rows",
            },
            PropDoc {
                name: "virtualized",
                type_str: "bool",
                doc: "Only spawns rows visible in an ancestor `ScrollBox`'s viewport",
            },
            PropDoc {
                name: "row_height",
                type_str: "Option<f32>",
                doc: "The fixed height of a single data row, required by `virtualized`",
            },
        ],
        Story::Chart => &[
            PropDoc {
                name: "series",
                type_str: "Vec<ChartSeries>",
                doc: "The series to plot, all sharing one Y scale",
            },
            PropDoc {
                name: "show_legend",
                type_str: "bool",
                doc: "Whether to render a legend row below the plot",
            },
        ],
        Story::BarChart => &[
            PropDoc {
                name: "categories",
                type_str: "Vec<String>",
                doc: "The category labels along the X axis, left to right",
            },
            PropDoc {
                name: "series",
                type_str: "Vec<BarSeries>",
                doc: "The series to plot, one value per category",
            },
            PropDoc {
                name: "mode",
                type_str: "BarMode",
                doc: "Grouped (side-by-side) or stacked bars",
            },
            PropDoc {
                name: "show_legend",
                type_str: "bool",
                doc: "Whether to render a legend row below the plot",
            },
        ],
        Story::AreaChart => &[
            PropDoc {
                name: "series",
                type_str: "Vec<AreaSeries>",
                doc: "The series to plot, each stacked on top of the ones before it",
            },
            PropDoc {
                name: "show_legend",
                type_str: "bool",
                doc: "Whether to render a legend row below the plot",
            },
        ],
        Story::PieChart => &[
            PropDoc {
                name: "slices",
                type_str: "Vec<PieSlice>",
                doc: "The slices to plot, their share computed as a fraction of the total",
            },
            PropDoc {
                name: "inner_radius",
                type_str: "f32",
                doc: "0.0 draws a plain pie; anything greater draws a donut with this hole radius",
            },
            PropDoc {
                name: "center_label",
                type_str: "Option<String>",
                doc: "Text shown in the donut hole -- only meaningful when inner_radius > 0.0",
            },
            PropDoc {
                name: "show_legend",
                type_str: "bool",
                doc: "Whether to render a legend row below the plot",
            },
        ],
        Story::Timeline => &[PropDoc {
            name: "items",
            type_str: "Vec<TimelineItem>",
            doc: "The entries, in order (typically newest- or oldest-first)",
        }],
        Story::TreeView => &[
            PropDoc {
                name: "nodes",
                type_str: "Vec<TreeNode>",
                doc: "The root-level nodes of the tree",
            },
            PropDoc {
                name: "selected_key",
                type_str: "Option<String>",
                doc: "Which node's key is selected, initially",
            },
            PropDoc {
                name: "indent",
                type_str: "f32",
                doc: "Indent per depth level, in logical pixels",
            },
        ],
        Story::Typography => &[
            PropDoc {
                name: "text",
                type_str: "String",
                doc: "The text to render",
            },
            PropDoc {
                name: "variant",
                type_str: "TypographyVariant",
                doc: "Which role of the theme's type scale to use",
            },
            PropDoc {
                name: "color",
                type_str: "Option<Color>",
                doc: "Overrides the variant's default color when set",
            },
        ],
        Story::Markdown => &[
            PropDoc {
                name: "content",
                type_str: "String",
                doc: "The markdown source to render",
            },
            PropDoc {
                name: "code_theme",
                type_str: "String",
                doc: "The syntax-highlight theme name used for fenced code blocks (default \"dracula\")",
            },
        ],
        Story::Divider => &[PropDoc {
            name: "vertical",
            type_str: "bool",
            doc: "Renders as a vertical line instead of the default horizontal one",
        }],
        Story::Tooltip => &[
            PropDoc {
                name: "text",
                type_str: "String",
                doc: "The label text; empty text never shows a label",
            },
            PropDoc {
                name: "placement",
                type_str: "PopoverPlacement",
                doc: "Which side of the trigger the label appears on",
            },
        ],
        Story::Rating => &[
            PropDoc {
                name: "value",
                type_str: "f32",
                doc: "The current value, 0..=`max` in half-star increments",
            },
            PropDoc {
                name: "max",
                type_str: "u8",
                doc: "Number of stars",
            },
            PropDoc {
                name: "read_only",
                type_str: "bool",
                doc: "Disables hover preview and click interaction",
            },
        ],
        Story::Alert => &[
            PropDoc {
                name: "variant",
                type_str: "BadgeVariant",
                doc: "The severity variant, driving the accent border color",
            },
            PropDoc {
                name: "dismissible",
                type_str: "bool",
                doc: "Whether to show a dismiss glyph",
            },
        ],
        Story::ProgressBar => &[PropDoc {
            name: "value",
            type_str: "f32",
            doc: "The current progress, clamped to 0..1",
        }],
        Story::Skeleton => &[
            PropDoc {
                name: "variant",
                type_str: "SkeletonVariant",
                doc: "Which shape to render",
            },
            PropDoc {
                name: "width",
                type_str: "f32",
                doc: "Width, in logical pixels",
            },
            PropDoc {
                name: "height",
                type_str: "f32",
                doc: "Height, in logical pixels",
            },
        ],
        Story::Spinner => &[
            PropDoc {
                name: "size",
                type_str: "f32",
                doc: "Diameter, in logical pixels",
            },
            PropDoc {
                name: "mode",
                type_str: "SpinnerMode",
                doc: "Indeterminate (default) or determinate",
            },
        ],
        Story::Toast => &[],
        Story::Card => &[
            PropDoc {
                name: "title",
                type_str: "Option<String>",
                doc: "Optional header title",
            },
            PropDoc {
                name: "subtitle",
                type_str: "Option<String>",
                doc: "Optional header subtitle, shown under the title",
            },
            PropDoc {
                name: "elevation",
                type_str: "u8",
                doc: "Forwarded to the backing `Paper`'s own `elevation`",
            },
        ],
        Story::Paper => &[PropDoc {
            name: "elevation",
            type_str: "u8",
            doc: "A coarse 0..=4 elevation tier, driving background/border and shadow",
        }],
        Story::Accordion => &[
            PropDoc {
                name: "mode",
                type_str: "AccordionMode",
                doc: "`Multiple` (default) or `Single`",
            },
            PropDoc {
                name: "sections",
                type_str: "AccordionItem",
                doc: "Each expandable section, added as a normal child",
            },
        ],
        Story::AppBar => &[
            PropDoc {
                name: "title",
                type_str: "Option<String>",
                doc: "An optional title shown after the leading slot",
            },
            PropDoc {
                name: "position",
                type_str: "AppBarPosition",
                doc: "Which edge this bar sits against",
            },
            PropDoc {
                name: "height",
                type_str: "f32",
                doc: "Bar height, in pixels",
            },
        ],
        Story::Drawer => &[
            PropDoc {
                name: "open",
                type_str: "bool",
                doc: "Whether the drawer is open; ignored by the `Persistent` variant",
            },
            PropDoc {
                name: "position",
                type_str: "DrawerPosition",
                doc: "Which edge the drawer slides in from",
            },
            PropDoc {
                name: "variant",
                type_str: "DrawerVariant",
                doc: "`Temporary` (overlay, animated) or `Persistent` (inline)",
            },
            PropDoc {
                name: "size",
                type_str: "f32",
                doc: "Width (Left/Right) or height (Top/Bottom), in pixels",
            },
            PropDoc {
                name: "stiffness",
                type_str: "f32",
                doc: "The driving spring's stiffness",
            },
            PropDoc {
                name: "damping",
                type_str: "f32",
                doc: "The driving spring's damping",
            },
        ],
        Story::Modal => &[
            PropDoc {
                name: "title",
                type_str: "String",
                doc: "The text to display in the modal's title bar",
            },
            PropDoc {
                name: "children_styles",
                type_str: "WoodpeckerStyle",
                doc: "Styles to apply to the children element wrapper",
            },
            PropDoc {
                name: "visible",
                type_str: "bool",
                doc: "Is the modal open?",
            },
            PropDoc {
                name: "timeout",
                type_str: "f32",
                doc: "Animation timeout in milliseconds",
            },
            PropDoc {
                name: "overlay_color",
                type_str: "Color",
                doc: "The overlay background alpha value",
            },
            PropDoc {
                name: "transition_play",
                type_str: "bool",
                doc: "State for animation play",
            },
            PropDoc {
                name: "min_size",
                type_str: "Vec2",
                doc: "The min size of the modal",
            },
        ],
        Story::Popover => &[
            PropDoc {
                name: "visible",
                type_str: "bool",
                doc: "Whether the floating content is currently shown",
            },
            PropDoc {
                name: "placement",
                type_str: "PopoverPlacement",
                doc: "Which side of the trigger the content appears on",
            },
        ],
        Story::Menu => &[PropDoc {
            name: "items",
            type_str: "Vec<String>",
            doc: "The items shown in the menu, in order",
        }],
        Story::Masonry => &[
            PropDoc {
                name: "items",
                type_str: "Vec<MasonryItem>",
                doc: "The tiles, in packing order",
            },
            PropDoc {
                name: "columns",
                type_str: "u16",
                doc: "Number of columns",
            },
            PropDoc {
                name: "column_width",
                type_str: "f32",
                doc: "Each column's fixed width, in logical pixels",
            },
            PropDoc {
                name: "gap",
                type_str: "f32",
                doc: "Gap between tiles (both axes), in logical pixels",
            },
        ],
        Story::ImageList => &[
            PropDoc {
                name: "items",
                type_str: "Vec<ImageListItem>",
                doc: "The tiles, in grid order (row-major)",
            },
            PropDoc {
                name: "columns",
                type_str: "u16",
                doc: "Number of columns",
            },
            PropDoc {
                name: "tile_size",
                type_str: "f32",
                doc: "Each (square) tile's side length, in logical pixels",
            },
            PropDoc {
                name: "gap",
                type_str: "f32",
                doc: "Gap between tiles, in logical pixels",
            },
        ],
        Story::VirtualList => &[
            PropDoc {
                name: "item_count",
                type_str: "usize",
                doc: "Total number of items in the list (not how many are mounted)",
            },
            PropDoc {
                name: "item_extent",
                type_str: "f32",
                doc: "The fixed height of a single item, in logical pixels",
            },
            PropDoc {
                name: "item_content",
                type_str: "VirtualListItemContent",
                doc: "Builds the content for the item at a given index",
            },
        ],
        Story::TransferList => &[
            PropDoc {
                name: "left",
                type_str: "Vec<String>",
                doc: "The left panel's items",
            },
            PropDoc {
                name: "right",
                type_str: "Vec<String>",
                doc: "The right panel's items",
            },
            PropDoc {
                name: "left_title",
                type_str: "String",
                doc: "The left panel's title",
            },
            PropDoc {
                name: "right_title",
                type_str: "String",
                doc: "The right panel's title",
            },
        ],
        Story::Splitter => &[PropDoc {
            name: "vertical",
            type_str: "bool",
            doc: "Renders as a vertical, horizontally-draggable line (default) or the reverse",
        }],
        Story::ColorPicker => &[PropDoc {
            name: "initial_color",
            type_str: "Color",
            doc: "Initial color to use",
        }],
        Story::Window => &[
            PropDoc {
                name: "title",
                type_str: "String",
                doc: "The title of the window",
            },
            PropDoc {
                name: "initial_position",
                type_str: "Vec2",
                doc: "Initial position",
            },
            PropDoc {
                name: "visible",
                type_str: "bool",
                doc: "Whether the window is shown at all. Defaults to `true`",
            },
            PropDoc {
                name: "window_styles",
                type_str: "WoodpeckerStyle",
                doc: "Styles for the window widget",
            },
            PropDoc {
                name: "title_styles",
                type_str: "WoodpeckerStyle",
                doc: "Styles for the title",
            },
            PropDoc {
                name: "divider_styles",
                type_str: "WoodpeckerStyle",
                doc: "Styles for the divider under the title",
            },
            PropDoc {
                name: "children_styles",
                type_str: "WoodpeckerStyle",
                doc: "Styles for the children",
            },
        ],
        Story::Dock => &[
            PropDoc {
                name: "initial_tree",
                type_str: "DockTree",
                doc: "The starting layout, seeded once when this DockArea first mounts",
            },
            PropDoc {
                name: "panels",
                type_str: "DockPanels",
                doc: "The panels this dock can show",
            },
        ],
    }
}
