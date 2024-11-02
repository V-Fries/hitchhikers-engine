use crate::error_struct;

error_struct!(
    FailedToConvertDescriptorSetsVecToArray {
        vec_len: usize,
        expected_len: usize,
    },
    "Failed to convert descriptor sets vec to array: Vec len was {}, expected {}",
    vec_len,
    expected_len,
);
