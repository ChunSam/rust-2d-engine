# skeleton-engine 엔진 용어 입문서

이 문서는 `skeleton-engine`에 구현된 기능을 이해하기 위한 전문용어 학습 자료다.
학습자는 게임 엔진을 처음 접한다고 가정한다.

목표는 "이 용어가 무엇인지", "게임에서 왜 필요한지", "이 엔진에서는 어떤 타입이나 파일로 구현되어 있는지"를 연결해서 익히는 것이다.

## 먼저 잡아야 할 큰 그림

게임 엔진은 게임을 만들 때 반복해서 필요한 공통 기능을 모아둔 프로그램 뼈대다.
예를 들어 창을 만들고, 입력을 받고, 캐릭터를 움직이고, 이미지를 그리고, 소리를 재생하고, 충돌을 계산하는 일을 엔진이 담당한다.

`skeleton-engine`은 Rust로 만든 2D 게임 엔진이다. 렌더링은 `wgpu`, 창과 입력은 `winit`, 물리는 `rapier2d`, 오디오는 `rodio`, 텍스트는 `glyphon`을 기반으로 한다.

한 프레임의 흐름은 대략 다음과 같다.

1. 운영체제에서 키보드, 마우스, 터치, 창 크기 변경 같은 이벤트를 받는다.
2. `InputState`, `TouchState`, `GamepadState` 같은 입력 리소스를 갱신한다.
3. 등록된 `System`들이 `World`를 읽고 게임 상태를 바꾼다.
4. 씬 전환, 이벤트 정리, 입력의 `just_pressed` 같은 일회성 상태 정리를 처리한다.
5. 스프라이트, UI, 텍스트, 파티클, 포스트 프로세스를 화면에 그린다.

## 학습 순서

처음에는 모든 용어를 외우려고 하지 않는 편이 좋다. 아래 순서로 읽으면 이해가 쉽다.

1. `App`, `World`, `Entity`, `Component`, `System`을 먼저 익힌다.
2. `Transform`, `Sprite`, `Camera`, `InputState`로 화면에 움직이는 물체를 만드는 구조를 익힌다.
3. `Scene`, `AssetServer`, `AnimationPlayer`, `PhysicsWorld`, `UiNode`처럼 게임 규모가 커질 때 필요한 기능을 익힌다.
4. `Prefab`, `Reflect`, `Scripting`, `BehaviorTree`, `Localization`, `Save` 같은 제작 편의 기능을 익힌다.
5. `RenderTarget`, `PostProcess`, `GPU Particle`, `WASM` 같은 고급 기능을 마지막에 익힌다.

## 기본 실행 구조

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Engine | 게임 제작에 필요한 공통 기능 묶음 | 전체 crate |
| Crate | Rust 패키지 또는 라이브러리 단위 | 패키지 `skeleton-engine`, 라이브러리 `engine` |
| App | 엔진 실행의 진입점 | `App` |
| Game Loop | 게임이 켜져 있는 동안 매 프레임 반복되는 흐름 | `App::run`, 내부 `update`와 `render` |
| Frame | 화면이 한 번 갱신되는 단위 | `dt`를 받아 시스템들이 한 번 실행됨 |
| Delta Time | 지난 프레임부터 현재 프레임까지 걸린 시간 | `System::run(&mut self, world, dt)`의 `dt` |
| Runtime | 게임이 실제로 실행되는 상태 | `App`, `World`, 렌더러, 입력 상태 전체 |
| Resource | 전역 상태 데이터 | `WindowConfig`, `InputState`, `TextQueue`, `AssetServer` |

`App`은 엔진의 중심 객체다. `World`를 들고 있고, 시스템을 등록하고, 창과 GPU를 초기화하고, 매 프레임 업데이트와 렌더링을 진행한다.

## ECS

ECS는 Entity Component System의 약자다. 게임 오브젝트를 상속 구조로 만들지 않고, 작은 데이터 조각을 조합해서 표현하는 방식이다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Entity | 게임 안의 물체를 가리키는 ID | `Entity(pub u32)` |
| Component | 엔티티에 붙는 데이터 조각 | `Transform`, `Sprite`, `PhysicsBody` 등 |
| System | 컴포넌트를 읽고 수정하는 로직 | `System` trait |
| World | 모든 엔티티, 컴포넌트, 리소스가 들어 있는 저장소 | `World` |
| Query | 특정 컴포넌트를 가진 엔티티를 찾는 기능 | `world.query::<T>()`, `query2`, `query3` |
| Resource | 엔티티에 속하지 않는 전역 데이터 | `world.insert_resource(...)` |
| Commands | 엔티티 생성/삭제/컴포넌트 추가를 나중에 적용하는 버퍼 | `Commands` |
| Event | 프레임 동안 시스템끼리 주고받는 메시지 | `Events<E>` |
| Schedule | 시스템 실행 순서를 정하는 규칙 | `SystemConfig`, `SystemLabel` |
| System Set | 여러 시스템을 그룹으로 묶어 켜고 끄는 단위 | `SystemConfig::in_set` |
| Change Detection | 컴포넌트가 추가되거나 바뀌었는지 추적 | `World`의 added/changed 추적 API |

예를 들어 플레이어는 하나의 `Entity`이고, 여기에 위치를 나타내는 `Transform`, 그림을 나타내는 `Sprite`, 플레이어 표식을 위한 `Player` 컴포넌트를 붙인다. 이동 시스템은 `Player`와 `Transform`을 가진 엔티티를 찾아 위치를 바꾼다.

## 씬과 게임 상태

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Scene | 메뉴, 게임플레이, 로딩 화면처럼 하나의 화면 상태 | `Scene` trait |
| Scene Change | 현재 씬을 바꾸는 명령 | `SceneChange`, `SceneCmd` |
| Push Scene | 기존 씬 위에 새 씬을 올림 | `SceneCmd::Push` |
| Pop Scene | 위에 올린 씬을 닫고 이전 씬으로 돌아감 | `SceneCmd::Pop` |
| Replace Scene | 현재 씬을 새 씬으로 교체 | `SceneCmd::Replace` |
| Game State | 게임 진행 상태를 나타내는 전역 값 | `GameState` |
| Fade Transition | 씬 전환 때 화면을 어둡게 하거나 밝게 하는 효과 | `FadeTransition` |

씬은 게임을 여러 화면으로 나누기 위한 단위다. 메인 메뉴, 플레이 화면, 일시정지 화면을 모두 같은 `World`에 억지로 넣으면 복잡해지므로, 씬 단위로 초기화와 정리를 나눈다.

## 좌표와 변환

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Transform | 위치, 크기, 회전을 묶은 데이터 | `Transform` |
| Position | 화면 또는 월드에서의 위치 | `Transform.position` |
| Scale | 물체의 크기 | `Transform.scale` |
| Rotation | 회전값, 이 엔진에서는 라디안 단위 | `Transform.rotation` |
| Z Order | 어떤 물체를 앞에 그릴지 정하는 값 | `Transform.z`, `RenderLayer` |
| Matrix | 위치, 크기, 회전을 GPU가 이해하는 형태로 합친 값 | `Transform::to_matrix` |
| World Space | 게임 세계 좌표 | 캐릭터, 타일맵, 물리 위치 |
| Screen Space | 화면 픽셀 기준 좌표 | UI, 마우스 커서, `DrawRect` |
| Viewport | 실제로 보이는 화면 영역 | `ViewportSize` |

2D 게임에서도 좌표계는 중요하다. 캐릭터는 월드 좌표에 있고, 버튼이나 HUD는 화면 좌표에 있다. 카메라는 월드 좌표를 화면에 어떻게 보여줄지 결정한다.

## 렌더링

렌더링은 게임 데이터를 화면 이미지로 바꾸는 작업이다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Renderer | 화면에 그리는 담당 코드 | `renderer` 모듈 |
| GPU | 그래픽 계산을 담당하는 장치 | `wgpu`를 통해 사용 |
| Sprite | 2D 이미지 또는 단색 사각형 | `Sprite` |
| Texture | GPU에 올라간 이미지 | `Texture`, `ImageAsset` |
| Texture Handle | 이미지 에셋을 가리키는 타입 안전 참조 | `Handle<ImageAsset>` |
| Texture Atlas | 여러 이미지를 하나의 큰 이미지에 모아둔 것 | `TextureAtlas` |
| Atlas Sprite | 아틀라스 안의 특정 칸을 그리는 스프라이트 | `AtlasSprite` |
| UV | 텍스처 안에서 어느 부분을 샘플링할지 나타내는 0.0-1.0 좌표 | `UvRect` |
| UV Flip | 같은 이미지를 좌우 또는 상하로 뒤집어 읽는 것 | `UvRect::flipped_x`, `flipped_y` |
| Render Layer | 렌더링 순서를 나누는 층 | `RenderLayer` |
| Culling | 화면 밖 오브젝트를 그리지 않아 성능을 아끼는 것 | `CullConfig` |
| Render Stats | 이번 프레임 렌더링 통계 | `RenderStats` |

`Sprite::textured_with_handle`과 `DrawImage::textured_with_handle`은 핸들이 있으면 핸들을 쓰고, 없으면 문자열 경로를 fallback으로 쓰는 도우미다. 에셋 로딩이 늦거나 테스트 환경처럼 핸들이 없을 수 있는 상황을 처리하기 위한 기능이다.

## 카메라

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Camera | 월드의 어느 부분을 볼지 정하는 눈 | `Camera` |
| View Projection | 카메라 좌표를 화면 좌표로 바꾸는 행렬 | `Camera::view_proj` |
| Zoom | 카메라 확대/축소 | `Camera.zoom`, `zoom_to` |
| Screen to World | 화면 좌표를 월드 좌표로 변환 | `Camera::screen_to_world` |
| Visible Rect | 현재 카메라에 보이는 월드 영역 | `Camera::visible_rect` |
| Camera Shake | 충격 표현을 위해 카메라를 흔드는 효과 | `Camera::shake` |
| Smooth Follow | 목표를 부드럽게 따라가는 카메라 움직임 | `Camera::update(..., follow_pos)` |

마우스로 월드의 적을 클릭하려면 마우스 좌표는 화면 좌표이고 적의 위치는 월드 좌표이므로 `screen_to_world` 같은 변환이 필요하다.

## 텍스트와 UI

UI는 체력바, 메뉴 버튼, 인벤토리, 설정 화면처럼 게임 위에 올라가는 인터페이스다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| UI | 사용자가 보는 인터페이스 | `ui` 모듈 |
| Widget | 버튼, 라벨, 슬라이더 같은 UI 부품 | `Button`, `Label`, `Slider` |
| UI Node | UI 요소의 위치와 크기 정보 | `UiNode` |
| Anchor | 화면의 어느 기준점에 붙일지 정하는 값 | `Anchor` |
| Layout | 자식 UI를 자동 배치하는 규칙 | `Panel`, `LayoutSystem` |
| Button | 클릭 가능한 UI | `Button`, `ButtonState` |
| Text Input | 글자를 입력하는 UI | `TextInput` |
| Scroll View | 내용이 길 때 스크롤하는 영역 | `ScrollView` |
| CheckBox | 켜기/끄기 선택 UI | `CheckBox` |
| Virtual Joystick | 터치 화면용 가상 조이스틱 | `VirtualJoystick` |
| Draw Rect | 화면 좌표 사각형 그리기 명령 | `DrawRect`, `UiQueue` |
| Draw Image | 화면 좌표 이미지 그리기 명령 | `DrawImage`, `UiImageQueue` |
| Text Queue | 매 프레임 그릴 텍스트 명령 모음 | `TextQueue` |
| Rich Text | 텍스트 안에 색상, 볼드, 이탤릭 태그를 넣는 기능 | `DrawText::rich` |
| IME | 한글, 일본어, 중국어 조합 입력 처리 | `InputState::ime_preedit`, `TextInput` |

`DrawRect`, `DrawImage`, `DrawText`는 월드에 엔티티를 만들지 않고 화면 좌표로 바로 그리는 명령이다. 체력바, HUD 아이콘, 로딩 바처럼 카메라 움직임과 무관해야 하는 요소에 적합하다.

## 입력

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Input State | 현재 입력 상태 | `InputState` |
| Pressed | 키나 버튼을 누르고 있는 상태 | `is_pressed` |
| Just Pressed | 이번 프레임에 막 눌린 상태 | `just_pressed` |
| Just Released | 이번 프레임에 막 뗀 상태 | `just_released` |
| Cursor | 마우스 포인터 위치 | `InputState::cursor` |
| Scroll | 마우스 휠 입력 | `InputState::scroll` |
| Text Input | 키보드가 만든 문자 입력 | `InputState::text_chars` |
| Input Map | 액션 이름과 키를 연결하는 리바인딩 테이블 | `InputMap` |
| Gamepad | 게임패드 입력 | `GamepadState`, `GamepadButton`, `GamepadAxis` |
| Touch | 터치 입력 | `TouchState` |
| Swipe | 손가락을 쓸어 넘기는 제스처 | `TouchState` |
| Pinch Zoom | 두 손가락 간격 변화로 확대/축소하는 제스처 | `TouchState` |

`just_pressed`는 점프처럼 한 번만 실행되어야 하는 입력에 쓰고, `is_pressed`는 이동처럼 누르고 있는 동안 계속 적용되는 입력에 쓴다.

## 물리와 충돌

물리는 중력, 속도, 충돌, 센서, 레이캐스트 같은 계산을 담당한다. 이 엔진은 네이티브 빌드에서 `rapier2d`를 사용한다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Physics World | 물리 시뮬레이션이 진행되는 공간 | `PhysicsWorld` |
| Rigid Body | 물리 세계의 움직이는 물체 본체 | `PhysicsBody` 내부 handle |
| Collider | 충돌 모양 | Rapier collider, 경량 `Collider` |
| Dynamic Body | 중력과 힘에 반응하는 바디 | `add_dynamic_box`, `add_dynamic_circle` |
| Static Body | 움직이지 않는 벽, 바닥 | `add_static_box` |
| Kinematic Body | 코드는 움직이지만 물리 힘에는 밀리지 않는 바디 | `add_kinematic_box`, `add_kinematic_circle` |
| Sensor | 부딪히지 않고 들어옴/나감만 감지하는 영역 | `add_sensor_box`, `TriggerEvent` |
| Trigger Zone | 센서로 만든 감지 구역 | `TriggerEvent::Entered`, `Exited` |
| Collision Event | 두 물체가 충돌 시작/종료했음을 알리는 이벤트 | `CollisionEvent` |
| Collision Groups | 어떤 레이어끼리 충돌할지 정하는 비트마스크 | `CollisionGroups` |
| Raycast | 보이지 않는 선을 쏴서 처음 맞는 물체를 찾는 기능 | `cast_ray`, `RaycastHit` |
| Character Controller | 캐릭터 이동, 경사, 계단, 접지 처리를 돕는 물리 컨트롤러 | `CharacterController` |
| Joint | 두 바디를 연결하는 물리 제약 | `add_distance_joint`, `add_revolute_joint`, `add_prismatic_joint` |
| Spatial Grid | 많은 충돌 후보를 빠르게 좁히는 격자 구조 | `SpatialGrid` |

센서는 "문 앞에 들어오면 문 열기", "아이템 근처에 가면 줍기", "함정 영역에 들어가면 데미지" 같은 로직에 쓴다. 실제로 밀어내는 충돌은 하지 않고 이벤트만 만든다.

## 애니메이션

애니메이션은 시간에 따라 이미지 프레임이나 상태를 바꾸는 기능이다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Animation Clip | 하나의 애니메이션 묶음 | `AnimationClip` |
| Frame | 애니메이션의 한 장면 | `UvRect` 목록의 한 항목 |
| FPS | 초당 프레임 수 | `AnimationClip.fps` |
| Looping | 끝나면 다시 처음부터 재생 | `AnimationClip.looping` |
| Animation Player | 현재 어떤 클립과 프레임을 재생하는지 가진 컴포넌트 | `AnimationPlayer` |
| Animation System | 매 프레임 애니메이션을 진행하는 시스템 | `AnimationSystem` |
| Crossfade | 두 애니메이션을 부드럽게 섞어 전환 | `play_with_crossfade`, `BlendWeight` |
| State Machine | 상태와 전환 조건으로 애니메이션을 바꾸는 구조 | `AnimationStateMachine` |
| Transition | 한 상태에서 다른 상태로 넘어가는 규칙 | `AnimTransition` |
| Trigger | 한 프레임만 소비되는 전환 신호 | `AnimParam`, `fire_trigger` |
| Blend Tree | 속도 같은 값에 따라 여러 애니메이션을 섞는 구조 | `BlendTree1D` |
| Blend Weight | 섞는 비율 | `BlendWeight` |

예를 들어 캐릭터가 서 있을 때는 `Idle`, 움직일 때는 `Run`, 공격할 때는 `Attack` 클립을 재생한다. 상태 머신은 `speed > 0`이면 `Idle`에서 `Run`으로 전환하는 식의 규칙을 관리한다.

## 에셋

에셋은 이미지, 스크립트, 아틀라스처럼 게임 밖 파일에서 읽어오는 데이터다.

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Asset | 게임이 사용하는 외부 데이터 | 이미지, 스크립트, 아틀라스 |
| Asset Server | 에셋을 로드하고 캐싱하는 관리자 | `AssetServer` |
| Handle | 에셋을 직접 들고 있지 않고 참조하는 가벼운 값 | `Handle<T>` |
| Cache | 같은 파일을 여러 번 로드하지 않도록 저장하는 공간 | `AssetServer` 내부 |
| Hot Reload | 파일이 바뀌면 실행 중 다시 반영하는 기능 | `AssetServer::poll_reloads` |
| Load State | 로딩 중, 성공, 실패 상태 | `AssetLoadState` |
| Async Loading | 게임을 멈추지 않고 백그라운드에서 로딩 | `load_image_async` |
| Fallback Texture | 로드 실패 시 대신 표시하는 기본 텍스처 | 마젠타 1x1 텍스처 |
| WASM Fetch | 브라우저에서 네트워크로 에셋을 받아오는 방식 | WASM 전용 async 로딩 |

핸들은 이미지 자체가 아니라 "이 이미지를 가리키는 표식"이다. 여러 스프라이트가 같은 이미지를 써도 이미지 데이터는 한 번만 관리할 수 있다.

## 타일맵과 경로 탐색

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Tile | 격자 하나에 놓이는 작은 그림 | `Tilemap.tiles`의 값 |
| Tilemap | 타일을 격자로 배치한 맵 | `Tilemap` |
| Tile Atlas | 타일 이미지가 모여 있는 아틀라스 | `TilemapAtlas` |
| Tile Size | 타일 한 칸의 픽셀 크기 | `Tilemap.tile_size` |
| Origin | 타일맵 좌상단 기준 위치 | `Tilemap.origin` |
| Pathfinding | 목표까지 이동 경로를 찾는 기능 | `find_path` |
| A Star | 비용이 낮은 경로를 우선 탐색하는 알고리즘 | `find_path` 구현 |
| Walkable | 지나갈 수 있는 칸 | `PathGrid::is_walkable` |
| Manhattan Distance | 격자에서 상하좌우 이동 거리 추정값 | A* 휴리스틱 |

타일맵은 바닥, 벽, 장식 같은 반복 패턴을 효율적으로 배치하는 데 쓰인다. 경로 탐색은 적 AI가 플레이어에게 찾아오거나 NPC가 목적지로 이동할 때 쓴다.

## 파티클과 시각 효과

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Particle | 불꽃, 먼지, 피격 효과처럼 작고 짧게 사는 시각 요소 | `Particle` |
| Emitter | 파티클을 계속 만들어내는 발생기 | `ParticleEmitter` |
| Spawn Rate | 초당 파티클 생성 수 | `spawn_rate` |
| Lifetime | 파티클이 살아 있는 시간 | `lifetime` |
| CPU Particle | CPU에서 위치와 색을 갱신하는 파티클 | `ParticleSystem` |
| GPU Particle | GPU compute shader로 갱신하는 파티클 | `GpuParticleEmitter` |
| Post Process | 화면 전체에 마지막으로 거는 효과 | `PostProcessConfig` |
| Vignette | 화면 가장자리를 어둡게 하는 효과 | `vignette_strength` |
| Chromatic Aberration | RGB 채널을 약간 어긋나게 하는 효과 | `chroma_offset` |
| Bloom | 밝은 부분이 번져 보이는 효과 | `bloom_threshold`, `bloom_intensity` |
| Color Grading | 밝기, 대비, 채도를 조정하는 색보정 | `brightness`, `contrast`, `saturation` |

CPU 파티클은 구조가 단순하고 디버깅이 쉽다. GPU 파티클은 많은 수의 파티클을 처리할 때 유리하지만 네이티브 전용 기능이다.

## 라이팅과 셰이더

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Lighting | 빛을 계산해 장면을 밝히는 기능 | `LightingRenderer` |
| Ambient Light | 전체 장면에 깔리는 기본 빛 | `AmbientLight` |
| Point Light | 한 지점에서 퍼지는 빛 | `PointLight` |
| Normal Map | 표면의 방향 정보를 담은 텍스처 | `Sprite.normal_texture` |
| Light Height | 2D 조명에서 가상의 빛 높이 | `PointLight.light_height` |
| Shader | GPU에서 실행되는 작은 프로그램 | WGSL shader |
| Material | 어떤 셰이더와 파라미터로 그릴지 나타내는 데이터 | `ShaderMaterial` |
| Uniform | CPU에서 GPU 셰이더로 넘기는 공통 파라미터 | `ShaderMaterial.params` |

노멀 맵 라이팅은 2D 이미지에 입체적인 빛 반응을 주기 위한 기술이다. 그림은 2D지만 표면 방향 정보를 추가해서 빛을 받는 느낌을 낸다.

## 렌더 텍스처

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Render Target | 화면이 아닌 텍스처에 그리기 위한 대상 | `RenderTarget` |
| Offscreen Rendering | 화면 밖 중간 텍스처에 먼저 그리는 것 | `OffscreenCamera`, `RenderTarget` |
| Minimap | 월드를 작은 지도처럼 다시 그린 화면 | `examples/minimap.rs` |
| Split Screen | 화면을 나눠 여러 카메라를 보여주는 방식 | `examples/split_screen.rs` |
| Texture Binding | 렌더된 텍스처를 다시 스프라이트처럼 쓰는 연결 | `RenderTarget.bind_group` |

렌더 텍스처는 미니맵, CCTV 화면, 포탈, 분할 화면처럼 "장면을 한 번 더 그려서 이미지처럼 사용"해야 할 때 필요하다.

## 오디오

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Audio Manager | 소리 재생 관리자 | `AudioManager` |
| Channel | 특정 소리를 식별하는 이름 | `"bgm"`, `"sfx_jump"` 같은 문자열 |
| BGM | 배경 음악 | 반복 재생 채널 |
| SFX | 효과음 | 짧은 단발 소리 |
| Volume | 소리 크기 | `set_volume` |
| Fade In | 소리가 서서히 커짐 | `play_fade_in` |
| Fade Out | 소리가 서서히 작아진 뒤 정지 | `fade_out` |
| Audio Bus | 여러 채널을 묶어 볼륨을 같이 조절하는 그룹 | `assign_bus`, `set_bus_volume` |
| Panning | 소리가 왼쪽/오른쪽에서 나는 것처럼 조절 | `set_pan` |
| Positional Audio | 소리 발생 위치와 리스너 위치로 볼륨/팬을 계산 | `play_at`, `update_position` |
| Low Pass Filter | 고음을 줄여 먹먹하게 만드는 필터 | `AudioEffect.low_pass_hz` |
| Pitch | 재생 속도와 음높이 | `AudioEffect.pitch` |
| Envelope | 소리가 시작할 때의 볼륨 변화 같은 시간 기반 음량 제어 | `AudioEffect.attack_secs`, `fade_out`, `fade_volume` |

게임에서는 음악과 효과음을 따로 조절해야 하므로 오디오 버스가 유용하다. 예를 들어 설정 화면에서 `music` 버스만 줄이면 BGM은 작아지고 효과음은 그대로 둘 수 있다.

## 저장과 직렬화

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Save | 게임 상태를 파일로 저장 | `save`, `save_with_key` |
| Load | 저장 파일을 읽어 게임 상태로 복원 | `load`, `load_with_key` |
| Serialization | 메모리 데이터를 파일 문자열/바이트로 바꾸기 | `serde`, `ron` |
| Deserialization | 파일 데이터를 다시 메모리 구조로 복원 | `load` |
| RON | Rust 데이터와 잘 맞는 텍스트 데이터 형식 | 저장 내부 plaintext 형식 |
| Encryption | 저장 내용을 읽기 어렵게 암호화 | `ChaCha20Poly1305` |
| AEAD | 암호화와 변조 검증을 함께 하는 방식 | 저장 파일 보안 방식 |
| Nonce | 같은 키로 매번 다른 암호문을 만들기 위한 값 | 저장 파일 헤더에 포함 |
| Save Key | 저장 파일 암호화 키 | `SaveKey` |
| Corrupted | 저장 파일이 깨졌거나 변조됨 | `SaveError::Corrupted` |

이 엔진의 저장 기능은 RON으로 직렬화한 뒤 AEAD로 암호화한다. 단, 클라이언트 바이너리에 들어 있는 기본 키는 완전한 비밀이 아니므로 치트 방지의 절대 경계로 보면 안 된다.

## 프리팹, 씬 데이터, 리플렉션

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Prefab | 미리 만들어 둔 엔티티 묶음 | `Prefab` |
| Prefab Instance | 프리팹에서 생성된 실제 엔티티 | `PrefabInstance` |
| SceneDef | 파일로 저장 가능한 씬 데이터 | `SceneDef` |
| EntityDef | 파일로 저장 가능한 엔티티 정의 | `EntityDef` |
| Tag | 엔티티를 이름으로 구분하는 문자열 표식 | `Tag` |
| Schema Version | 저장 데이터 형식의 버전 | `SCENE_DEF_VERSION` |
| Hierarchy | 부모-자식 엔티티 구조 | `Parent`, `Children`, `HierarchySystem` |
| Global Transform | 부모 변환이 반영된 최종 위치 | `GlobalTransform` |
| Reflection | 타입을 몰라도 필드 목록을 읽고 수정하는 기능 | `Reflect`, `ReflectValue` |
| Inspector | 실행 중 엔티티와 컴포넌트를 확인/수정하는 도구 | `DebugUi`, egui 기반 에디터 UI |
| Undo/Redo | 편집 작업 되돌리기/다시 실행 | `App` 내부 editor history |
| Clone Entity | 엔티티와 컴포넌트를 복제하는 기능 | `World::clone_entity` 관련 registry |

리플렉션은 에디터를 만들 때 중요하다. `Transform`의 `x`, `y`, `rotation` 같은 필드를 타입별 전용 UI 없이 공통 방식으로 보여주고 바꿀 수 있기 때문이다.

## 스크립팅

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Scripting | Rust 코드를 다시 컴파일하지 않고 외부 스크립트로 로직 실행 | `ScriptingSystem` |
| Rhai | 이 엔진이 사용하는 스크립트 언어 | `rhai` |
| Script Asset | 로드된 스크립트 파일 | `ScriptAsset` |
| Script Runner | 스크립트를 실행하는 도우미 | `ScriptRunner` |
| AST | 파싱된 스크립트 구조 | `ScriptAsset.ast` |
| Script Limits | 스크립트 실행 제한 | `ScriptingLimits` |

스크립팅은 몬스터 패턴, 이벤트 트리거, 간단한 상호작용처럼 자주 바뀌는 로직을 빠르게 수정하는 데 유용하다.

## AI와 게임플레이 로직

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Behavior Tree | AI 행동을 트리 구조로 표현하는 방식 | `BehaviorTree` |
| Behavior Node | 행동 트리의 한 노드 | `BehaviorNode` |
| Tick | 한 프레임 동안 행동 노드를 실행하는 것 | `BehaviorNode::tick` |
| Status | 행동 실행 결과 | `BehaviorStatus` |
| Sequence | 자식 행동을 순서대로 모두 성공해야 성공 | `Sequence` |
| Selector | 자식 행동 중 하나라도 성공하면 성공 | `Selector` |
| Inverter | 성공과 실패를 뒤집는 노드 | `Inverter` |
| Blackboard | AI가 공유하는 상태 저장소 | `Blackboard` |
| Steering | 목표를 향해 움직이거나 피하는 이동 행동 | `SteeringSystem` |
| Seek | 목표를 향해 이동 | `Seek` |
| Flee | 목표에서 도망 | `Flee` |
| Arrive | 목표 근처에서 감속하며 도착 | `Arrive` |
| Wander | 무작위로 배회 | `Wander` |

비헤이비어 트리는 "플레이어가 보이면 추적, 가까우면 공격, 아니면 순찰" 같은 결정을 만들 때 쓰고, 스티어링은 실제 이동 방향과 속도를 만드는 데 쓴다.

## 시간 제어

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Timer | 일정 시간이 지났는지 재는 도구 | `Timer` |
| Tween | 값이 시작값에서 끝값으로 부드럽게 변하는 것 | `Tween` |
| Easing | 트윈이 빠르게 시작할지, 천천히 끝날지 정하는 곡선 | `Easing` |
| Timeline | 여러 값을 시간표처럼 키프레임으로 제어 | `Timeline` |
| Keyframe | 특정 시간의 특정 값 | `Keyframe<T>` |
| Track | 같은 종류의 키프레임 묶음 | `Track<T>` |
| Cutscene | 카메라나 오브젝트를 시간에 맞춰 연출하는 장면 | `TimelineSystem` 활용 |
| Lerp | 두 값 사이를 보간하는 계산 | `Lerp` trait |

예를 들어 문이 0.5초 동안 열리게 하거나, 카메라가 2초 동안 보스에게 이동하게 하거나, 화면이 서서히 어두워지는 효과를 만들 때 타이머, 트윈, 타임라인을 쓴다.

## 로컬라이제이션

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Localization | 언어별 텍스트와 폰트를 바꾸는 기능 | `LocaleResource` |
| Locale | 언어/지역 코드 | `"en"`, `"ko"`, `"ja"` |
| Translation Key | 실제 문장 대신 쓰는 키 | `"menu.start"` |
| Bundle | 여러 언어 데이터를 묶은 파일/데이터 | `LocaleBundle` |
| Fallback | 현재 언어에 번역이 없을 때 기본 언어 또는 키를 쓰는 동작 | `LocaleResource::t` |
| Font per Locale | 언어별 폰트 지정 | `LocaleData.font` |
| Text Direction | 글자가 흐르는 방향 | `TextDirection::LeftToRight`, `RightToLeft` |
| RTL | 오른쪽에서 왼쪽으로 쓰는 언어 방향 | 아랍어 등 |

로컬라이제이션은 게임 출시 후 붙이기 어렵다. UI 텍스트를 처음부터 키 기반으로 관리하면 언어 추가가 쉬워진다.

## 네트워크

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| WebSocket | 서버와 계속 연결된 채로 메시지를 주고받는 통신 방식 | `NetworkClient` |
| Client | 서버에 접속하는 쪽 | `NetworkClient` |
| Network Event | 연결, 해제, 메시지 수신 같은 네트워크 알림 | `NetworkEvent` |
| Text Message | 문자열 메시지 | `NetworkEvent::TextMessage` |
| Binary Message | 바이트 메시지 | `NetworkEvent::BinaryMessage` |
| Send Queue | 보낼 메시지를 임시로 쌓아두는 큐 | `NetworkConfig.max_pending_messages` |
| Message Limit | 너무 큰 메시지를 거르는 제한 | `NetworkConfig.max_message_bytes` |
| Native Thread | 네이티브 빌드에서 네트워크를 백그라운드 스레드로 처리 | `tungstenite` 기반 구현 |
| Browser WebSocket | WASM 빌드에서 브라우저 WebSocket API 사용 | `web_sys::WebSocket` |

네트워크 기능은 멀티플레이나 외부 서버 연동의 기반이다. 엔진은 메시지를 직접 게임 로직으로 처리하지 않고 `NetworkEvent`로 넘긴다.

## 오브젝트 풀과 성능

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Object Pool | 자주 만들고 지우는 객체를 재사용하는 구조 | `Pool` |
| Pooled | 풀에 반납되어 비활성화된 엔티티 표식 | `Pooled` |
| Allocation | 메모리를 새로 확보하는 작업 | 엔티티/컴포넌트 생성 비용 |
| Reuse | 기존 엔티티를 다시 초기화해 사용하는 것 | `Pool::acquire` |
| Release | 사용이 끝난 엔티티를 풀에 반납 | `Pool::release` |
| Profiler | 어떤 시스템이 시간을 얼마나 쓰는지 재는 도구 | `ProfilerData`, `SystemProfile` |
| Panic Recovery | 시스템이 panic해도 엔진이 계속 살아남게 하는 처리 | `PanickedSystems`, crash log |

총알, 이펙트, 데미지 숫자처럼 짧게 많이 생기는 오브젝트는 매번 새로 만들면 비용이 커진다. 풀을 쓰면 성능이 안정적이다.

## 디버그 도구

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Debug UI | 실행 중 상태를 보는 개발자용 UI | `DebugUi` |
| Debug Draw | 선, 박스, 원을 임시로 그려 확인하는 기능 | `DebugDraw`, `DebugDrawQueue` |
| Collision Debug | 충돌 영역을 화면에 표시 | `CollisionDebugSystem` |
| Selected Entity | 에디터에서 현재 선택된 엔티티 | `SelectedEntity` |
| Crash Log | 시스템 panic 정보를 남기는 파일 | `write_crash_log` 경로 |

디버그 도구는 최종 게임 화면을 꾸미는 기능이 아니라 개발 중 문제를 빨리 찾기 위한 기능이다. 충돌 박스가 왜 안 맞는지, 어떤 엔티티가 선택됐는지 확인할 때 쓴다.

## 플랫폼과 배포

| 용어 | 쉬운 뜻 | 이 엔진에서의 이름 |
| --- | --- | --- |
| Native Build | Windows, macOS, Linux 같은 데스크톱 실행 파일 | `cfg(not(target_arch = "wasm32"))` |
| WASM | 브라우저에서 실행할 수 있는 WebAssembly 빌드 | `wasm32-unknown-unknown` |
| WebGL2 | 브라우저에서 GPU 렌더링을 쓰는 그래픽 API | `wgpu`의 `webgl` feature |
| Feature Gate | 플랫폼별로 코드를 켜고 끄는 조건 | `#[cfg(...)]` |
| Package Include | 배포 패키지에 포함할 파일 목록 | `Cargo.toml`의 `include` |
| Release Profile | 릴리즈 빌드 최적화 설정 | `profile.release`, `profile.release-wasm` |

일부 기능은 네이티브 전용이다. 예를 들어 `AudioManager`, `PhysicsWorld`, `GpuParticleEmitter`는 현재 WASM이 아닌 빌드에서 공개된다. 브라우저에서는 OS API나 스레드, 일부 GPU 기능 제약이 있기 때문이다.

## 자주 헷갈리는 용어

| 헷갈리는 쌍 | 차이 |
| --- | --- |
| `Entity`와 `Component` | 엔티티는 ID이고, 컴포넌트는 그 ID에 붙는 데이터다. |
| `System`과 `Scene` | 시스템은 매 프레임 도는 로직이고, 씬은 화면 상태 단위다. |
| `Sprite`와 `Texture` | 스프라이트는 그릴 물체의 외형 컴포넌트이고, 텍스처는 실제 이미지 데이터다. |
| `World Space`와 `Screen Space` | 월드 좌표는 게임 세계 기준, 화면 좌표는 픽셀 화면 기준이다. |
| `Collider`와 `Sprite` | 콜라이더는 충돌 모양이고, 스프라이트는 보이는 그림이다. 둘은 같을 수도 다를 수도 있다. |
| `Collision`과 `Trigger` | 충돌은 물체를 밀어낼 수 있고, 트리거는 들어옴/나감만 감지한다. |
| `Animation Clip`과 `Animation State` | 클립은 실제 프레임 묶음이고, 상태는 현재 논리 상태다. |
| `Handle`과 실제 에셋 | 핸들은 참조 표식이고, 실제 데이터는 `AssetServer`가 가진다. |
| `Timer`와 `Tween` | 타이머는 시간이 지났는지 재고, 트윈은 값이 부드럽게 변하게 한다. |
| `Prefab`과 `SceneDef` | 프리팹은 재사용 가능한 일부 오브젝트 묶음, 씬 정의는 전체 씬 저장 데이터에 가깝다. |

## 기능별 대표 파일

| 기능 | 대표 파일 |
| --- | --- |
| 앱 루프와 렌더 orchestration | `src/app.rs` |
| 공개 API re-export | `src/lib.rs` |
| ECS 저장소 | `src/ecs/world.rs` |
| 시스템 순서 | `src/ecs/schedule.rs` |
| 기본 컴포넌트 | `src/components.rs` |
| 입력 | `src/input/` |
| 렌더링 | `src/renderer/` |
| 카메라 | `src/camera.rs` |
| 애니메이션 | `src/animation/` |
| 물리 | `src/physics/` |
| 경량 충돌 그리드 | `src/collision/` |
| UI | `src/ui/` |
| 에셋 | `src/asset.rs` |
| 프리팹과 씬 데이터 | `src/prefab.rs` |
| 저장/불러오기 | `src/save.rs` |
| 스크립팅 | `src/scripting.rs` |
| AI 행동 | `src/behavior.rs`, `src/steering.rs` |
| 타일맵과 경로 탐색 | `src/tilemap.rs`, `src/pathfinding.rs` |
| 오디오 | `src/audio.rs` |
| 파티클 | `src/particle.rs`, `src/gpu_particle.rs` |
| 로컬라이제이션 | `src/locale.rs` |
| 네트워크 | `src/network.rs` |

## 최소 예제로 연결해 보기

아래 문장을 이해할 수 있으면 엔진의 기본 구조를 잡은 것이다.

> `App`이 `World`를 가진다. `World` 안에는 `Entity`가 있고, 엔티티에는 `Transform`과 `Sprite` 같은 `Component`가 붙는다. `System`은 매 프레임 `Query`로 필요한 컴포넌트를 찾고, `dt`만큼 게임 상태를 갱신한다. 렌더러는 갱신된 `Transform`과 `Sprite`를 읽어 `Camera` 기준으로 화면에 그린다.

다음 단계로는 `examples/basic.rs`를 읽으면서 위 용어가 실제 코드에서 어디에 나타나는지 확인하면 된다.
