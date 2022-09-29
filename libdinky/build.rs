#[cfg(feature = "gl_generator")]
extern crate gl_generator;
#[cfg(feature = "gl_generator")]
use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
#[cfg(feature = "gl_generator")]
use std::{env, fs::File, path::Path};

fn main() {
    #[cfg(feature = "gl_generator")]
    {
        let dest = env::var("OUT_DIR").unwrap();
        let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

        Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, [])
            .write_bindings(GlobalGenerator, &mut file)
            .unwrap();
    }
}
