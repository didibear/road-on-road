use bevy::prelude::*;

use rand::seq::SliceRandom;

pub fn play_sound(sound: &Handle<AudioSource>) -> AudioSourceBundle {
    AudioBundle {
        source: sound.clone(),
        settings: PlaybackSettings::DESPAWN,
    }
}

pub fn play_random_sound(sounds: &Vec<Handle<AudioSource>>) -> AudioSourceBundle {
    let mut rng = rand::thread_rng();
    let sound = sounds.choose(&mut rng).expect("Asset sound");
    play_sound(sound)
}
