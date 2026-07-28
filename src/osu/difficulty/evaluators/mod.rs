pub use self::{
    aim::{
        agility::AgilityEvaluator,
        flow_aim::FlowAimEvaluator,
        snap_aim::SnapAimEvaluator,
    },
    speed::{
        rhythm::RhythmEvaluator,
        speed::SpeedEvaluator,
    },
    flashlight::FlashlightEvaluator
};

mod aim;
mod speed;
mod flashlight;
