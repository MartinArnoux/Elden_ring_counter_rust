pub mod language;
pub mod translations;
pub use language::Language;
#[macro_export]
macro_rules! t {
    ($app:expr, $key:ident) => {
        $app.i18n.get($crate::i18n::TranslationKey::$key)
    };
}
