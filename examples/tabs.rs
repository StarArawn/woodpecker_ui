use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            bevy_inspector_egui::bevy_egui::EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ))
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let mut tab_buttons = WidgetChildren::default();
    let mut tab_content = WidgetChildren::default();

    let lorem_ipsum = r#"
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sed tellus neque. Proin tempus ligula a mi molestie aliquam. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam venenatis consequat ultricies. Sed ac orci purus. Nullam velit nisl, dapibus vel mauris id, dignissim elementum sapien. Vestibulum faucibus sapien ut erat bibendum, id lobortis nisi luctus. Mauris feugiat at lectus at pretium. Pellentesque vitae finibus ante. Nulla non ex neque. Cras varius, lorem facilisis consequat blandit, lorem mauris mollis massa, eget consectetur magna sem vel enim. Nam aliquam risus pulvinar, volutpat leo eget, eleifend urna. Suspendisse in magna sed ligula vehicula volutpat non vitae augue. Phasellus aliquam viverra consequat. Nam rhoncus molestie purus, sed laoreet neque imperdiet eget. Sed egestas metus eget sodales congue.
    
     Sed vel ante placerat, posuere lacus sit amet, tempus enim. Cras ullamcorper ex vitae metus consequat, a blandit leo semper. Nunc lacinia porta massa, a tempus leo laoreet nec. Sed vel metus tincidunt, scelerisque ex sit amet, lacinia dui. In sollicitudin pulvinar odio vitae hendrerit. Maecenas mollis tempor egestas. Nulla facilisi. Praesent nisi turpis, accumsan eu lobortis vestibulum, ultrices id nibh. Suspendisse sed dui porta, mollis elit sed, ornare sem. Cras molestie est libero, quis faucibus leo semper at.
    
     Nulla vel nisl rutrum, fringilla elit non, mollis odio. Donec convallis arcu neque, eget venenatis sem mattis nec. Nulla facilisi. Phasellus risus elit, vehicula sit amet risus et, sodales ultrices est. Quisque vulputate felis orci, non tristique leo faucibus in. Duis quis velit urna. Sed rhoncus dolor vel commodo aliquet. In sed tempor quam. Nunc non tempus ipsum. Praesent mi lacus, vehicula eu dolor eu, condimentum venenatis diam. In tristique ligula a ligula dictum, eu dictum lacus consectetur. Proin elementum egestas pharetra. Nunc suscipit dui ac nisl maximus, id congue velit volutpat. Etiam condimentum, mauris ac sodales tristique, est augue accumsan elit, ut luctus est mi ut urna. Mauris commodo, tortor eget gravida lacinia, leo est imperdiet arcu, a ullamcorper dui sapien eget erat.
    
     Vivamus pulvinar dui et elit volutpat hendrerit. Praesent luctus dolor ut rutrum finibus. Fusce ut odio ultrices, laoreet est at, condimentum turpis. Morbi at ultricies nibh. Mauris tempus imperdiet porta. Proin sit amet tincidunt eros. Quisque rutrum lacus ac est vehicula dictum. Pellentesque nec augue mi.
    
     Vestibulum rutrum imperdiet nisl, et consequat massa porttitor vel. Ut velit justo, vehicula a nulla eu, auctor eleifend metus. Ut egestas malesuada metus, sit amet pretium nunc commodo ac. Pellentesque gravida, nisl in faucibus volutpat, libero turpis mattis orci, vitae tincidunt ligula ligula ut tortor. Maecenas vehicula lobortis odio in molestie. Curabitur dictum elit sed arcu dictum, ut semper nunc cursus. Donec semper felis non nisl tincidunt elementum.
        "#.to_string();

    for i in 0..2 {
        tab_buttons.add::<TabButton>(TabButtonBundle {
            tab_button: TabButton {
                index: i,
                title: format!("Tab {}", i + 1),
                ..Default::default()
            },
            ..Default::default()
        });
        tab_content.add::<TabContent>(TabContentBundle {
            tab_content: TabContent { index: i },
            children: PassedChildren(
                WidgetChildren::default().with_child::<ScrollContextProvider>((
                    ScrollContextProviderBundle {
                        styles: WoodpeckerStyle {
                            margin: Edge::all(10.0).left(10.0).right(0.0).bottom(10.0),
                            width: Units::Percentage(100.0),
                            height: 200.0.into(),
                            ..Default::default()
                        },
                        children: WidgetChildren::default().with_child::<ScrollBox>(
                            ScrollBoxBundle {
                                children: PassedChildren(
                                    WidgetChildren::default().with_child::<Element>((
                                        Element,
                                        WoodpeckerStyle {
                                            font_size: 14.0,
                                            color: Srgba::WHITE.into(),
                                            ..Default::default()
                                        },
                                        WidgetRender::Text {
                                            content: lorem_ipsum.clone(),
                                            word_wrap: true,
                                        },
                                    )),
                                ),
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    },
                )),
            ),
            ..Default::default()
        });
    }

    tab_buttons.add::<TabButton>(TabButtonBundle {
        tab_button: TabButton {
            index: 2,
            title: format!("Tab {}", 3),
            ..Default::default()
        },
        ..Default::default()
    });
    tab_content.add::<TabContent>(TabContentBundle {
        tab_content: TabContent { index: 2 },
        children: PassedChildren(WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(10.0),
                width: Units::Auto,
                height: Units::Pixels(200.0),
                ..Default::default()
            },
            WidgetRender::Svg {
                handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                color: Some(Srgba::RED.into()),
            },
        ))),
        ..Default::default()
    });

    let root = commands
        .spawn((
            WoodpeckerApp,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetAlignContent::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<TabContextProvider>((
                TabContextProviderBundle {
                    children: PassedChildren(
                        WidgetChildren::default()
                            .with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    flex_direction: WidgetFlexDirection::Row,
                                    ..Default::default()
                                },
                                tab_buttons,
                            ))
                            .with_child::<Element>((Element, tab_content)),
                    ),
                    ..Default::default()
                },
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
