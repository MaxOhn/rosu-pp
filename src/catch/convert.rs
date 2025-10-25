use rosu_map::section::{general::GameMode, hit_objects::CurveBuffers};

use crate::{
    model::{
        beatmap::Beatmap,
        hit_object::{HitObject, HitObjectKind, HoldNote, Spinner},
        mods::Reflection,
    },
    util::{float_ext::FloatExt, random::osu::Random},
};

use super::{
    PLAYFIELD_WIDTH,
    attributes::ObjectCountBuilder,
    catcher::Catcher,
    object::{
        banana_shower::BananaShower,
        fruit::Fruit,
        juice_stream::{JuiceStream, JuiceStreamBufs, NestedJuiceStreamObjectKind},
        palpable::PalpableObject,
    },
};

const RNG_SEED: i32 = 1337;

pub const fn convert(map: &mut Beatmap) {
    map.mode = GameMode::Catch;
    map.is_convert = true;
}

pub fn convert_objects(
    map: &Beatmap,
    count: &mut ObjectCountBuilder,
    reflection: Reflection,
    hr_offsets: bool,
    cs: f32,
) -> Vec<PalpableObject> {
    // mean=686.54 | median=501
    let mut palpable_objects = Vec::with_capacity(512);

    let mut bufs = JuiceStreamBufs {
        curve: CurveBuffers::default(),
        // mean=31.65 | median=16
        nested_objects: Vec::with_capacity(16),
        // mean=5.21 | median=4
        ticks: Vec::new(),
    };

    let mut rng = Random::new(RNG_SEED);
    let mut last_pos = None;
    let mut last_start_time = 0.0;

    for h in map.hit_objects.iter() {
        let mut new_objects = convert_object(h, map, count, &mut bufs);

        apply_pos_offset(
            &mut new_objects,
            hr_offsets,
            &mut last_pos,
            &mut last_start_time,
            &mut rng,
        );

        palpable_objects.extend(new_objects);
    }

    if let Reflection::Horizontal = reflection {
        for h in palpable_objects.iter_mut() {
            h.x = PLAYFIELD_WIDTH - h.x;
            h.x_offset = -h.x_offset;
        }
    }

    palpable_objects.sort_by(|a, b| a.start_time.total_cmp(&b.start_time));
    initialize_hyper_dash(cs, &mut palpable_objects);

    palpable_objects
}

fn convert_object<'a>(
    h: &'a HitObject,
    map: &Beatmap,
    count: &mut ObjectCountBuilder,
    bufs: &'a mut JuiceStreamBufs,
) -> ObjectIter<'a> {
    let state = match h.kind {
        HitObjectKind::Circle => ObjectIterState::Fruit(Some(Fruit::new(count))),
        HitObjectKind::Slider(ref slider) => {
            let effective_x = h.pos.x.clamp(0.0, PLAYFIELD_WIDTH);
            let stream = JuiceStream::new(effective_x, h.start_time, slider, map, count, bufs);

            ObjectIterState::JuiceStream(stream)
        }
        HitObjectKind::Spinner(Spinner { duration })
        | HitObjectKind::Hold(HoldNote { duration }) => {
            ObjectIterState::BananaShower(BananaShower::new(h.start_time, h.start_time + duration))
        }
    };

    ObjectIter {
        x: h.pos.x,
        start_time: h.start_time,
        state,
    }
}

struct ObjectIter<'a> {
    x: f32,
    start_time: f64,
    state: ObjectIterState<'a>,
}

enum ObjectIterState<'a> {
    Fruit(Option<Fruit>),
    JuiceStream(JuiceStream<'a>),
    BananaShower(BananaShower),
}

impl Iterator for ObjectIter<'_> {
    type Item = PalpableObject;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            ObjectIterState::Fruit(ref mut fruit) => fruit
                .take()
                .map(|fruit| PalpableObject::new(self.x, fruit.x_offset, self.start_time)),
            ObjectIterState::JuiceStream(ref mut stream) => stream
                .nested_objects
                .find(|nested| !matches!(nested.kind, NestedJuiceStreamObjectKind::TinyDroplet))
                .map(|nested| PalpableObject::new(nested.pos, 0.0, nested.start_time)),
            ObjectIterState::BananaShower(_) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for ObjectIter<'_> {
    fn len(&self) -> usize {
        match self.state {
            ObjectIterState::Fruit(ref fruit) => usize::from(fruit.is_some()),
            ObjectIterState::JuiceStream(ref stream) => stream.nested_objects.len(),
            ObjectIterState::BananaShower(_) => 0,
        }
    }
}

fn apply_pos_offset(
    hit_object: &mut ObjectIter<'_>,
    hr_offsets: bool,
    last_pos: &mut Option<f32>,
    last_start_time: &mut f64,
    rng: &mut Random,
) {
    match hit_object.state {
        ObjectIterState::Fruit(Some(ref mut fruit)) => {
            if hr_offsets {
                apply_hr_offset(
                    hit_object.x,
                    &mut fruit.x_offset,
                    hit_object.start_time,
                    last_pos,
                    last_start_time,
                    rng,
                );
            }
        }
        ObjectIterState::JuiceStream(ref stream) => {
            let pos = hit_object.x
                + stream
                    .control_points
                    .last()
                    .map_or(0.0, |control_point| control_point.pos.x);

            *last_pos = Some(pos);
            *last_start_time = hit_object.start_time;

            for nested in stream.nested_objects.as_slice() {
                if let NestedJuiceStreamObjectKind::Droplet
                | NestedJuiceStreamObjectKind::TinyDroplet = nested.kind
                {
                    let _ = rng.next_int();
                }
            }
        }
        ObjectIterState::BananaShower(ref shower) => {
            for _ in 0..shower.n_bananas {
                let _ = rng.next_double();
                let _ = rng.next_int();
                let _ = rng.next_int();
                let _ = rng.next_int();
            }
        }
        ObjectIterState::Fruit(None) => unreachable!(),
    }
}

fn apply_hr_offset(
    x: f32,
    x_offset: &mut f32,
    start_time: f64,
    last_pos: &mut Option<f32>,
    last_start_time: &mut f64,
    rng: &mut Random,
) {
    let mut offset_pos = x;

    let last_pos = match last_pos {
        Some(pos) if pos.abs() >= f32::EPSILON => pos,
        Some(_) | None => {
            *last_pos = Some(offset_pos);
            *last_start_time = start_time;

            return;
        }
    };

    let pos_diff = offset_pos - *last_pos;
    let time_diff = (start_time - *last_start_time) as i32;

    if time_diff > 1000 {
        *last_pos = offset_pos;
        *last_start_time = start_time;

        return;
    }

    if pos_diff.eq(0.0) {
        apply_random_offset(&mut offset_pos, f64::from(time_diff) / 4.0, rng);
        *x_offset = offset_pos - x;

        return;
    }

    if pos_diff.abs() < (time_diff / 3) as f32 {
        apply_offset(&mut offset_pos, pos_diff);
    }

    *x_offset = offset_pos - x;

    *last_pos = offset_pos;
    *last_start_time = start_time;
}

fn apply_random_offset(pos: &mut f32, max_offset: f64, rng: &mut Random) {
    let right = rng.next_bool();
    let rand = (rng.next_double_range(0.0, max_offset.max(0.0)) as f32).min(20.0);

    if right {
        if *pos + rand <= PLAYFIELD_WIDTH {
            *pos += rand;
        } else {
            *pos -= rand;
        }
    } else if *pos - rand >= 0.0 {
        *pos -= rand;
    } else {
        *pos += rand;
    }
}

fn apply_offset(pos: &mut f32, amount: f32) {
    if amount > 0.0 {
        if *pos + amount < PLAYFIELD_WIDTH {
            *pos += amount;
        }
    } else if *pos + amount > 0.0 {
        *pos += amount;
    }
}

fn initialize_hyper_dash(cs: f32, palpable_objects: &mut [PalpableObject]) {
    let mut half_catcher_width = f64::from(Catcher::calculate_catch_width(cs) / 2.0);
    half_catcher_width /= f64::from(Catcher::ALLOWED_CATCH_RANGE);

    let mut last_dir = 0;
    let mut last_excess = half_catcher_width;

    for i in 0..palpable_objects.len().saturating_sub(1) {
        let next = &palpable_objects[i + 1];
        let curr = &palpable_objects[i];

        let this_dir = if next.effective_x() > curr.effective_x() {
            1
        } else {
            -1
        };

        // * Int truncation added to match osu!stable.
        let time_to_next = f64::from(
            (next.start_time as i32 - curr.start_time as i32) as f32 - 1000.0 / 60.0 / 4.0,
        );

        let dist_to_next = f64::from((next.effective_x() - curr.effective_x()).abs())
            - if last_dir == this_dir {
                last_excess
            } else {
                half_catcher_width
            };

        let dist_to_hyper = (time_to_next * Catcher::BASE_SPEED - dist_to_next) as f32;

        let curr = &mut palpable_objects[i];

        if dist_to_hyper < 0.0 {
            curr.hyper_dash = true;
            last_excess = half_catcher_width;
        } else {
            curr.dist_to_hyper_dash = dist_to_hyper;
            last_excess = f64::from(dist_to_hyper).clamp(0.0, half_catcher_width);
        }

        last_dir = this_dir;
    }
}
