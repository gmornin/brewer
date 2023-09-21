use cmdarg_macro_derive::CmdArg;
use goodmorning_bindings::services::v1::ItemVisibility;

#[derive(Debug, PartialEq, Clone, Copy, CmdArg)]
pub enum Visibility {
    Public,
    Hidden,
    Private,
    Inherit,
}

impl From<Visibility> for ItemVisibility {
    fn from(val: Visibility) -> Self {
        match val {
            Visibility::Public => ItemVisibility::Public,
            Visibility::Hidden => ItemVisibility::Hidden,
            Visibility::Private => ItemVisibility::Private,
            Visibility::Inherit => unreachable!("unimplemented does not map to a visibility item"),
        }
    }
}
