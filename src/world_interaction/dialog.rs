use crate::{
    level_instantiation::spawning::objects::camera::IngameCameraMarker,
    player_control::actions::ActionsFrozen,
};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_yarnspinner::{events::DialogueCompleteEvent, prelude::*};
use bevy_yarnspinner_example_dialogue_view::{prelude::*, UiRootNode};
use serde::{Deserialize, Serialize};

pub(crate) fn dialog_plugin(app: &mut App) {
    app.add_plugins((
        EguiPlugin,
        YarnSpinnerPlugin::new(),
        ExampleYarnSpinnerDialogueViewPlugin::new(),
    ))
    .add_systems(
        Update,
        (
            spawn_dialogue_runner.run_if(resource_added::<YarnProject>),
            unfreeze_after_dialog,
            set_ui_target_camera,
        )
            .after(ExampleYarnSpinnerDialogueViewSystemSet),
    );
}

#[derive(Component, Debug, Clone, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub(crate) struct DialogTarget {
    pub(crate) speaker: String,
    pub(crate) node: String,
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let dialogue_runner = project.create_dialogue_runner();
    // Immediately start showing the dialogue to the player
    commands.spawn(dialogue_runner);
}

fn unfreeze_after_dialog(
    mut dialogue_complete_event: EventReader<DialogueCompleteEvent>,
    mut freeze: ResMut<ActionsFrozen>,
) {
    for _event in dialogue_complete_event.read() {
        freeze.unfreeze();
    }
}

fn set_ui_target_camera(
    mut commands: Commands,
    root_ui_node: Query<Entity, (With<UiRootNode>, Without<TargetCamera>)>,
    main_camera: Query<Entity, With<IngameCameraMarker>>,
) {
    for camera_entity in main_camera.iter() {
        for node_entity in root_ui_node.iter() {
            commands
                .entity(node_entity)
                .insert(TargetCamera(camera_entity));
        }
    }
}
