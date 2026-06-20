/// 注册领域类型宏
///
/// 生成连续的 `app.register_type::<T>()` 调用。
/// 简化 Plugin::build() 中的类型注册样板代码。
///
/// # 用法
///
/// ```ignore
/// register_domain_types!(app, [
///     Experience,
///     ClassLevels,
///     TalentTree,
///     SubclassChoice,
///     ProgressionMarker,
/// ]);
/// ```
///
/// 展开为：
///
/// ```ignore
/// app.register_type::<Experience>();
/// app.register_type::<ClassLevels>();
/// app.register_type::<TalentTree>();
/// app.register_type::<SubclassChoice>();
/// app.register_type::<ProgressionMarker>();
/// ```
///
/// 支持带路径的复杂类型和尾部逗号。
#[macro_export]
macro_rules! register_domain_types {
    ($app:ident, [$($ty:ty),+$(,)?]) => {
        $(
            $app.register_type::<$ty>();
        )+
    };
}
