//! Erzeugt die Swift-Anbindung aus der gebauten lot-ffi-Bibliothek.
//! Aufruf über `app-mac/scripts/build-core.sh`.

fn main() {
    uniffi::uniffi_bindgen_main()
}
