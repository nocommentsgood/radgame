use godot::{classes::Engine, prelude::*};

use crate::components::managers::global_data_singleton::GlobalData;

mod traits;
pub mod utils;
mod classes {
    pub mod characters;
    pub mod components;
    pub mod enemies;
    pub mod scenes;
}
mod components {
    pub mod managers;
    pub mod state_machines;
}

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            // The `&str` identifies your singleton and can be
            // used later to access it.
            Engine::singleton().register_singleton(
                &GlobalData::class_name().to_string_name(),
                &GlobalData::new_alloc(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            // Let's keep a variable of our Engine singleton instance,
            // and MyEngineSingleton name.
            let mut engine = Engine::singleton();
            let singleton_name = &GlobalData::class_name().to_string_name();

            // Here, we manually retrieve our singleton(s) that we've registered,
            // so we can unregister them and free them from memory - unregistering
            // singletons isn't handled automatically by the library.
            if let Some(my_singleton) = engine.get_singleton(singleton_name) {
                // Unregistering from Godot, and freeing from memory is required
                // to avoid memory leaks, warnings, and hot reloading problems.
                engine.unregister_singleton(singleton_name);
                my_singleton.free();
            } else {
                // You can either recover, or panic from here.
                panic!("Failed to get singleton");
            }
        }
    }
}
