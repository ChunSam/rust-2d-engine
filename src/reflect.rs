use glam::Vec2;

/// 런타임에 읽고 쓸 수 있는 필드 값.
///
/// `Reflect::fields()`에서 반환되며, 에디터 Inspector에서 값을 편집하고
/// `Reflect::set_field()`로 다시 적용할 때 사용한다.
#[derive(Clone, Debug, PartialEq)]
pub enum ReflectValue {
    F32(f32),
    Vec2(Vec2),
    Bool(bool),
    String(String),
    Color([f32; 4]),
}

/// 런타임 필드 읽기/쓰기 트레잇.
///
/// 엔진 내장 컴포넌트(Transform, Sprite, Tag)에 구현되어 있으며,
/// 사용자 컴포넌트에도 수동으로 구현할 수 있다.
///
/// # egui Inspector 연동
/// `World::register_reflect::<T>()` 로 등록하면 F1 Inspector 패널에
/// 해당 컴포넌트의 필드가 자동으로 표시되어 실시간 편집이 가능하다.
///
/// # 예시
/// ```rust,no_run
/// # use engine::reflect::{Reflect, ReflectValue};
/// struct Hp(f32);
/// impl Reflect for Hp {
///     fn fields(&self) -> Vec<(&'static str, ReflectValue)> {
///         vec![("hp", ReflectValue::F32(self.0))]
///     }
///     fn set_field(&mut self, name: &str, val: ReflectValue) -> bool {
///         if name == "hp" { if let ReflectValue::F32(v) = val { self.0 = v; return true; } }
///         false
///     }
///     fn type_name(&self) -> &'static str { "Hp" }
/// }
/// ```
pub trait Reflect {
    /// 현재 필드 이름·값 목록을 반환한다.
    fn fields(&self) -> Vec<(&'static str, ReflectValue)>;
    /// 이름으로 필드를 수정한다. 성공하면 `true` 반환.
    fn set_field(&mut self, name: &str, val: ReflectValue) -> bool;
    /// Inspector 표시용 타입 이름.
    fn type_name(&self) -> &'static str;
}
