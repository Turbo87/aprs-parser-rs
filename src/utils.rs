pub(crate) fn trim_spaces_end(arr: &mut Vec<u8>) {
    let space_count = arr.iter().rev().take_while(|&&b| b == b' ').count();

    arr.truncate(arr.len() - space_count);
}
