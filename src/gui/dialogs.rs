use fltk::dialog;

/// ファイルパスを取得するファイルオープンダイアログ
pub fn open_file_dialog(_title: &str, _filter: &str) -> Option<String> {
    let mut chooser = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
    chooser.show();
    chooser.filename().to_str().map(|s| s.to_string())
}

/// 保存先ファイルパスを取得するダイアログ
pub fn save_file_dialog(
    _title: &str,
    _filter: &str,
    _default_name: &str,
) -> Option<String> {
    let mut chooser = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseSaveFile);
    chooser.show();
    chooser.filename().to_str().map(|s| s.to_string())
}

/// ディレクトリ選択ダイアログ
pub fn choose_directory_dialog(_title: &str) -> Option<String> {
    let mut chooser = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseDir);
    chooser.show();
    chooser.filename().to_str().map(|s| s.to_string())
}

/// 確認ダイアログ
pub fn confirm_dialog(title: &str, message: &str) -> bool {
    dialog::choice2(0, 0, title, "はい", "いいえ", "") == Some(0)
}

/// 情報ダイアログ
pub fn info_dialog(title: &str, _message: &str) {
    dialog::message(0, 0, title);
}

/// エラーダイアログ
pub fn error_dialog(title: &str, _message: &str) {
    dialog::alert(0, 0, title);
}