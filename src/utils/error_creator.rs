#[macro_export]
macro_rules! error_struct {
    ($struct_name:ident $( { $( $field_name:ident : $field_type:ty ),* $(,)? } )?,
     $format_message:expr $(, $( $format_var:ident ), * $(,)? )?) => {
        pub struct $struct_name {
            $( $( $field_name : $field_type ),*, )?
        }

        impl $struct_name {
            #[allow(dead_code)]
            pub fn new($( $( $field_name : $field_type ),* )?) -> Self {
                $struct_name {
                    $( $( $field_name ),* )?
                }
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $format_message $(, $( self.$format_var ), * )?)
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }

        impl std::error::Error for $struct_name {}
    };
}
