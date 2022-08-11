use bevy::{prelude::*, utils::Uuid};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_kira_audio::{
    prelude::{AudioControl, DynamicAudioChannel, DynamicAudioChannels},
    AudioPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(CameraControllerPlugin)
        .add_startup_system(setup)
        .add_system(pan);

    app.run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct AudioChannel(String);

impl AudioChannel {
    pub fn new(audio: &mut DynamicAudioChannels) -> AudioChannel {
        let id = Uuid::new_v4().to_string();
        audio.create_channel(&id);
        AudioChannel(id)
    }
    pub fn channel<'a>(&'a self, audio: &'a DynamicAudioChannels) -> &DynamicAudioChannel {
        audio.channel(&self.0)
    }
}

fn pan(
    audio: Res<DynamicAudioChannels>,
    query: Query<(&Transform, &AudioChannel)>,
    cam: Query<&Transform, With<MainCamera>>,
) {
    if let Some(cam_trans) = cam.iter().next() {
        for (aud_trans, ch) in &query {
            let ch = ch.channel(&audio);
            let camera_to_aud = (aud_trans.translation - cam_trans.translation).normalize_or_zero();
            // Get dot product between camera right vector, and vector pointing from camera to sound src
            // When pointing at sound src dot is 0.0, when pointing right dot is 1.0, left is -1.0
            let mut pan = cam_trans.right().dot(camera_to_aud);
            pan = pan * 0.5 + 0.5; // pan input expects 0.0 to 1.0
            ch.set_panning(pan as f64);

            let dist = aud_trans.translation.distance(cam_trans.translation) * 0.1;
            let level = (1.0 / dist).min(1.0); // not accurate falloff
            ch.set_volume(level as f64);
        }
    }
}

fn setup(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut audio: ResMut<DynamicAudioChannels>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 15.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // New audio channel for sphere entity
    let new_audio_channel = AudioChannel::new(&mut audio);

    // Play looped sound on sphere entity audio channel
    new_audio_channel
        .channel(&audio)
        .play(ass.load("sounds/loop.ogg"))
        .looped();

    // spawn sphere entity
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 1.0,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 2.5, 0.0),
            ..default()
        })
        .insert(new_audio_channel);

    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 2.5, 10.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..default()
        })
        .insert(CameraController {
            lock_y: true,
            ..default()
        })
        .insert(MainCamera);
}
