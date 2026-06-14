#!/usr/bin/env python3
"""Replace catalog entries 177-205 (Input + Audio + Camera sections).

Original mapping:
  175-176: Time section (timers.rs, virtual_time.rs) — NOT touched
  177-189: Input section (13 entries)
  190-196: Audio section (7 entries)
  197-205: Camera section (9 entries)
Total: 29 entries (177-205)
"""

import re

CATALOG = "bevy_examples_catalog.md"

# (seq, category, [intent_lines], [knowledge], [examples], filename)
DATA = [
    # ===== Input — 输入处理 (177-189) =====
    (
        177,
        "输入处理",
        [
            "**意图**：按键按下/释放处理——区分物理键位(KeyCode)与逻辑字符(Key)两种检测方式。⭐⭐",
            "- **ButtonInput 资源**:",
            "  - Res<ButtonInput<KeyCode>> 按物理键位检测，不受键盘布局影响；",
            "  - Res<ButtonInput<Key>> 按逻辑字符检测，适合符号键(如「?」「+」)；",
            "  - ⚠️ KeyCode::KeyA 永远是物理 A 键位置，不论 QWERTY/AZERTY 布局；",
            "- **三种检测时机**:",
            "  - .pressed(key) 持续按住时每帧为 true；",
            "  - .just_pressed(key) 按下瞬间仅一帧为 true；",
            "  - .just_released(key) 释放瞬间仅一帧为 true；",
            "  - 🔧 just_pressed 适合触发动作，pressed 适合持续移动；",
            "- **Key::Character 用法**:",
            "  - Key::Character(「?」.into()) 创建逻辑字符键；",
            "  - ⚠️ 需要 .clone() 因为 pressed 会消耗所有权；",
            "  - 🔧 适合快捷键绑定(如 ? 打开帮助菜单)；",
        ],
        [
            "ButtonInput<T> 资源检测按键/按钮状态；",
            "KeyCode 物理键位 vs Key 逻辑字符；",
            "just_pressed/pressed/just_released 三种时机。",
        ],
        [
            "hello_world(1)。",
        ],
        "keyboard_input.rs",
    ),
    (
        178,
        "输入处理",
        [
            "**意图**：通过事件系统打印所有键盘输入事件，展示 MessageReader 事件消费模式。⭐",
            "- **MessageReader 事件消费**:",
            "  - MessageReader<KeyboardInput> 逐帧读取事件队列；",
            "  - .read() 返回本次帧内所有未读事件的迭代器；",
            "  - ⚠️ MessageReader 替代旧版 EventReader，是 Bevy 0.18 的标准事件读取方式；",
            "- **KeyboardInput 事件结构**:",
            "  - .state: ButtonState::Pressed/Released 按键状态；",
            "  - .key_code: KeyCode 物理键位；",
            "  - .logical_key: Key 逻辑按键；",
            "  - 🔧 事件包含完整信息，比 ButtonInput 资源更灵活；",
            "- **事件 vs 资源选择**:",
            "  - ButtonInput<T> 适合「当前帧是否按住」的轮询式检测；",
            "  - MessageReader<KeyboardInput> 适合「按键时触发」的事件驱动模式；",
            "  - 🧩 两者可并存，按场景选择；",
        ],
        [
            "MessageReader<T> 事件读取器；",
            "KeyboardInput 事件结构；",
            "事件驱动 vs 轮询模式。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "keyboard_input_events.rs",
    ),
    (
        179,
        "输入处理",
        [
            "**意图**：检测 Ctrl+Shift+A 组合键，演示修饰键组合判断模式。⭐",
            "- **修饰键检测**:",
            "  - any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) 检查左右任一 Shift；",
            "  - any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) 检查左右任一 Ctrl；",
            "  - 🔧 同时检查左右修饰键是跨平台最佳实践；",
            "- **组合键判断**:",
            "  - shift && ctrl && just_pressed(KeyCode::KeyA) 三重条件组合；",
            "  - ⚠️ just_pressed 放最后：先判断修饰键持续状态，再判断目标键触发；",
            "  - 🧩 可扩展为 Ctrl+Shift+数字 等复杂快捷键组合；",
            "- **系统效率**:",
            "  - 修饰键用 pressed（持续状态），目标键用 just_pressed（单次触发）；",
            "  - 🔧 避免全部用 just_pressed——修饰键可能在目标键之前按下；",
        ],
        [
            "any_pressed() 修饰键组合检测；",
            "修饰键 + 目标键的组合模式。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "keyboard_modifiers.rs",
    ),
    (
        180,
        "输入处理",
        [
            "**意图**：从 KeyboardInput 事件中提取字符输入，仅过滤 Pressed 状态下的字符。⭐",
            "- **字符过滤逻辑**:",
            "  - event.state.is_pressed() 过滤掉 Released 事件；",
            "  - Key::Character(character) 模式匹配提取字符值；",
            "  - ⚠️ 键盘事件包含所有按键(含功能键)，需过滤才能得到纯字符；",
            "- **事件模式匹配**:",
            "  - if let Key::Character(character) = &event.logical_key 精确匹配；",
            "  - 🔧 字符事件与按键事件不同：字符事件包含组合键结果(如 Shift+A=「A」)；",
            "- **应用场景**:",
            "  - 🧩 聊天输入框、控制台命令行等需要逐字符输入的场景；",
            "  - ⚠️ 不要用于游戏操控——游戏操作应使用 ButtonInput<KeyCode>；",
        ],
        [
            "KeyboardInput 事件字符过滤；",
            "Key::Character 模式匹配；",
            "is_pressed() 状态过滤。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "char_input_events.rs",
    ),
    (
        181,
        "输入处理",
        [
            "**意图**：完整文本输入系统——支持 IME 输入法、逐字符编辑、回车换行和气泡浮动动画。⭐⭐⭐",
            "- **IME 输入法支持**:",
            "  - Window.ime_enabled 切换 IME 开关；",
            "  - Window.ime_position 设置输入法候选框位置；",
            "  - MessageReader<Ime> 读取 Ime::Preedit(预编辑)/Commit(确认)/Enabled/Disabled 事件；",
            "  - ⚠️ Ime::Preedit 的 cursor 为 None 时表示取消预编辑；",
            "- **文本编辑逻辑**:",
            "  - keyboard_input.text 获取已输入文本(非 logical_key)；",
            "  - Key::Enter 触发换行(气泡动画)，Key::Backspace 删除末字符；",
            "  - 🔧 text.push_str(inserted_text) 追加字符，text.pop() 删除末字符；",
            "- **Text2d + TextSpan 富文本**:",
            "  - Text2d::new(「」) 创建空文本实体作为编辑区；",
            "  - TextSpan 子实体实现多行/多样式文本；",
            "  - TextUiWriter 运行时修改指定 span 的文本内容；",
            "- **气泡浮动动画**:",
            "  - Bubble { timer } 组件控制生命周期(5秒后消失)；",
            "  - 每帧 transform.translation.y += delta_secs * 100.0 向上浮动；",
            "  - ⚠️ mem::take(&mut **text) 取出并清空原文本，用于生成气泡实体；",
            "- **可打印字符过滤**:",
            "  - is_printable_char() 过滤控制字符和私有使用区(PPU)；",
            "  - 🔧 来自 egui-winit 的成熟过滤逻辑；",
        ],
        [
            "Window.ime_enabled/ime_position IME 控制；",
            "Ime 事件(Preedit/Commit/Enabled/Disabled)；",
            "Text2d + TextSpan + TextUiWriter 文本编辑；",
            "keyboard_input.text 获取输入文本。",
        ],
        [
            "input/keyboard_input_events.rs(178)；",
            "2D/text2d.rs(11)。",
        ],
        "text_input.rs",
    ),
    (
        182,
        "输入处理",
        [
            "**意图**：鼠标按钮检测和累积鼠标运动/滚轮数据读取。⭐⭐",
            "- **ButtonInput<MouseButton> 按钮检测**:",
            "  - MouseButton::Left/Right/Middle 三种标准按钮；",
            "  - pressed/just_pressed/just_released 与键盘相同的三种时机；",
            "  - 🔧 模式与 ButtonInput<KeyCode> 完全一致；",
            "- **AccumulatedMouseMotion 累积运动**:",
            "  - Res<AccumulatedMouseMotion> 本帧鼠标总位移(delta)；",
            "  - delta != Vec2::ZERO 判断是否有移动；",
            "  - ⚠️ AccumulatedMouseMotion 已按帧率归一化，可直接用于位移计算；",
            "- **AccumulatedMouseScroll 累积滚轮**:",
            "  - Res<AccumulatedMouseScroll> 本帧滚轮总滚动量(delta)；",
            "  - delta.x 水平滚动，delta.y 垂直滚动；",
            "  - 🔧 delta.y > 0 向上滚动(放大)，< 0 向下滚动(缩小)；",
        ],
        [
            "ButtonInput<MouseButton> 鼠标按钮检测；",
            "AccumulatedMouseMotion 累积鼠标位移；",
            "AccumulatedMouseScroll 累积滚轮。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "mouse_input.rs",
    ),
    (
        183,
        "输入处理",
        [
            "**意图**：通过事件系统打印所有鼠标事件——按钮/移动/光标/滚轮/触控板手势。⭐⭐",
            "- **五类鼠标事件**:",
            "  - MouseButtonInput: 鼠标按钮按下/释放；",
            "  - MouseMotion: 逐帧原始鼠标移动(非累积)；",
            "  - CursorMoved: 光标在窗口内移动的绝对位置；",
            "  - MouseWheel: 滚轮事件(含 Line/Pixel 两种单位)；",
            "  - 🧩 MouseMotion 是原始增量，AccumulatedMouseMotion 是帧末累积值；",
            "- **macOS 触控板手势**:",
            "  - PinchGesture: 双指捏合(缩放)；",
            "  - RotationGesture: 双指旋转；",
            "  - DoubleTapGesture: 双击；",
            "  - ⚠️ 这三个事件仅在 macOS 上触发；",
            "- **事件 vs 资源选择**:",
            "  - MessageReader<MouseButtonInput> 适合逐事件处理(如点击判定)；",
            "  - Res<AccumulatedMouseMotion> 适合连续运动(如视角旋转)；",
            "  - 🔧 按需选择，不必同时使用两种方式；",
        ],
        [
            "MouseButtonInput/MouseMotion/CursorMoved/MouseWheel 事件；",
            "PinchGesture/RotationGesture/DoubleTapGesture 手势；",
            "事件驱动 vs 资源轮询。",
        ],
        [
            "input/mouse_input.rs(182)。",
        ],
        "mouse_input_events.rs",
    ),
    (
        184,
        "输入处理",
        [
            "**意图**：抓取并隐藏鼠标光标——用于 FPS/TPS 游戏的鼠标锁定。⭐⭐",
            "- **CursorOptions 组件**:",
            "  - Single<&mut CursorOptions> 直接操作窗口的光标选项；",
            "  - .visible: bool 控制光标可见性；",
            "  - .grab_mode: CursorGrabMode 枚举控制抓取模式；",
            "- **CursorGrabMode 枚举**:",
            "  - CursorGrabMode::Locked: 锁定光标到窗口中心(推荐 FPS)；",
            "  - CursorGrabMode::None: 释放光标(默认)；",
            "  - ⚠️ Locked 模式下光标隐藏且不移动，鼠标移动转为相对位移；",
            "- **操作模式**:",
            "  - 鼠标左键点击 → 隐藏并锁定光标；",
            "  - Escape 键 → 显示并释放光标；",
            "  - 🔧 这是 FPS 游戏的标准鼠标锁定/释放模式；",
        ],
        [
            "CursorOptions 组件(visible/grab_mode)；",
            "CursorGrabMode::Locked/None 枚举。",
        ],
        [
            "input/mouse_input.rs(182)。",
        ],
        "mouse_grab.rs",
    ),
    (
        185,
        "输入处理",
        [
            "**意图**：手柄输入检测——按钮触发、模拟轴值读取、多手柄支持。⭐⭐",
            "- **Gamepad 组件查询**:",
            "  - Query<(Entity, &Gamepad)> 获取所有已连接手柄实体；",
            "  - Entity 作为手柄唯一标识(多手柄时区分)；",
            "  - 🔧 手柄连接/断开时实体自动创建/销毁；",
            "- **按钮检测**:",
            "  - GamepadButton::South/East/North/West 对应四种脸键(ABXY)；",
            "  - gamepad.just_pressed(button) / just_released(button) 触发检测；",
            "  - ⚠️ 不同手柄的 South/East 映射可能不同(Xbox=南A，Switch=南B)；",
            "- **模拟轴读取**:",
            "  - gamepad.get(GamepadButton::RightTrigger2) 返回 Option<f32>；",
            "  - gamepad.get(GamepadAxis::LeftStickX) 返回 Option<f32>；",
            "  - 值范围 -1.0..=1.0(轴) 或 0.0..=1.0(按钮)；",
            "  - ⚠️ abs() > 0.01 过滤死区(摇杆中心漂移)；",
        ],
        [
            "Gamepad 组件与 GamepadButton/GamepadAxis；",
            "gamepad.get() 读取模拟值；",
            "死区过滤(abs > 0.01)。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "gamepad_input.rs",
    ),
    (
        186,
        "输入处理",
        [
            "**意图**：手柄事件系统——连接/断开、轴值变化、按钮压力值变化的完整事件流。⭐⭐",
            "- **四种手柄事件**:",
            "  - GamepadConnectionEvent: 手柄连接/断开事件；",
            "  - GamepadAxisChangedEvent: 轴值变化(摇杆/扳机连续值)；",
            "  - GamepadButtonChangedEvent: 按钮压力值变化(模拟量，0.0-1.0)；",
            "  - GamepadButtonStateChangedEvent: 按钮布尔状态变化(超过阈值触发)；",
            "  - 🧩 Changed 事件是连续值，State 事件是离散布尔值；",
            "- **GamepadEvent 统一事件**:",
            "  - GamepadEvent::Connection/Button/Axis 枚举包装所有事件；",
            "  - 适合需要按帧排序处理所有手柄事件的场景；",
            "  - 🔧 标准用法选单独事件，需排序时用 GamepadEvent；",
            "- **与 ButtonInput 对比**:",
            "  - ButtonInput<GamepadButton> 适合轮询检测；",
            "  - MessageReader 事件适合精确追踪变化时机；",
            "  - ⚠️ GamepadButtonChangedEvent 包含连续压力值(如轻按/重按)；",
        ],
        [
            "GamepadConnectionEvent/AxisChanged/ButtonChanged/ButtonStateChanged 事件；",
            "GamepadEvent 统一事件枚举。",
        ],
        [
            "input/gamepad_input.rs(185)。",
        ],
        "gamepad_input_events.rs",
    ),
    (
        187,
        "输入处理",
        [
            "**意图**：手柄力反馈——通过 GamepadRumbleRequest 触发不同强度的震动效果。⭐⭐",
            "- **GamepadRumbleRequest 两种操作**:",
            "  - GamepadRumbleRequest::Add { gamepad, intensity, duration } 添加震动；",
            "  - GamepadRumbleRequest::Stop { gamepad } 停止当前震动；",
            "  - MessageWriter<GamepadRumbleRequest> 发送震动请求；",
            "- **GamepadRumbleIntensity 强度配置**:",
            "  - GamepadRumbleIntensity::strong_motor(0.1) 仅低频马达(左手侧)；",
            "  - GamepadRumbleIntensity::weak_motor(0.25) 仅高频马达(右手侧)；",
            "  - GamepadRumbleIntensity::MAX 双马达最大震动；",
            "  - 🔧 GamepadRumbleIntensity { strong_motor, weak_motor } 自定义双马达强度；",
            "- **重复按键增强**:",
            "  - 多次按同一键会叠加震动强度(Add 是累加而非替换)；",
            "  - ⚠️ 需要手动调用 Stop 才能停止震动；",
            "  - 🔧 Duration::from_secs(5) 控制震动时长；",
        ],
        [
            "GamepadRumbleRequest::Add/Stop；",
            "GamepadRumbleIntensity 强度/双马达。",
        ],
        [
            "input/gamepad_input.rs(185)。",
        ],
        "gamepad_rumble.rs",
    ),
    (
        188,
        "输入处理",
        [
            "**意图**：触摸输入检测——通过 Res<Touches> 资源查询按下/释放/取消状态。⭐",
            "- **Touches 资源 API**:",
            "  - Res<Touches> 访问所有触摸点状态；",
            "  - iter_just_pressed(): 本帧新按下的触摸点；",
            "  - iter_just_released(): 本帧释放的触摸点；",
            "  - iter_just_canceled(): 本帧取消的触摸点(如系统手势打断)；",
            "  - iter(): 所有当前活跃的触摸点；",
            "- **触摸点信息**:",
            "  - touch.id(): 触摸点唯一标识(多指触控时区分)；",
            "  - touch.position(): 触摸点屏幕坐标(Vec2)；",
            "  - touches.just_pressed(id): 查询指定触摸点是否刚按下；",
            "- **与事件方式对比**:",
            "  - Touches 资源是轮询式(当前状态快照)；",
            "  - MessageReader<TouchInput> 是事件式(逐事件处理)；",
            "  - 🔧 简单场景用 touches，需要精确追踪用事件；",
        ],
        [
            "Touches 资源(轮询式触摸状态)；",
            "iter_just_pressed/released/canceled 迭代。",
        ],
        [
            "input/keyboard_input.rs(177)。",
        ],
        "touch_input.rs",
    ),
    (
        189,
        "输入处理",
        [
            "**意图**：通过 MessageReader<TouchInput> 打印所有触摸事件详情。⭐",
            "- **TouchInput 事件**:",
            "  - MessageReader<TouchInput> 逐帧读取触摸事件流；",
            "  - TouchInput 包含 id、position、phase(Pressed/Released/Moved/Cancelled)；",
            "  - 🔧 比 Touches 资源更底层，包含所有原始事件(含 Moved)；",
            "- **与 Touches 资源对比**:",
            "  - Touches 不包含 Moved 事件(仅状态变化)；",
            "  - TouchInput 包含每帧的 Moved 事件(可追踪拖拽轨迹)；",
            "  - ⚠️ 大量触摸移动时 TouchInput 事件量大，注意性能；",
            "- **触摸 vs 鼠标**:",
            "  - 触摸支持多点(多指)，鼠标仅单点；",
            "  - 🧩 触摸手势(缩放/旋转)需从多点触摸数据自行计算；",
        ],
        [
            "TouchInput 事件(含 phase 状态)；",
            "Touches vs TouchInput 选择。",
        ],
        [
            "input/touch_input.rs(188)。",
        ],
        "touch_input_events.rs",
    ),
    # ===== Audio — 音频 (190-196) =====
    (
        190,
        "音频",
        [
            "**意图**：最简音频播放——一行代码加载并播放 OGG 音频文件。⭐",
            "- **AudioPlayer 组件**:",
            "  - AudioPlayer::new(asset_server.load(「path.ogg」)) 创建音频播放器；",
            "  - AudioPlayer 是组件，spawn 到实体即自动播放；",
            "  - ⚠️ 音频文件位于 assets/ 目录下；",
            "- **音频加载**:",
            "  - AssetServer::load() 加载音频资源返回 Handle<AudioSource>；",
            "  - 支持 OGG 格式(默认)，其他格式需启用对应 feature；",
            "  - 🔧 AudioPlayer 会在实体销毁时自动停止播放；",
            "- **最小模板**:",
            "  - DefaultPlugins 包含 AudioPlugin(无需额外添加)；",
            "  - Startup 系统中 spawn AudioPlayer 即可播放背景音乐；",
        ],
        [
            "AudioPlayer 组件(自动播放)；",
            "AssetServer 加载音频资源。",
        ],
        [
            "asset/asset_loading.rs(159)。",
        ],
        "audio.rs",
    ),
    (
        191,
        "音频",
        [
            "**意图**：音频播放控制——暂停/恢复、音量调节、播放速度、进度查询。⭐⭐⭐",
            "- **AudioSink 组件**:",
            "  - Single<&AudioSink, With<T>> 获取音频控制器(独占访问)；",
            "  - Query<&mut AudioSink> 获取可变访问(修改音量/速度)；",
            "  - 🔧 AudioSink 与 AudioPlayer 分离：Player 负责加载，Sink 负责控制；",
            "- **播放控制 API**:",
            "  - sink.toggle_playback(): 播放/暂停切换(Space 键)；",
            "  - sink.toggle_mute(): 静音/取消静音切换(M 键)；",
            "  - sink.position(): 当前播放进度(Duration)；",
            "  - sink.is_paused(): 查询暂停状态；",
            "- **音量调节**:",
            "  - sink.volume() 获取当前音量；",
            "  - sink.set_volume(Volume) 设置新音量；",
            "  - Volume::increase_by_percentage(10.0) 增加 10% 音量；",
            "  - ⚠️ decrease 用负百分比: increase_by_percentage(-10.0)；",
            "- **播放速度**:",
            "  - sink.set_speed(f32) 设置播放速度(1.0=正常)；",
            "  - 🔧 用 sin 函数动态改变速度可产生趣味音效；",
        ],
        [
            "AudioSink 组件(playback/volume/speed/mute)；",
            "Volume::increase_by_percentage 音量调节。",
        ],
        [
            "audio/audio.rs(190)。",
        ],
        "audio_control.rs",
    ),
    (
        192,
        "音频",
        [
            "**意图**：播放单频音调(Pitch)——程序化生成音频，支持半音阶频率调整。⭐⭐",
            "- **Pitch 资产**:",
            "  - Pitch::new(frequency, duration) 创建单频音调；",
            "  - Assets<Pitch> 管理 Pitch 资产(手动创建，非文件加载)；",
            "  - ⚠️ Pitch 不通过 AssetServer 加载，直接插入 Assets<Pitch>；",
            "- **Message 事件触发**:",
            "  - #[derive(Message)] PlayPitch 自定义消息类型；",
            "  - app.add_message::<PlayPitch>() 注册消息；",
            "  - MessageWriter/MessageReader 发送/接收触发播放；",
            "- **半音阶频率调整**:",
            "  - powf(2.0, 1.0/12.0) 半音比率(12 个半音=1 个八度)；",
            "  - ArrowUp 加半音(频率乘比率)，ArrowDown 减半音(频率除比率)；",
            "  - 🔧 220Hz = A3，440Hz = A4，经典音高标准；",
            "- **PlaybackSettings::DESPAWN**:",
            "  - 播放完毕自动销毁实体，避免内存泄漏；",
            "  - ⚠️ 无此设置的音频实体会一直存在(即使已播完)；",
        ],
        [
            "Pitch::new(frequency, duration)；",
            "Assets<Pitch> 手动资产管理；",
            "PlaybackSettings::DESPAWN 自动销毁。",
        ],
        [
            "audio/audio.rs(190)。",
        ],
        "pitch.rs",
    ),
    (
        193,
        "音频",
        [
            "**意图**：自定义 Decodable 实现——创建程序化正弦波音频解码器。⭐⭐⭐",
            "- **Decodable trait 实现**:",
            "  - SineAudio 作为资产类型(#[derive(Asset, TypePath)])；",
            "  - SineDecoder 作为解码器(实现 Iterator<Item=f32> + Source trait)；",
            "  - Decodable::decoder() 返回解码器实例；",
            "- **Source trait 必须实现**:",
            "  - channels(): 声道数(1=单声道)；",
            "  - sample_rate(): 采样率(44100 标准)；",
            "  - current_frame_len(): 当前帧长度(None=无帧边界)；",
            "  - total_duration(): 总时长(None=无限)；",
            "- **解码器逻辑**:",
            "  - current_progress 在 0.0-1.0 间循环(一个周期)；",
            "  - progress_per_frame = frequency / sample_rate 控制音高；",
            "  - sin(period * progress) 生成正弦波采样；",
            "- **注册与使用**:",
            "  - app.add_audio_source::<SineAudio>() 注册自定义音频类型；",
            "  - AudioPlugin { global_volume: Volume::Linear(0.2) } 全局音量控制；",
            "  - 🔧 适合程序化音效、合成器、测试音频等场景；",
        ],
        [
            "Decodable trait(Asset + Iterator + Source)；",
            "add_audio_source 注册自定义音频类型；",
            "Source trait(channels/sample_rate)。",
        ],
        [
            "audio/audio.rs(190)。",
        ],
        "decodable.rs",
    ),
    (
        194,
        "音频",
        [
            "**意图**：根据游戏状态切换背景音乐，实现渐入渐出过渡效果。⭐⭐⭐",
            "- **状态驱动切换**:",
            "  - GameState 资源(Peaceful/Battle)驱动音轨切换；",
            "  - game_state.is_changed() 检测状态变化(仅在变化时切换)；",
            "  - 🔧 Resource + is_changed() 是状态响应的标准模式；",
            "- **SoundtrackPlayer 资源**:",
            "  - track_list: Vec<Handle<AudioSource>> 存储音轨列表；",
            "  - match game_state 选择对应音轨；",
            "  - ⚠️ 需要 unwrap() 确保索引有效；",
            "- **渐入渐出效果**:",
            "  - FadeIn/FadeOut 组件标记需要过渡的实体；",
            "  - Volume::SILENT.fade_towards(Volume::Linear(1.0), t) 渐入；",
            "  - Volume::Linear(1.0).fade_towards(Volume::SILENT, t) 渐出；",
            "  - FADE_TIME = 2.0 秒过渡时长；",
            "- **播放设置**:",
            "  - PlaybackMode::Loop 循环播放；",
            "  - Volume::SILENT 初始静音(渐入从零开始)；",
            "  - 🔧 FadeOut 完成后 commands.entity().despawn() 销毁旧音频实体；",
            "- **GameStateTimer 模拟**:",
            "  - Timer::from_seconds(10.0, TimerMode::Repeating) 每 10 秒切换状态；",
            "  - 🔧 实际项目中状态切换来自游戏逻辑(如进入战斗)；",
        ],
        [
            "Volume::fade_towards 渐变过渡；",
            "PlaybackMode::Loop 循环播放；",
            "GameState Resource + is_changed() 状态驱动。",
        ],
        [
            "audio/audio.rs(190)；",
            "state/states.rs(155)。",
        ],
        "soundtrack.rs",
    ),
    (
        195,
        "音频",
        [
            "**意图**：2D 空间音频——声音源位置影响音量衰减，模拟双耳听觉。⭐⭐",
            "- **SpatialScale 配置**:",
            "  - SpatialScale::new_2d(1.0/100.0) 设置 2D 空间缩放；",
            "  - 100 像素 = 1 单位距离(音频衰减基准)；",
            "  - ⚠️ 默认 2D 相机中 1 像素 = 1 单位，需缩放避免衰减过快；",
            "- **SpatialListener 组件**:",
            "  - SpatialListener::new(gap) 创建双耳监听器(间距 gap)；",
            "  - .left_ear_offset / .right_ear_offset 左右耳偏移量；",
            "  - 🔧 子实体可视化左右耳位置(红/绿方块)；",
            "- **Emitter 组件**:",
            "  - Emitter 标记声音发射源实体；",
            "  - AudioPlayer + PlaybackSettings::LOOP.with_spatial(true) 空间音频播放；",
            "  - ⚠️ 必须设置 spatial: true 才启用空间音频衰减；",
            "- **距离衰减**:",
            "  - Emitter 距 Listener 越远，音量越低；",
            "  - 运动 Emitter 产生动态空间音效；",
            "  - 🔧 Stopwatch 控制 Emitter 运动暂停/恢复；",
        ],
        [
            "SpatialScale::new_2d 空间缩放；",
            "SpatialListener 双耳监听器；",
            "PlaybackSettings::with_spatial(true)。",
        ],
        [
            "audio/audio.rs(190)。",
        ],
        "spatial_audio_2d.rs",
    ),
    (
        196,
        "音频",
        [
            "**意图**：3D 空间音频——声音源在 3D 空间中的位置影响音量和方向感。⭐⭐",
            "- **3D SpatialListener**:",
            "  - SpatialListener::new(4.0) 3D 双耳监听器(间距 4 单位)；",
            "  - left_ear_offset/right_ear_offset 可视化耳位置；",
            "  - 🔧 与 2D 版本相同 API，但坐标系为 3D；",
            "- **3D Emitter 运动**:",
            "  - sin/cos 产生圆形运动轨迹(半径 3.0)；",
            "  - Stopwatch 控制运动暂停/恢复；",
            "  - 🧩 空间音频 + 运动源 = 环绕音效；",
            "- **SpatialAudioSink 静音**:",
            "  - Query<&mut SpatialAudioSink> 空间音频控制器；",
            "  - sink.toggle_mute() 空间音频专用静音；",
            "  - ⚠️ SpatialAudioSink 与 AudioSink 不同——空间音频用 SpatialAudioSink；",
            "- **3D 场景要素**:",
            "  - Camera3d + DirectionalLight 标准 3D 场景；",
            "  - Sphere/Cuboid 网格可视化 Emitter 和 Listener；",
        ],
        [
            "SpatialListener 3D 双耳监听；",
            "SpatialAudioSink 空间音频控制。",
        ],
        [
            "audio/spatial_audio_2d.rs(195)。",
        ],
        "spatial_audio_3d.rs",
    ),
    # ===== Camera — 相机 (197-205) =====
    (
        197,
        "相机",
        [
            "**意图**：2D 平移/缩放相机控制器——使用 PanCameraPlugin 一行代码实现完整相机操控。⭐⭐",
            "- **PanCameraPlugin 插件**:",
            "  - .add_plugins(PanCameraPlugin) 注册相机控制插件；",
            "  - 自动处理 WASD/方向键平移、鼠标滚轮缩放；",
            "  - 🔧 开箱即用，无需手动编写相机控制逻辑；",
            "- **PanCamera 组件**:",
            "  - PanCamera::default() 附加到 Camera2d 实体；",
            "  - 控制参数可自定义(速度/缩放范围等)；",
            "  - ⚠️ 仅适用于 2D 正交相机；",
            "- **使用场景**:",
            "  - 🧩 策略游戏/地图编辑器等需要自由平移缩放的场景；",
            "  - 🔧 比手动实现相机控制简洁得多；",
        ],
        [
            "PanCameraPlugin 2D 相机控制插件；",
            "PanCamera 组件配置。",
        ],
        [
            "2D/move_sprite.rs(3)。",
        ],
        "pan_camera_controller.rs",
    ),
    (
        198,
        "相机",
        [
            "**意图**：2D 俯视相机平滑跟随玩家——使用 smooth_nudge 实现自然延迟跟踪。⭐⭐⭐",
            "- **smooth_nudge 跟随**:",
            "  - camera.translation.smooth_nudge(&direction, decay_rate, dt) 指数衰减插值；",
            "  - CAMERA_DECAY_RATE = 2.0 控制跟随速度(越大越紧)；",
            "  - 🔧 smooth_nudge 比 lerp 更自然，自动处理接近时减速；",
            "- **Bloom 泛光效果**:",
            "  - Camera2d + Bloom::NATURAL 启用自然泛光；",
            "  - 玩家实体 RGB 超过 1.0 实现发光效果；",
            "  - ⚠️ HDR 颜色(>1.0)才能产生辉光；",
            "- **Without 过滤模式**:",
            "  - Single<&mut Transform, (With<Camera2d>, Without<Player>)> 相机排除玩家；",
            "  - Single<&Transform, (With<Player>, Without<Camera2d>)> 玩家排除相机；",
            "  - ⚠️ Without 避免两个 Single 匹配同一实体；",
            "- **normalize_or_zero 归一化**:",
            "  - direction.normalize_or_zero() 防止对角移动速度过快；",
            "  - 🔧 零向量时返回零(避免 NaN)；",
        ],
        [
            "smooth_nudge 指数衰减插值；",
            "Bloom::NATURAL 泛光效果；",
            "Without<T> 过滤避免冲突。",
        ],
        [
            "camera/pan_camera_controller.rs(197)。",
        ],
        "2d_top_down_camera.rs",
    ),
    (
        199,
        "相机",
        [
            "**意图**：2D 屏幕震动——基于创伤值(trauma)和 Perlin 噪声实现平滑相机抖动。⭐⭐⭐",
            "- **创伤(trauma)系统**:",
            "  - trauma 范围 0.0-1.0，越高震动越强；",
            "  - shake = trauma^TRAUMA_EXPONENT 非线性映射(指数 2.0 使低创伤时震动微弱)；",
            "  - trauma_decay_per_second = 0.5 自然衰减(1.0→0.0 需 2 秒)；",
            "  - ⚠️ trauma > 1.0 时 clamp 到 1.0(最大震动)；",
            "- **Perlin 噪声平滑**:",
            "  - 1D Perlin noise 生成平滑连续的位移值(范围 -1..1)；",
            "  - 三个独立噪声源(旋转/X/Y)用不同偏移避免相关；",
            "  - 🔧 噪声比随机数更「自然」——不会突兀跳变；",
            "- **帧末应用模式**:",
            "  - PreUpdate: reset_transform 恢复原始 Transform；",
            "  - PostUpdate: shake_camera.before(TransformSystems::Propagate) 应用震动；",
            "  - ⚠️ 震动仅影响渲染，游戏逻辑使用未震动的原始 Transform；",
            "- **CameraShakeConfig 配置**:",
            "  - #[require(CameraShakeState)] 自动附加状态组件；",
            "  - max_angle: 10° 最大旋转角度；",
            "  - max_translation: 20px 最大平移距离；",
            "  - 🔧 所有参数集中在 Config 中，方便调参；",
        ],
        [
            "trauma 曲线(trauma^exponent)；",
            "Perlin 噪声平滑位移；",
            "PreUpdate 恢复 + PostUpdate 震动的帧分离模式。",
        ],
        [
            "camera/2d_top_down_camera.rs(198)。",
        ],
        "2d_screen_shake.rs",
    ),
    (
        200,
        "相机",
        [
            "**意图**：轨道相机——通过 Pitch/Yaw/Roll 欧拉角控制环绕静态场景的相机。⭐⭐⭐",
            "- **Pitch/Yaw/Roll 控制**:",
            "  - AccumulatedMouseMotion.delta.y → pitch(俯仰)；",
            "  - AccumulatedMouseMotion.delta.x → yaw(偏航)；",
            "  - 鼠标左/右键 → roll(翻滚)；",
            "  - ⚠️ 鼠标运动不乘 delta_time(已累积)；按钮输入需要乘；",
            "- **欧拉角分解与重组**:",
            "  - rotation.to_euler(EulerRot::YXZ) 分解为(yaw, pitch, roll)；",
            "  - Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll) 重组四元数；",
            "  - ⚠️ YXZ 顺序避免万向锁(先偏航→再俯仰→最后翻滚)；",
            "- **Pitch 钳位**:",
            "  - pitch.clamp(-FRAC_PI_2+0.01, FRAC_PI_2-0.01) 限制 ±90°；",
            "  - ⚠️ 超过 90° 会导致方向翻转(万向锁)；",
            "- **轨道位置更新**:",
            "  - camera.translation = target - forward() * orbit_distance；",
            "  - 🔧 始终面向目标 + 保持固定距离 = 轨道相机；",
        ],
        [
            "to_euler(EulerRot::YXZ) 欧拉角分解；",
            "Pitch 钳位避免万向锁；",
            "AccumulatedMouseMotion 已累积(不乘 dt)。",
        ],
        [
            "3D/3d_scene.rs(32)；",
            "input/mouse_input_events.rs(183)。",
        ],
        "camera_orbit.rs",
    ),
    (
        201,
        "相机",
        [
            "**意图**：自由相机控制器——6 轴移动 + 鼠标视角，适合 3D 场景探索和调试。⭐⭐⭐",
            "- **FreeCamera 组件**:",
            "  - sensitivity: 鼠标灵敏度；",
            "  - friction: 摩擦力(停止速度)；",
            "  - walk_speed/run_speed: 行走/奔跑速度；",
            "  - scroll_factor: 滚轮调整速度系数；",
            "  - 🔧 所有参数运行时可调(通过 FreeCameraState.enabled 开关)；",
            "- **FreeCameraPlugin 插件**:",
            "  - .add_plugins(FreeCameraPlugin) 注册控制逻辑；",
            "  - FreeCameraState 存储运行时状态(velocity/enabled)；",
            "  - ⚠️ 插件系统会持续修改 FreeCamera 组件(需注意竞态)；",
            "- **控制方案**:",
            "  - WASD: 水平移动；QE: 垂直移动；",
            "  - 鼠标: 视角旋转；左键按住: 捕获鼠标；",
            "  - M: 切换鼠标捕获；左 Shift: 奔跑；",
            "  - 🔧 滚轮: 调整移动速度；",
            "- **Plugin 架构**:",
            "  - CameraPlugin/CameraSettingsPlugin/ScenePlugin 分离关注点；",
            "  - 🔧 每个 Plugin 负责一个独立功能(相机/设置UI/场景)；",
        ],
        [
            "FreeCamera + FreeCameraState 组件；",
            "FreeCameraPlugin 插件系统；",
            "Plugin 架构分离关注点。",
        ],
        [
            "3D/3d_scene.rs(32)。",
        ],
        "free_camera_controller.rs",
    ),
    (
        202,
        "相机",
        [
            "**意图**：第一人称相机——ViewModel/WorldModel 双相机分层渲染，不同 FOV 独立控制。⭐⭐⭐",
            "- **双相机架构**:",
            "  - WorldModelCamera: 渲染世界(默认层 0)，FOV 可调(20°-160°)；",
            "  - View Model Camera: 渲染手臂(层 1)，FOV 固定 70°；",
            "  - Camera { order: 1 } 确保 ViewModel 后渲染(覆盖世界)；",
            "  - ⚠️ ViewModel 设计为固定 FOV——玩家手臂模型按特定 FOV 制作；",
            "- **RenderLayers 渲染层**:",
            "  - DEFAULT_RENDER_LAYER = 0: 世界物体；",
            "  - VIEW_MODEL_RENDER_LAYER = 1: 手臂+ViewModel 相机；",
            "  - RenderLayers::from_layers(&[0, 1]) 灯光同时照亮两层；",
            "  - 🔧 RenderLayers 是 Bevy 的图层蒙版系统；",
            "- **鼠标视角控制**:",
            "  - CameraSensitivity(Vec2) 组件控制水平/垂直灵敏度；",
            "  - AccumulatedMouseMotion 已累积(不乘 dt)；",
            "  - pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT) 防万向锁；",
            "- **FOV 动态调整**:",
            "  - Single<&mut Projection, With<WorldModelCamera>> 访问投影；",
            "  - PerspectiveProjection.fov 修改透视 FOV；",
            "  - 🔧 ArrowUp/Down 调整世界 FOV(不影响 ViewModel)；",
        ],
        [
            "RenderLayers 双层渲染；",
            "ViewModel/WorldModel 分层架构；",
            "Camera::order 渲染优先级。",
        ],
        [
            "camera/camera_orbit.rs(200)。",
        ],
        "first_person_view_model.rs",
    ),
    (
        203,
        "相机",
        [
            "**意图**：自定义相机投影——实现 CameraProjection trait 创建斜透视投影。⭐⭐⭐",
            "- **CameraProjection trait**:",
            "  - get_clip_from_view(): 返回裁剪空间矩阵(Mat4)；",
            "  - get_clip_from_view_for_sub(): 子视口裁剪矩阵；",
            "  - update(width, height): 窗口尺寸变化时更新；",
            "  - far(): 远裁剪面距离；",
            "  - 🔧 实现此 trait 即可注册自定义投影类型；",
            "- **ObliquePerspectiveProjection**:",
            "  - 在标准透视矩阵基础上修改第 3 列(col_mut(2))；",
            "  - horizontal/vertical_obliqueness 控制灭点偏移；",
            "  - ⚠️ 修改裁剪矩阵的第 3 列实现斜投影(非对称灭点)；",
            "- **Projection::custom() 注册**:",
            "  - Projection::custom(ObliquePerspectiveProjection { ... }) 附加到相机；",
            "  - 🔧 自定义投影与内置投影(Orthographic/Perspective)使用相同接口；",
            "- **子视口支持**:",
            "  - get_clip_from_view_for_sub(SubCameraView) 支持分屏/画中画；",
            "  - 🔧 委托给内部 PerspectiveProjection 的同名方法再修改；",
        ],
        [
            "CameraProjection trait(自定义投影接口)；",
            "Projection::custom() 注册；",
            "裁剪矩阵修改实现斜投影。",
        ],
        [
            "camera/pan_camera_controller.rs(197)。",
        ],
        "custom_projection.rs",
    ),
    (
        204,
        "相机",
        [
            "**意图**：正交/透视投影缩放——鼠标滚轮控制缩放，支持运行时切换投影类型。⭐⭐⭐",
            "- **正交投影缩放**:",
            "  - OrthographicProjection.scale 控制缩放比(值越小越放大)；",
            "  - Multiplicative zoom: scale *= 1 + delta_zoom(对数缩放更自然)；",
            "  - ScalingMode::FixedVertical 保持视口高度固定；",
            "  - ⚠️ 正交缩放是乘法(不是加法)，滚轮速度需配合；",
            "- **透视投影缩放**:",
            "  - PerspectiveProjection.fov 控制视角(范围 PI/5 到 PI-0.2)；",
            "  - perspective.fov += delta_zoom(加法缩放)；",
            "  - ⚠️ FOV 范围有限(约 36°-162°)，不能无限缩放；",
            "- **投影类型切换**:",
            "  - **camera = match **camera 运行时切换投影枚举；",
            "  - Space 键在 Orthographic/Perspective 间切换；",
            "  - 🔧 切换时需重建 Projection(不是修改内部值)；",
            "- **AccumulatedMouseScroll**:",
            "  - Res<AccumulatedMouseScroll> 滚轮输入(已累积)；",
            "  - delta.y 向上为正(放大)，向下为负(缩小)；",
            "  - ⚠️ 不需要乘 delta_time——滚轮值已按帧率归一化；",
        ],
        [
            "OrthographicProjection.scale 正交缩放；",
            "PerspectiveProjection.fov 透视缩放；",
            "Projection 枚举运行时切换。",
        ],
        [
            "camera/custom_projection.rs(203)。",
        ],
        "projection_zoom.rs",
    ),
    (
        205,
        "相机",
        [
            "**意图**：在 UI 上层渲染 2D 物体——使用第二个高 order 相机 + RenderLayers 分离。⭐⭐",
            "- **双相机 UI 叠加**:",
            "  - 默认相机(Camera2d + IsDefaultUiCamera)渲染 UI；",
            "  - 第二相机(Camera2d, order:1)渲染 2D 物体在 UI 之上；",
            "  - ClearColorConfig::None 第二相机不清除背景(透出第一相机内容)；",
            "  - ⚠️ Camera::order 值越大越后渲染(覆盖前者)；",
            "- **RenderLayers 分层**:",
            "  - 第二相机 RenderLayers::layer(1) 只渲染层 1 实体；",
            "  - 2D 物体(Sprite)也附加 RenderLayers::layer(1)；",
            "  - 🧩 RenderLayers 是相机-实体间的可见性蒙版；",
            "- **IsDefaultUiCamera**:",
            "  - 标记默认 UI 相机(替代旧版 UiCamera 组件)；",
            "  - 也可用 UiTargetCamera 组件指定每个 UI 节点的目标相机；",
            "  - 🔧 未标记 IsDefaultUiCamera 的 UI 节点不显示；",
            "- **Sprite 渲染**:",
            "  - Sprite + Transform 控制 2D 物体位置/旋转；",
            "  - 🔧 rotate_sprite 系统同时旋转 Sprite 演示 2D 能力；",
        ],
        [
            "Camera::order 渲染优先级；",
            "IsDefaultUiCamera 默认 UI 相机标记；",
            "ClearColorConfig::None 透明背景。",
        ],
        [
            "2D/sprite.rs(2)；",
            "ui/button.rs(230)。",
        ],
        "2d_on_ui.rs",
    ),
]


def main():
    with open(CATALOG, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.split("\n")

    # Build seq -> entry lookup
    DATA_MAP = {entry[0]: entry for entry in DATA}

    # Find the start of entry 177 and end of entry 205
    start_idx = None
    end_idx = None

    for i, line in enumerate(lines):
        if re.match(r"\| 177 \|", line):
            start_idx = i
        if re.match(r"\| 205 \|", line):
            end_idx = i
            break

    if start_idx is None or end_idx is None:
        print(f"ERROR: Could not find entries 177-205. start={start_idx} end={end_idx}")
        return

    # Find the section header line before 177 (should be "## Input — 输入处理")
    section_start = start_idx
    for i in range(start_idx - 1, max(start_idx - 5, 0), -1):
        if lines[i].startswith("## "):
            section_start = i
            break

    # Find the next section header after 205 (should be "## Gizmos — 调试绘图")
    next_section_start = end_idx + 1
    while next_section_start < len(lines) and not lines[next_section_start].startswith(
        "## "
    ):
        next_section_start += 1

    # Build section headers matching original structure
    input_header = "## Input — 输入处理"
    audio_header = "## Audio — 音频"
    camera_header = "## Camera — 相机"

    table_header = "| 序号 | 分类 | 文件名 | 意图及技术细节 | 前置知识及前置例子 |\n|------|--------|--------|------|------|"

    def build_entry(entry):
        seq, cat, intents, knowledge, examples, filename = entry
        intent_text = "<br>".join(intents)
        knowledge_text = "<br>".join(knowledge)
        examples_text = "<br>".join(examples)
        return (
            f"| {seq} | {cat} | `{filename}` "
            f"| {intent_text} "
            f"| 知：{knowledge_text}例：{examples_text} |"
        )

    new_lines = []

    # Input section (177-189)
    new_lines.append(input_header)
    new_lines.append("")
    new_lines.append(table_header)
    for seq in range(177, 190):
        new_lines.append(build_entry(DATA_MAP[seq]))

    new_lines.append("")

    # Audio section (190-196)
    new_lines.append("---")
    new_lines.append("")
    new_lines.append(audio_header)
    new_lines.append("")
    new_lines.append(table_header)
    for seq in range(190, 197):
        new_lines.append(build_entry(DATA_MAP[seq]))

    new_lines.append("")

    # Camera section (197-205)
    new_lines.append("---")
    new_lines.append("")
    new_lines.append(camera_header)
    new_lines.append("")
    new_lines.append(table_header)
    for seq in range(197, 206):
        new_lines.append(build_entry(DATA_MAP[seq]))

    new_lines.append("")

    # Reconstruct the file
    before = lines[:section_start]
    after = lines[next_section_start:]

    result = before + new_lines + after

    with open(CATALOG, "w", encoding="utf-8") as f:
        f.write("\n".join(result))

    print(f"OK: Replaced entries 177-205 ({len(DATA)} entries) in {CATALOG}")
    print(f"  Section start: line {section_start + 1}")
    print(f"  Next section: line {next_section_start + 1}")


if __name__ == "__main__":
    main()
