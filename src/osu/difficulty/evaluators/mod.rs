pub use self::{
    aim::{agility::AgilityEvaluator, flow_aim::FlowAimEvaluator, snap_aim::SnapAimEvaluator},
    flashlight::FlashlightEvaluator,
    reading::ReadingEvaluator,
    speed::{rhythm::RhythmEvaluator, speed::SpeedEvaluator},
};

mod aim;
mod flashlight;
mod reading;
mod speed;
