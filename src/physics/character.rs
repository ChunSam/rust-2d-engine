use rapier2d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier2d::na;

/// 키네마틱 캐릭터 컨트롤러 컴포넌트.
///
/// `PhysicsBody`와 함께 같은 엔티티에 붙여 사용하며,
/// `PhysicsWorld::move_character()`를 통해 매 프레임 이동을 처리한다.
///
/// ```rust,ignore
/// // 엔티티 생성 시
/// let (rb, col) = physics.add_kinematic_box(start_pos / PPU, 0.4, 0.9);
/// let player = world.spawn();
/// world.add_component(player, PhysicsBody { rigid_body_handle: rb, collider_handle: col });
/// world.add_component(player, CharacterController::new());
///
/// // 시스템 run() 내
/// let desired = Vec2::new(move_x * speed * dt, gravity_vel * dt);
/// physics.move_character(controller, body.rigid_body_handle, body.collider_handle, desired, dt, PPU);
/// if controller.grounded { /* 점프 가능 */ }
/// ```
pub struct CharacterController {
    /// 기어오를 수 있는 최대 경사면 각도 (라디안). 기본값 π/4 (45°).
    pub max_slope_angle: f32,
    /// 이전 `move_character` 호출 후 접지 여부.
    pub grounded: bool,
    pub(crate) inner: KinematicCharacterController,
}

impl Default for CharacterController {
    fn default() -> Self {
        // 엔진은 화면 좌표(Y+는 아래)를 사용하므로 up = -Y
        let inner = KinematicCharacterController {
            up: na::Unit::new_normalize(na::Vector2::new(0.0, -1.0)),
            max_slope_climb_angle: std::f32::consts::FRAC_PI_4,
            min_slope_slide_angle: std::f32::consts::FRAC_PI_4,
            snap_to_ground: Some(CharacterLength::Absolute(0.1)),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.3),
                min_width: CharacterLength::Absolute(0.1),
                include_dynamic_bodies: false,
            }),
            slide: true,
            ..Default::default()
        };

        Self {
            max_slope_angle: std::f32::consts::FRAC_PI_4,
            grounded: false,
            inner,
        }
    }
}

impl CharacterController {
    pub fn new() -> Self {
        Self::default()
    }

    /// 경사면 각도를 도(degree) 단위로 설정한다.
    pub fn with_max_slope_deg(mut self, degrees: f32) -> Self {
        let rad = degrees.to_radians();
        self.max_slope_angle = rad;
        self.inner.max_slope_climb_angle = rad;
        self.inner.min_slope_slide_angle = rad;
        self
    }

    /// 계단 최대 높이를 물리 단위로 설정한다.
    pub fn with_autostep(mut self, max_height: f32, min_width: f32) -> Self {
        self.inner.autostep = Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(max_height),
            min_width: CharacterLength::Absolute(min_width),
            include_dynamic_bodies: false,
        });
        self
    }

    /// 접지 스냅 거리를 물리 단위로 설정한다. 0.0이면 스냅 비활성.
    pub fn with_snap_to_ground(mut self, distance: f32) -> Self {
        if distance > 0.0 {
            self.inner.snap_to_ground = Some(CharacterLength::Absolute(distance));
        } else {
            self.inner.snap_to_ground = None;
        }
        self
    }
}
