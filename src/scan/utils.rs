//! 扫描相关的工具函数

use std::path::PathBuf;
use crate::scan::patterns::{
    VERSION_PATTERNS, PREFIX_PATTERNS, VERSION_REMOVAL_PATTERNS,
    PLATFORM_PATTERNS, SUFFIX_PATTERNS,
};

/// 计算目录大小（异步版本，使用迭代而非递归避免栈溢出）
///
/// # 参数
/// - `dir_path`: 要计算大小的目录路径
///
/// # 返回
/// 目录的总大小（字节）
pub async fn calculate_directory_size_async(dir_path: PathBuf) -> u64 {
    use tokio::fs;

    let mut total_size = 0u64;
    let mut stack = vec![dir_path];

    while let Some(path) = stack.pop() {
        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    match entry.metadata().await {
                        Ok(metadata) => {
                            if metadata.is_file() {
                                total_size += metadata.len();
                            } else if metadata.is_dir() {
                                stack.push(entry.path());
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => continue,
        }
    }

    total_size
}

pub async fn find_icon_path(
    dir_path: PathBuf,
    child_paths: &[String],
    try_extract_exe_icon: bool,
    dir_name_hint: &str,
    preferred_exe_rel: Option<&str>,
) -> Option<String> {
    use tokio::fs;
    // helper to check a relative candidate exists as file
    async fn exists_file(dir: &PathBuf, rel: &std::path::Path) -> bool {
        let abs = dir.join(rel);
        match fs::metadata(&abs).await {
            Ok(m) => m.is_file(),
            Err(_) => false,
        }
    }
    // Build candidate order: png first, then ico; prefer exe parent, then root
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    fn score_exe(path: &str, dir_name_hint: &str) -> i32 {
        let lower = path.to_lowercase();
        let fname = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mut score = 0;
        let binding = dir_name_hint.to_lowercase();
        let tokens: Vec<&str> = binding
            .split(|c: char| c.is_whitespace() || c == '_' || c == '-' || c == '・' || c == '～')
            .filter(|t| !t.is_empty())
            .collect();
        if tokens.iter().any(|t| fname.contains(t)) {
            score += 15;
        }
        let blocklist = [
            "ueprereqsetup", "vc_redist", "unitycrashhandler", "crashpad_handler",
            "chromedriver", "notification_helper", "payload", "zfgamebrowser",
            "redist", "extras", "setup",
        ];
        if blocklist.iter().any(|b| lower.contains(b)) {
            score -= 25;
        }
        // prefer shallower paths
        let depth = lower.matches('/').count() + lower.matches('\\').count();
        score -= depth as i32;
        // prefer .exe at root
        if !lower.contains('/') && !lower.contains('\\') {
            score += 5;
        }
        score
    }
    let first_exe_rel = if let Some(p) = preferred_exe_rel {
        Some(p.to_string())
    } else {
        let mut exe_list: Vec<(i32, String)> = child_paths
            .iter()
            .filter(|p| p.to_lowercase().ends_with(".exe"))
            .map(|p| (score_exe(p, dir_name_hint), p.clone()))
            .collect();
        exe_list.sort_by(|a, b| b.0.cmp(&a.0));
        exe_list.first().map(|(_, p)| p.clone())
    };
    if let Some(exe_rel) = first_exe_rel.as_ref() {
        let mut exe_rel_path = std::path::PathBuf::new();
        for comp in exe_rel.split(|c| c == '/' || c == '\\') {
            exe_rel_path.push(comp);
        }
        if let Some(parent_rel) = exe_rel_path.parent() {
            candidates.push(parent_rel.join("icon").join("icon.png"));
            candidates.push(parent_rel.join("icon").join("icon.ico"));
            candidates.push(parent_rel.join("icon.png"));
            candidates.push(parent_rel.join("icon.ico"));
        }
    }
    candidates.push(std::path::PathBuf::from("icon").join("icon.png"));
    candidates.push(std::path::PathBuf::from("icon").join("icon.ico"));
    candidates.push(std::path::PathBuf::from("icon.png"));
    candidates.push(std::path::PathBuf::from("icon.ico"));

    for rel in candidates.iter() {
        if exists_file(&dir_path, rel).await {
            let s = rel.to_string_lossy().to_string();
            #[cfg(target_os = "windows")]
            {
                return Some(s.replace("/", "\\"));
            }
            #[cfg(not(target_os = "windows"))]
            {
                return Some(s);
            }
        }
    }

    #[cfg(all(target_os = "windows", feature = "win_exe_icon"))]
    {
        if try_extract_exe_icon {
            if let Some(exe_rel) = first_exe_rel.clone() {
                if let Ok(result) = tokio::task::spawn_blocking({
                    let dir_path_clone = dir_path.clone();
                    let exe_rel_clone = exe_rel.clone();
                    move || extract_exe_icon_to_root_sync(&dir_path_clone, &exe_rel_clone)
                }).await {
                    if let Some(file_name) = result {
                        return Some(file_name);
                    }
                }
            }
        }
    }

    None
}

#[cfg(all(target_os = "windows", feature = "win_exe_icon"))]
fn extract_exe_icon_to_root_sync(dir_path: &std::path::PathBuf, exe_rel: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO, HICON, DrawIconEx, DI_NORMAL, GetSystemMetrics};
    use windows::Win32::UI::Shell::ExtractIconExW;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, CreateDIBSection,
        BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
    };
    let mut exe_abs = dir_path.clone();
    for comp in exe_rel.split(|c| c == '/' || c == '\\') {
        exe_abs.push(comp);
    }
    if !exe_abs.exists() {
        return None;
    }
    let mut wpath: Vec<u16> = OsStr::new(&exe_abs.as_os_str())
        .encode_wide()
        .collect();
    wpath.push(0);
    let mut sfi = SHFILEINFOW::default();
    let flags = SHGFI_ICON | SHGFI_LARGEICON;
    unsafe {
        let res = SHGetFileInfoW(
            windows::core::PCWSTR(wpath.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if res == 0 {
            return None;
        }
        let hicon: HICON = sfi.hIcon;
        
        // Try hbmColor path first
        let mut icon_info = ICONINFO::default();
        let mut saved = false;
        if GetIconInfo(hicon, &mut icon_info).is_ok() {
            let hbm_color = icon_info.hbmColor;
            let mut bmp = BITMAP::default();
            if GetObjectW(hbm_color, std::mem::size_of::<BITMAP>() as i32, Some(&mut bmp as *mut _ as *mut std::ffi::c_void)) != 0 {
                let width = bmp.bmWidth as i32;
                let height = bmp.bmHeight as i32;
                let mut bih = BITMAPINFOHEADER::default();
                bih.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bih.biWidth = width;
                bih.biHeight = -height; // top-down
                bih.biPlanes = 1;
                bih.biBitCount = 32;
                bih.biCompression = BI_RGB.0;
                let mut bi = BITMAPINFO::default();
                bi.bmiHeader = bih;
                let hdc = CreateCompatibleDC(None);
                if hdc.0 != 0 {
                    let prev = SelectObject(hdc, hbm_color);
                    let mut buffer: Vec<u8> = vec![0u8; (width * height * 4) as usize];
                    let got = GetDIBits(
                        hdc,
                        hbm_color,
                        0,
                        height as u32,
                        Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                        &mut bi,
                        DIB_RGB_COLORS,
                    );
                    SelectObject(hdc, prev);
                    DeleteDC(hdc);
                    DeleteObject(hbm_color);
                    DeleteObject(icon_info.hbmMask);
                    if got != 0 {
                        for px in buffer.chunks_mut(4) { px.swap(0, 2); }
                        let rgba = buffer;
                        if let Some(img) = image::RgbaImage::from_raw(width as u32, height as u32, rgba) {
                            let out_path = dir_path.join("icon.png");
                            if img.save(&out_path).is_ok() {
                                saved = true;
                            }
                        }
                    }
                }
            } else {
                DeleteObject(hbm_color);
                DeleteObject(icon_info.hbmMask);
            }
        }
        // Fallback: draw HICON into DIB and save
        if !saved {
            let cx = unsafe { GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX(11)) };
            let cy = unsafe { GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX(12)) };
            let width = cx.max(64);
            let height = cy.max(64);
            let mut try_hicon = hicon;
            if try_hicon.0 == 0 {
                let mut h_large: HICON = HICON(0);
                let mut h_small: HICON = HICON(0);
                let cnt = unsafe {
                    ExtractIconExW(
                        windows::core::PCWSTR(wpath.as_ptr()),
                        0,
                        Some(&mut h_large),
                        Some(&mut h_small),
                        1,
                    )
                };
                if cnt > 0 {
                    if h_large.0 != 0 {
                        try_hicon = h_large;
                    } else if h_small.0 != 0 {
                        try_hicon = h_small;
                    }
                }
            }
            let mut bih = BITMAPINFOHEADER::default();
            bih.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bih.biWidth = width;
            bih.biHeight = -(height as i32); // top-down
            bih.biPlanes = 1;
            bih.biBitCount = 32;
            bih.biCompression = BI_RGB.0;
            let mut bi = BITMAPINFO::default();
            bi.bmiHeader = bih;
            let hdc = CreateCompatibleDC(None);
            if hdc.0 != 0 {
                let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                let hbmp: HBITMAP = unsafe { CreateDIBSection(hdc, &bi, DIB_RGB_COLORS, &mut bits_ptr, None, 0).expect("CreateDIBSection failed") };
                if hbmp.0 != 0 && !bits_ptr.is_null() {
                    let prev = SelectObject(hdc, hbmp);
                    let _ = unsafe { DrawIconEx(hdc, 0, 0, try_hicon, width, height, 0, None, DI_NORMAL) };
                    // copy buffer
                    let len = (width * height * 4) as usize;
                    let mut buffer: Vec<u8> = vec![0u8; len];
                    unsafe {
                        std::ptr::copy_nonoverlapping(bits_ptr as *const u8, buffer.as_mut_ptr(), len);
                    }
                    SelectObject(hdc, prev);
                    DeleteObject(hbmp);
                    DeleteDC(hdc);
                    let mut rgba = buffer;
                    for px in rgba.chunks_mut(4) { px.swap(0, 2); }
                    if let Some(img) = image::RgbaImage::from_raw(width as u32, height as u32, rgba) {
                        let out_dir = dir_path.join("icon");
                        let _ = std::fs::create_dir_all(&out_dir);
                        let out_path = out_dir.join("icon.png");
                        if img.save(&out_path).is_ok() {
                            let _ = DestroyIcon(hicon);
                            return Some(String::from("icon\\icon.png"));
                        }
                    }
                } else {
                    if hbmp.0 != 0 { DeleteObject(hbmp); }
                    DeleteDC(hdc);
                }
            }
        }
        let _ = DestroyIcon(hicon);
        None
    }
}
/// 从游戏目录名中提取版本号
///
/// 支持以下格式：
/// - `ver.1.0`, `ver 1.0`, `v.1.0`, `v 1.0`
/// - `_1.0`, `_1.0.0`
/// - `1.0`, `1.0.0` (结尾)
///
/// # 参数
/// - `dir_name`: 目录名称
///
/// # 返回
/// 提取到的版本号，如果没有找到则返回 `None`
pub fn extract_version(dir_name: &str) -> Option<String> {
    // 使用预编译的正则表达式（避免重复编译）
    for re in VERSION_PATTERNS.iter() {
        if let Some(captures) = re.captures(dir_name) {
            if let Some(version) = captures.get(1) {
                return Some(version.as_str().to_string());
            }
        }
    }

    None
}

/// 从游戏目录名中提取搜索关键词
///
/// 去除常见的前缀标签和版本号，如：【RPG官中】、【SLG汉化】、v1.0 等
///
/// # 参数
/// - `dir_name`: 目录名称
///
/// # 返回
/// 清理后的搜索关键词
///
/// # 示例
/// ```
/// use scanners::scan::extract_search_key;
///
/// let key = extract_search_key("【RPG官中】游戏名称 v1.0");
/// assert_eq!(key, "游戏名称");
/// ```
pub fn extract_search_key(dir_name: &str) -> String {
    let mut result = dir_name.to_string();

    // 1. 移除前缀标签（使用预编译的正则表达式）
    for re in PREFIX_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 2. 移除版本号（使用预编译的正则表达式）
    for re in VERSION_REMOVAL_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 3. 移除平台标识（使用预编译的正则表达式）
    for re in PLATFORM_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 4. 移除常见的后缀（使用预编译的正则表达式）
    for re in SUFFIX_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 5. 清理多余的空白和特殊字符
    result = result.trim().to_string();

    // 移除末尾的下划线、空格、点号、波浪号
    while result.ends_with('_') || result.ends_with(' ') || result.ends_with('.') || result.ends_with('~') {
        result.pop();
    }

    result = result.trim().to_string();

    // 如果结果为空，返回原始名称
    if result.is_empty() {
        dir_name.to_string()
    } else {
        result
    }
}

/// 找到一组路径的最近公共父目录（不包括文件名）
///
/// # 参数
/// - `paths`: 路径组件的向量列表
///
/// # 返回
/// 公共父目录的长度（组件数量）
///
/// # 示例
/// ```
/// use scanners::scan::find_common_parent_dir;
///
/// let paths = vec![
///     vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
///     vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "data".to_string(), "game.exe".to_string()],
/// ];
/// let common_len = find_common_parent_dir(&paths);
/// assert_eq!(common_len, 3); // C:\Games\Game1
/// ```
pub fn find_common_parent_dir(paths: &[Vec<String>]) -> usize {
    if paths.is_empty() {
        return 0;
    }

    // 找到最短路径的长度（排除文件名，所以 -1）
    let min_len = paths.iter().map(|p| p.len().saturating_sub(1)).min().unwrap_or(0);

    let mut common_len = 0;
    for i in 0..min_len {
        let component = &paths[0][i];

        // 检查所有路径在这个位置是否都有相同的组件
        if paths.iter().all(|p| i < p.len() && &p[i] == component) {
            common_len = i + 1;
        } else {
            break;
        }
    }

    common_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("Game v1.0"), Some("1.0".to_string()));
        assert_eq!(extract_version("Game ver.2.1.3"), Some("2.1.3".to_string()));
        assert_eq!(extract_version("Game_1.5"), Some("1.5".to_string()));
        assert_eq!(extract_version("Game 1.0.0"), Some("1.0.0".to_string()));
        assert_eq!(extract_version("Game"), None);
    }

    #[test]
    fn test_extract_search_key() {
        assert_eq!(extract_search_key("【RPG官中】游戏名称 v1.0"), "游戏名称");
        assert_eq!(extract_search_key("[SLG汉化]游戏名称"), "游戏名称");
        assert_eq!(extract_search_key("游戏名称 PC版"), "游戏名称");
        assert_eq!(extract_search_key("游戏名称 汉化版"), "游戏名称");
    }

    #[test]
    fn test_find_common_parent_dir() {
        let paths = vec![
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "data".to_string(), "game.exe".to_string()],
        ];
        assert_eq!(find_common_parent_dir(&paths), 3);

        let paths = vec![
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
            vec!["C:".to_string(), "Games".to_string(), "Game2".to_string(), "game.exe".to_string()],
        ];
        assert_eq!(find_common_parent_dir(&paths), 2);
    }
}
