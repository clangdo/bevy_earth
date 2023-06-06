use bevy::{
    prelude::*,
    input::mouse::{MouseMotion, MouseWheel},
};
    
pub fn mouse_motion(mut motion_events: EventReader<MouseMotion>) -> Vec2 {
    let extract_delta = |event: &MouseMotion| event.delta;
    motion_events.iter().map(extract_delta).sum()
}

pub fn scrolling(mut scroll_events: EventReader<MouseWheel>) -> f32 {
    let extract_vertical_scroll = |event: &MouseWheel| event.y;
    scroll_events.iter().map(extract_vertical_scroll).sum()
}

pub fn window_dimensions(window: &Window) -> Vec2 {
    Vec2::new(window.resolution.width(), window.resolution.height())
}

pub fn ar_fov_normalize(mouse_motion: Vec2, dimensions: Vec2, projection: &Projection) -> Vec2 {
    let mut output = mouse_motion;
    if let Projection::Perspective(PerspectiveProjection{fov, aspect_ratio,..}) = projection {
        output *= Vec2::new(fov * aspect_ratio, *fov) / dimensions;
    }
    output
}
