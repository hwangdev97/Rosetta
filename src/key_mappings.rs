use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Common iOS system keys and their meanings
pub static KEY_MEANINGS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Bundle keys
    map.insert("CFBundleDisplayName", "应用在主屏幕显示的名称");
    map.insert("CFBundleName", "应用包名称");
    map.insert("CFBundleShortVersionString", "应用版本号");
    map.insert("CFBundleVersion", "应用构建版本号");
    
    // Privacy usage descriptions
    map.insert("NSCameraUsageDescription", "相机权限使用说明");
    map.insert("NSPhotoLibraryUsageDescription", "照片库访问权限说明");
    map.insert("NSPhotoLibraryAddUsageDescription", "添加照片到相册权限说明");
    map.insert("NSMicrophoneUsageDescription", "麦克风权限使用说明");
    map.insert("NSLocationWhenInUseUsageDescription", "使用期间位置权限说明");
    map.insert("NSLocationAlwaysUsageDescription", "始终位置权限说明");
    map.insert("NSLocationAlwaysAndWhenInUseUsageDescription", "位置权限使用说明");
    map.insert("NSContactsUsageDescription", "通讯录访问权限说明");
    map.insert("NSCalendarsUsageDescription", "日历访问权限说明");
    map.insert("NSRemindersUsageDescription", "提醒事项访问权限说明");
    map.insert("NSMotionUsageDescription", "运动与健身权限说明");
    map.insert("NSHealthShareUsageDescription", "健康数据读取权限说明");
    map.insert("NSHealthUpdateUsageDescription", "健康数据写入权限说明");
    map.insert("NSBluetoothPeripheralUsageDescription", "蓝牙外设权限说明");
    map.insert("NSBluetoothAlwaysUsageDescription", "蓝牙权限使用说明");
    map.insert("NSAppleMusicUsageDescription", "Apple Music权限说明");
    map.insert("NSSpeechRecognitionUsageDescription", "语音识别权限说明");
    map.insert("NSVideoSubscriberAccountUsageDescription", "视频订阅账户权限说明");
    map.insert("NSNetworkVolumesUsageDescription", "网络卷访问权限说明");
    map.insert("NSDesktopFolderUsageDescription", "桌面文件夹访问权限说明");
    map.insert("NSDocumentsFolderUsageDescription", "文档文件夹访问权限说明");
    map.insert("NSDownloadsFolderUsageDescription", "下载文件夹访问权限说明");
    map.insert("NSRemovableVolumesUsageDescription", "可移动卷访问权限说明");
    
    // Widget configuration
    map.insert("CFBundleDisplayName_widget", "小组件显示名称");
    map.insert("widget_description", "小组件描述");
    map.insert("widget_configuration_intent_response", "小组件配置响应");
    
    // App Store metadata
    map.insert("APP_STORE_DESCRIPTION", "App Store应用描述");
    map.insert("APP_STORE_KEYWORDS", "App Store关键词");
    map.insert("APP_STORE_RELEASE_NOTES", "App Store更新说明");
    
    map
});

/// Infer the meaning of a key based on patterns and known mappings
pub fn infer_key_meaning(key: &str) -> Option<String> {
    // First check exact matches
    if let Some(&meaning) = KEY_MEANINGS.get(key) {
        return Some(meaning.to_string());
    }
    
    // Pattern-based inference
    let key_lower = key.to_lowercase();
    
    // Privacy descriptions
    if key_lower.contains("usagedescription") {
        return Some("隐私权限使用说明".to_string());
    }
    
    // Bundle-related keys
    if key_lower.starts_with("cfbundle") {
        return Some("应用包配置信息".to_string());
    }
    
    // Widget-related keys
    if key_lower.contains("widget") {
        return Some("小组件相关配置".to_string());
    }
    
    // Configuration keys
    if key_lower.contains("config") || key_lower.contains("setting") {
        return Some("配置设置".to_string());
    }
    
    // Error messages
    if key_lower.contains("error") || key_lower.contains("fail") {
        return Some("错误信息".to_string());
    }
    
    // Success messages
    if key_lower.contains("success") || key_lower.contains("complete") {
        return Some("成功信息".to_string());
    }
    
    // Button labels
    if key_lower.contains("button") || key_lower.contains("btn") {
        return Some("按钮标签".to_string());
    }
    
    // Alert/dialog related
    if key_lower.contains("alert") || key_lower.contains("dialog") || key_lower.contains("message") {
        return Some("提示信息".to_string());
    }
    
    // Navigation
    if key_lower.contains("nav") || key_lower.contains("tab") || key_lower.contains("menu") {
        return Some("导航菜单".to_string());
    }
    
    None
}

/// Categorize usage context based on key and meaning
pub fn categorize_usage(key: &str, meaning: Option<&str>) -> Option<String> {
    let key_lower = key.to_lowercase();
    
    if key_lower.contains("usagedescription") || key_lower.starts_with("ns") {
        return Some("system_permission".to_string());
    }
    
    if key_lower.starts_with("cfbundle") {
        return Some("app_metadata".to_string());
    }
    
    if key_lower.contains("widget") {
        return Some("widget_ui".to_string());
    }
    
    if key_lower.contains("button") || key_lower.contains("label") {
        return Some("ui_element".to_string());
    }
    
    if key_lower.contains("error") || key_lower.contains("alert") {
        return Some("user_message".to_string());
    }
    
    Some("general".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_meaning_inference() {
        assert_eq!(
            infer_key_meaning("CFBundleDisplayName"),
            Some("应用在主屏幕显示的名称".to_string())
        );
        
        assert_eq!(
            infer_key_meaning("NSCameraUsageDescription"),
            Some("相机权限使用说明".to_string())
        );
        
        assert_eq!(
            infer_key_meaning("some_custom_widget_title"),
            Some("小组件相关配置".to_string())
        );
        
        assert!(infer_key_meaning("unknown_key").is_some() == false);
    }

    #[test]
    fn test_usage_categorization() {
        assert_eq!(
            categorize_usage("NSCameraUsageDescription", None),
            Some("system_permission".to_string())
        );
        
        assert_eq!(
            categorize_usage("CFBundleDisplayName", None),
            Some("app_metadata".to_string())
        );
        
        assert_eq!(
            categorize_usage("widget_title", None),
            Some("widget_ui".to_string())
        );
    }
} 