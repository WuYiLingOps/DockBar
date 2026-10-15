macro_rules! define_app_errors {
    ($(
        $variant:ident($error_type:ty);
    )*) => {
        #[derive(Debug)]
        pub enum DockBarLibError {
            $(
                $variant($error_type),
            )*
        }

        $(
            impl From<$error_type> for DockBarLibError {
                fn from(err: $error_type) -> Self {
                    DockBarLibError::$variant(err)
                }
            }
        )*
    };
}

define_app_errors!(
    Custom(String);
    Io(std::io::Error);
    SerdeJson(serde_json::Error);
    SerdeYaml(serde_yaml::Error);
    Base64Decode(base64::DecodeError);
    Grass(Box<grass::Error>);
);

impl From<&str> for DockBarLibError {
    fn from(err: &str) -> Self {
        err.to_owned().into()
    }
}

impl std::fmt::Display for DockBarLibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type Result<T, E = DockBarLibError> = std::result::Result<T, E>;
