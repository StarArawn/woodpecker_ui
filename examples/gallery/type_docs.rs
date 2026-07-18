use crate::props::{PropDoc, TypeDoc};

/// Detail for prop types worth cross-referencing -- supporting structs/enums that appear as
/// another widget's prop type but aren't themselves a top-level `Story`. `linked_type_name`
/// checks any candidate name against this directly, so adding an entry here is the only step
/// needed to make a new type cross-referenceable.
pub(crate) fn type_doc(name: &str) -> Option<TypeDoc> {
    match name {
        "ListItem" => Some(TypeDoc {
            description: "One row within a `List` -- optional leading/trailing content around a primary/secondary text pair, with hover/selected highlighting.",
            fields: &[
                PropDoc { name: "primary_text", type_str: "String", doc: "The row's main text." },
                PropDoc { name: "secondary_text", type_str: "Option<String>", doc: "An optional smaller line under `primary_text`." },
                PropDoc { name: "selected", type_str: "bool", doc: "Whether this row is the current selection." },
                PropDoc { name: "dense", type_str: "bool", doc: "Uses tighter vertical padding when `true`." },
            ],
        }),
        "TabButton" => Some(TypeDoc {
            description: "One clickable tab header within a `Tabs` row -- switches the active tab when clicked.",
            fields: &[
                PropDoc { name: "index", type_str: "usize", doc: "Should match the index of its `TabContent`." },
                PropDoc { name: "title", type_str: "String", doc: "The title of the tab." },
            ],
        }),
        "TabContent" => Some(TypeDoc {
            description: "The panel shown when its matching `TabButton` index is active.",
            fields: &[
                PropDoc { name: "index", type_str: "usize", doc: "Should match its `TabButton`'s index." },
            ],
        }),
        "AccordionItem" => Some(TypeDoc {
            description: "One expandable section within an `Accordion` -- a clickable header (label + disclosure glyph) that toggles a conditionally-shown body.",
            fields: &[
                PropDoc { name: "key", type_str: "String", doc: "A stable identifier, unique within the enclosing `Accordion`." },
                PropDoc { name: "label", type_str: "String", doc: "The header's label text." },
            ],
        }),
        "AccordionMode" => Some(TypeDoc {
            description: "`Accordion`'s two expand/collapse behaviors.",
            fields: &[
                PropDoc { name: "Multiple", type_str: "", doc: "Any number of `AccordionItem`s can be expanded at once (MUI's own default)." },
                PropDoc { name: "Single", type_str: "", doc: "Expanding one item collapses whichever other item was previously expanded." },
            ],
        }),
        "AppBarPosition" => Some(TypeDoc {
            description: "Which edge of its container an `AppBar` sits against -- affects which side gets the dividing border, not actual screen positioning.",
            fields: &[
                PropDoc { name: "Top", type_str: "", doc: "Sits at the top of its container; border on the bottom edge." },
                PropDoc { name: "Bottom", type_str: "", doc: "Sits at the bottom of its container; border on the top edge." },
            ],
        }),
        "BadgeVariant" => Some(TypeDoc {
            description: "A semantic color variant for `Badge` (and reused by `Chip`/`Alert`).",
            fields: &[
                PropDoc { name: "Neutral", type_str: "", doc: "Neutral gray -- the default, for status that isn't good/bad/risky." },
                PropDoc { name: "Success", type_str: "", doc: "Green -- positive/active/success status." },
                PropDoc { name: "Danger", type_str: "", doc: "Red -- negative/error/inactive status." },
                PropDoc { name: "Warning", type_str: "", doc: "Amber -- caution/pending status." },
                PropDoc { name: "Info", type_str: "", doc: "The library's accent blue -- informational status." },
            ],
        }),
        "CalendarDate" => Some(TypeDoc {
            description: "A plain calendar date -- deliberately self-contained (no wall-clock \"today\", no timezone) since `DatePicker` only needs to display and compare dates a caller hands it.",
            fields: &[
                PropDoc { name: "year", type_str: "i32", doc: "The calendar year (astronomical numbering)." },
                PropDoc { name: "month", type_str: "u8", doc: "The month, 1..=12." },
                PropDoc { name: "day", type_str: "u8", doc: "The day of month, 1..=31." },
            ],
        }),
        "ComboBoxMatchMode" => Some(TypeDoc {
            description: "How `ComboBox` matches its typed filter text against each item, case-insensitively.",
            fields: &[
                PropDoc { name: "Contains", type_str: "", doc: "The item contains the filter text anywhere." },
                PropDoc { name: "StartsWith", type_str: "", doc: "The item starts with the filter text." },
            ],
        }),
        "DrawerPosition" => Some(TypeDoc {
            description: "Which edge of the screen a `Drawer` slides in from.",
            fields: &[
                PropDoc { name: "Left", type_str: "", doc: "Slides in from the left edge." },
                PropDoc { name: "Right", type_str: "", doc: "Slides in from the right edge." },
                PropDoc { name: "Top", type_str: "", doc: "Slides in from the top edge." },
                PropDoc { name: "Bottom", type_str: "", doc: "Slides in from the bottom edge." },
            ],
        }),
        "DrawerVariant" => Some(TypeDoc {
            description: "`Drawer`'s two behavior modes.",
            fields: &[
                PropDoc { name: "Temporary", type_str: "", doc: "Portaled overlay with a scrim, opened/closed via `Drawer::open`, sliding in with a spring." },
                PropDoc { name: "Persistent", type_str: "", doc: "Renders inline, in normal layout flow, always visible -- no scrim, no portal, no transition." },
            ],
        }),
        "PopoverPlacement" => Some(TypeDoc {
            description: "Which side of the trigger a `Popover`'s floating content appears on.",
            fields: &[
                PropDoc { name: "Bottom", type_str: "", doc: "Directly below the trigger, left-edge aligned." },
                PropDoc { name: "Top", type_str: "", doc: "Directly above the trigger, left-edge aligned." },
                PropDoc { name: "Left", type_str: "", doc: "Directly to the left of the trigger, top-edge aligned." },
                PropDoc { name: "Right", type_str: "", doc: "Directly to the right of the trigger, top-edge aligned." },
            ],
        }),
        "SkeletonVariant" => Some(TypeDoc {
            description: "`Skeleton`'s three shapes.",
            fields: &[
                PropDoc { name: "Rect", type_str: "", doc: "A rounded rectangle -- a card, an image, a generic block." },
                PropDoc { name: "Circle", type_str: "", doc: "A full circle -- an avatar placeholder." },
                PropDoc { name: "Text", type_str: "", doc: "A thin pill -- one line of text." },
            ],
        }),
        "SpinnerMode" => Some(TypeDoc {
            description: "`Spinner`'s two animation modes.",
            fields: &[
                PropDoc { name: "Indeterminate", type_str: "", doc: "A rotating arc animated continuously from wall-clock time -- unknown-duration work." },
                PropDoc { name: "Determinate", type_str: "f32", doc: "A fixed arc sweeping clockwise from the top, proportional to the given value (clamped 0..=1)." },
            ],
        }),
        "StepperOrientation" => Some(TypeDoc {
            description: "`Stepper`'s two layouts.",
            fields: &[
                PropDoc { name: "Horizontal", type_str: "", doc: "Steps flow left-to-right, connected by horizontal lines." },
                PropDoc { name: "Vertical", type_str: "", doc: "Steps flow top-to-bottom, connected by vertical lines, label to the circle's right." },
            ],
        }),
        "TabMode" => Some(TypeDoc {
            description: "`TextBox`'s tab-key behavior. Defaults to 4 spaces.",
            fields: &[
                PropDoc { name: "Tab", type_str: "", doc: "Inserts a tab character." },
                PropDoc { name: "Space", type_str: "u8", doc: "Inserts this many space characters; defaults to 4." },
            ],
        }),
        "TableColumn" => Some(TypeDoc {
            description: "One column definition within a `Table`.",
            fields: &[
                PropDoc { name: "header", type_str: "String", doc: "The header label." },
                PropDoc { name: "width", type_str: "f32", doc: "A fixed width in logical pixels, or 0.0 to grow and fill remaining space." },
            ],
        }),
        "TableRows" => Some(TypeDoc {
            description: "Wraps a `Table`'s data rows in an `Arc`, so an otherwise-unchanged `Table` compares in O(1) across renders instead of a full deep comparison every frame.",
            fields: &[
                PropDoc { name: "0", type_str: "Arc<Vec<TableRow>>", doc: "The rows, each holding one string per column, in column order." },
            ],
        }),
        "TypographyVariant" => Some(TypeDoc {
            description: "Which role of the theme's type scale `Typography` resolves its font size from.",
            fields: &[
                PropDoc { name: "H1", type_str: "", doc: "Large page/section heading." },
                PropDoc { name: "H2", type_str: "", doc: "Sub-heading / card title." },
                PropDoc { name: "H3", type_str: "", doc: "Small heading / emphasized label." },
                PropDoc { name: "Body", type_str: "", doc: "Default body text." },
                PropDoc { name: "BodySmall", type_str: "", doc: "Secondary/de-emphasized body text." },
                PropDoc { name: "Caption", type_str: "", doc: "Smallest legible text." },
            ],
        }),
        "BottomNavItem" => Some(TypeDoc {
            description: "One destination within a `BottomNavigation` bar.",
            fields: &[
                PropDoc { name: "icon", type_str: "String", doc: "A glyph shown above the label (rendered via the icon font)." },
                PropDoc { name: "label", type_str: "String", doc: "The destination's label." },
            ],
        }),
        "ChartSeries" => Some(TypeDoc {
            description: "One line series plotted by a `Chart`.",
            fields: &[
                PropDoc { name: "name", type_str: "String", doc: "The series name, shown in the legend." },
                PropDoc { name: "data", type_str: "Vec<f32>", doc: "The data points to plot, left to right." },
                PropDoc { name: "color", type_str: "Color", doc: "The color of this series' line, fill, and legend swatch." },
            ],
        }),
        "ImageListItem" => Some(TypeDoc {
            description: "One tile within an `ImageList`.",
            fields: &[
                PropDoc { name: "handle", type_str: "Handle<Image>", doc: "The image to display." },
                PropDoc { name: "label", type_str: "Option<String>", doc: "An optional caption, overlaid on a scrim at the tile's bottom edge." },
            ],
        }),
        "MasonryItem" => Some(TypeDoc {
            description: "One tile within a `Masonry` layout.",
            fields: &[
                PropDoc { name: "handle", type_str: "Handle<Image>", doc: "The image to display." },
                PropDoc { name: "height", type_str: "f32", doc: "The tile's rendered height, in logical pixels, at the masonry's column width." },
                PropDoc { name: "label", type_str: "Option<String>", doc: "An optional caption, overlaid on a scrim at the tile's bottom edge." },
            ],
        }),
        "NavigationRailItem" => Some(TypeDoc {
            description: "One destination within a `NavigationRail`.",
            fields: &[
                PropDoc { name: "icon", type_str: "String", doc: "A glyph (rendered via the icon font)." },
                PropDoc { name: "label", type_str: "String", doc: "The destination's label, shown as a small caption under the icon." },
            ],
        }),
        "SpeedDialAction" => Some(TypeDoc {
            description: "One action button shown when a `SpeedDial` is expanded.",
            fields: &[
                PropDoc { name: "icon", type_str: "String", doc: "A glyph shown on the action's own round button (rendered via the icon font)." },
            ],
        }),
        "TimelineItem" => Some(TypeDoc {
            description: "One entry within a `Timeline`.",
            fields: &[
                PropDoc { name: "title", type_str: "String", doc: "The entry's main line." },
                PropDoc { name: "subtitle", type_str: "Option<String>", doc: "An optional smaller line under `title` (typically a timestamp)." },
                PropDoc { name: "dot_color", type_str: "Option<Color>", doc: "Overrides the theme's default dot color for this entry's dot." },
            ],
        }),
        "TreeNode" => Some(TypeDoc {
            description: "One row within a `TreeView`, recursively nesting further `TreeNode`s.",
            fields: &[
                PropDoc { name: "key", type_str: "String", doc: "A stable identifier, unique across the whole tree." },
                PropDoc { name: "label", type_str: "String", doc: "The row's display text." },
                PropDoc { name: "children", type_str: "Vec<TreeNode>", doc: "Nested nodes. An empty `Vec` means this row has no disclosure triangle." },
            ],
        }),
        _ => None,
    }
}
