use rs42::error_struct_custom_display;

error_struct_custom_display!(
    FailedToConvertDescriptorSetsVecToArray {
        vec_len: usize,
        expected_len: usize,
    },
    "Failed to convert descriptor sets vec to array: Vec len was {}, expected {}",
    vec_len,
    expected_len,
);
