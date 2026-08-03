macro_rules! define_new_skill {
    // Entry point without `new` function
    (
        $( #[$meta:meta] )*
        $vis:vis struct $skill:ident: $trait:ident => $objects:ty[$object:ty] {
            $( $field_name:ident: $field_type:ty $( = $field_default:expr )?, )*
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields $trait
            fields { $( $field_name $field_type $( = $field_default )?, )* }
            struct { $( #[$meta] )* $vis $skill }
            new {
                setup {}
                args {}
                assigns {}
            }
        }
    };

    // Entry point with `new` function
    (
        $( #[$meta:meta] )*
        $vis:vis struct $skill:ident: $trait:ident => $objects:ty[$object:ty] {
            $( $field_name:ident: $field_type:ty $( = $field_default:expr )?, )*
        }

        $_new_vis:vis fn new( $( $arg_name:ident: $arg_type:ty ),* ) -> Self {
            $( $body:tt )*
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields $trait
            fields { $( $field_name $field_type |, )* }
            struct { $( #[$meta] )* $vis $skill }
            new {
                setup_body { $( $body )* }
                setup {}
                args { $( $arg_name $arg_type, )* }
            }
        }
    };

    // Processing `new` function: Found the final `Self` return value
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields $extend_fields:ident
        fields { $( $fields:tt )* }
        struct { $( $struct:tt )* }
        new {
            setup_body {
                Self { $( $assign_name:ident: $assign_expr:expr, )* } // <-
            }
            setup { $( $setup:tt )* }
            args { $( $args:tt )* }
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields $trait
            fields { $( $fields )* }
            struct { $( $struct )* }
            new {
                setup { $( $setup )* }
                args { $( $args )* }
                assigns { $( $assign_name $assign_expr, )* } // <-
            }
        }
    };

    // Processing `new` function: Pop next statement from body and continue
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields $extend_fields:ident
        fields { $( $fields:tt )* }
        struct { $( $struct:tt )* }
        new {
            setup_body {
                $stmt:stmt; // <-
                $( $rest:tt )+
            }
            setup { $( $setup:tt )* }
            args { $( $args:tt )* }
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields $trait
            fields { $( $fields )* }
            struct { $( $struct )* }
            new {
                setup_body { $( $rest )* }   // <-
                setup { $( $setup )* $stmt } // <-
                args { $( $args )* }
            }
        }
    };

    // Extend `Skill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields Skill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            fields {
                $( $fields )*
                skill_object_difficulties Vec<f64> = Vec::with_capacity(256), // <-
            }
            $( $rest )*
        }
    };

    // Extend `VariableLengthStrainSkill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields VariableLengthStrainSkill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields Skill
            fields {
                $( $fields )*
                skill_current_section_peak f64 = 0.0, // <-
                skill_current_section_end f64 = 0.0,  // <-
                skill_current_section_begin f64 = 0.0,  // <-
                skill_total_length f64 = 0.0, // <-
                skill_strain_peaks Vec<crate::any::difficulty::skills_new::variable_length_strain_skill::StrainPeak> = Vec::with_capacity(256), // <-
                skill_queued_strains Vec<(f64, f64)> = Vec::with_capacity(256), // <-
            }
            $( $rest )*
        }
    };

    // Extend `NewStrainSkill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields NewStrainSkill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields Skill
            fields {
                $( $fields )*
                skill_current_section_peak f64 = 0.0, // <-
                skill_current_section_end f64 = 0.0,  // <-
                skill_strain_peaks Vec<f64> = Vec::with_capacity(256), // <-
            }
            $( $rest )*
        }
    };

    // Extend `HarmonicSkill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields HarmonicSkill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            extend_fields Skill
            fields {
                $( $fields )*
                skill_object_weight_sum f64 = 0.0, // <-
            }
            $( $rest )*
        }
    };

    // Parse field without default
    (
        @$trait:ident $objects:ty[$object:ty]
        fields {
            $field_name:ident $field_type:ty, // <-
            $( $fields:tt )*
        }
        struct { $( $struct:tt )* }
        new {
            setup { $( $setup:tt )* }
            args { $( $args:tt )* }
            assigns { $( $assigns:tt )* }
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            fields { $( $fields )* }
            struct { $( $struct )* $field_name $field_type, } // <-
            new {
                setup { $( $setup )* }
                args { $( $args )* $field_name $field_type, } // <-
                assigns { $( $assigns )* $field_name, }       // <-
            }
        }
    };

    // Parse field with default
    (
        @$trait:ident $objects:ty[$object:ty]
        fields {
            $field_name:ident $field_type:ty = $field_default:expr, // <-
            $( $fields:tt )*
        }
        struct { $( $struct:tt )* }
        new {
            setup { $( $setup:tt )* }
            args { $( $args:tt )* }
            assigns { $( $assigns:tt )* }
        }
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            fields { $( $fields )* }
            struct { $( $struct )* $field_name $field_type, } // <-
            new {
                setup { $( $setup )* }
                args { $( $args )* }
                assigns { $( $assigns )* $field_name $field_default, } // <-
            }
        }
    };

    // Parse field with but skip for `new` function
    (
        @$trait:ident $objects:ty[$object:ty]
        fields {
            $field_name:ident $field_type:ty |, // <-
            $( $fields:tt )*
        }
        struct { $( $struct:tt )* }
        $( $rest:tt )*
    ) => {
        define_new_skill! {
            @$trait $objects[$object]
            fields { $( $fields )* }
            struct { $( $struct )* $field_name $field_type, } // <-
            $( $rest )*
        }
    };

    // Final output
    (
        @$trait:ident $objects:ty[$object:ty]
        fields {}
        struct {
            $( #[$meta:meta] )*
            $vis:vis $name:ident
            $( $field_name:ident $field_type:ty, )*
        }
        new {
            setup { $( $setup:stmt )* }
            args { $( $arg_name:ident $arg_type:ty, )* }
            assigns { $( $assign_name:ident $( $assign_expr:expr )?, )* }
        }
    ) => {
        $( #[$meta] )*
        $vis struct $name {
            $( $field_name: $field_type, )*
        }

        impl $name {
            $vis fn new(
                $( $arg_name: $arg_type, )*
            ) -> Self {
                $( $setup )*

                Self {
                    $( $assign_name $( : $assign_expr )?, )*
                }
            }
        }

        const _: () = {
            #[expect(unused_imports, reason = "fine for macros")]
            use crate::{
                util::traits::IEnumerable,
                any::difficulty::{
                    object::{IDifficultyObject, IDifficultyObjects, HasStartTime},
                    skills_new::{
                        skill::Skill,
                        strain_skill::NewStrainSkill,
                        variable_length_strain_skill::VariableLengthStrainSkill,
                        harmonic_skill::HarmonicSkill,
                    },
                },
            };

            define_new_skill!( @impl $trait $name $objects[$object] );
        };
    };

    // Implement `Skill` trait
    ( @impl Skill $name:ident $objects:ty[$object:ty] ) => {
        impl Skill for $name {
            type DifficultyObject<'a> = $object;
            type DifficultyObjects<'a> = $objects;

            fn process<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                let difficulty_value = self.process_internal(curr, objects);
                self.skill_object_difficulties.push(difficulty_value);
            }

            fn get_object_difficulties(&self) -> &[f64] {
                &self.skill_object_difficulties
            }
        }
    };

    // Implement `NewStrainSkill` trait
    ( @impl NewStrainSkill $name:ident $objects:ty[$object:ty] ) => {
        define_new_skill!( @impl Skill $name $objects[$object] );

        impl NewStrainSkill for $name {
            fn process_internal<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                let section_length = f64::from(Self::SECTION_LENGTH);

                // * The first object doesn't generate a strain, so we begin with an incremented section end
                if curr.idx == 0 {
                    self.skill_current_section_end =
                        f64::ceil(curr.start_time / section_length) * section_length;
                }

                while curr.start_time > self.skill_current_section_end {
                    self.save_current_peak();
                    self.start_new_section_from(
                        self.skill_current_section_end,
                        curr,
                        objects
                    );
                    self.skill_current_section_end += section_length;
                }

                let strain = self.strain_value_at(curr, objects);
                self.skill_current_section_peak
                    = f64::max(strain, self.skill_current_section_peak);

                strain
            }

            #[expect(unused_variables, reason = "placeholder")]
            fn strain_value_at<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                todo!()
            }

            fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
                crate::any::difficulty::skills_new::count_top_weighted_strains(
                    &self.skill_object_difficulties,
                    difficulty_value,
                    Self::DECAY_WEIGHT,
                )
            }

            fn save_current_peak(&mut self) {
                self.skill_strain_peaks.push(self.skill_current_section_peak);
            }

            fn start_new_section_from<'a>(
                &mut self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                self.skill_current_section_peak
                    = self.calculate_initial_strain(time, curr, objects);
            }

            #[expect(unused_variables, reason = "placeholder")]
            fn calculate_initial_strain<'a>(
                &self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                todo!()
            }

            fn into_current_strain_peaks(self) -> Vec<f64> {
                Self::get_current_strain_peaks(
                    self.skill_strain_peaks,
                    self.skill_current_section_peak,
                )
            }

            fn difficulty_value(current_strain_peaks: Vec<f64>) -> f64 {
                crate::any::difficulty::skills_new::strain_skill::strain_skill_difficulty_value(
                    current_strain_peaks,
                    Self::DECAY_WEIGHT,
                )
            }

            fn into_difficulty_value(self) -> f64 {
                Self::difficulty_value(
                    Self::get_current_strain_peaks(
                        self.skill_strain_peaks,
                        self.skill_current_section_peak,
                    )
                )
            }

            fn cloned_difficulty_value(&self) -> f64 {
                Self::difficulty_value(
                    Self::get_current_strain_peaks(
                        self.skill_strain_peaks.clone(),
                        self.skill_current_section_peak,
                    )
                )
            }
        }
    };

    // Implement `VariableLengthStrainSkill` trait
    ( @impl VariableLengthStrainSkill $name:ident $objects:ty[$object:ty] ) => {
        define_new_skill!( @impl Skill $name $objects[$object] );

        impl VariableLengthStrainSkill for $name {
            fn process_internal<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                // * If we're on the first object, set up the first section to end `MaxSectionLength` after it.
                if curr.idx == 0 {
                    self.skill_current_section_begin = curr.start_time;
                    self.skill_current_section_end
                        = self.skill_current_section_begin + Self::MAX_SECTION_LENGTH;

                    // * No work is required for first object after calculating difficulty
                    self.skill_current_section_peak
                        = self.strain_value_at(curr, objects);

                    return self.skill_current_section_peak;
                }

                self.backfill_peaks(curr, objects);

                let current_strain = self.strain_value_at(curr, objects);

                // * If the current strain is larger than the current peak, begin a new peak
                // * Otherwise, add the current strain to the queue
                if current_strain > self.skill_current_section_peak {
                    // * Clear the queue since none of the strains inside of it will be contributing to the difficulty.
                    self.skill_queued_strains.clear();

                    // * End the current section with the new peak
                    self.save_current_peak(curr.start_time - self.skill_current_section_begin);

                    // * Set up the new section to start at the current object with the current strain
                    self.skill_current_section_begin = curr.start_time;
                    self.skill_current_section_end = self.skill_current_section_begin + Self::MAX_SECTION_LENGTH;
                    self.skill_current_section_peak = current_strain;
                } else {
                    // * Empty the queue of smaller elements as they won't be relevant to difficulty
                    while self.skill_queued_strains.last().filter(|(strain_value, _)| strain_value < &current_strain).is_some() {
                        self.skill_queued_strains.pop();
                    }
                    self.skill_queued_strains.push((current_strain, curr.start_time));
                }

                current_strain
            }

            #[expect(unused_variables, reason = "placeholder")]
            fn strain_value_at<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                todo!()
            }

            fn backfill_peaks<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                // * If the current object starts after the current section ends
                // * then we want to start a new section without any harsh drop-off.
                // * If we have previous strains that influence the current difficulty we will prioritise those first.
                // * Otherwise, start with the current object's initial strain.
                while curr.start_time > self.skill_current_section_end {
                    // * Save the current peak, marking the end of the section.
                    self.save_current_peak(self.skill_current_section_end - self.skill_current_section_begin);
                    self.skill_current_section_begin = self.skill_current_section_end;

                    // * If we have any strains queued, then we will use those until the object falls into the new section.
                    if !self.skill_queued_strains.is_empty() {
                        let (strain, start_time) = self.skill_queued_strains.remove(0);

                        // * We want the section to end `MaxSectionLength` after the strain we're using as an influence.
                        // * This effectively means the queued strain will exist in its own section if the gap between the queued strain and current object is large enough.
                        // * This is required to make sure there's no harsh difficulty difference between 2 sections if there was a large gap.
                        self.skill_current_section_end = start_time + Self::MAX_SECTION_LENGTH;
                        self.start_new_section_from(self.skill_current_section_begin, curr, objects);

                        // * If the current object's peak was higher, we don't want to override it with a lower strain.
                        // * Only use the queued strain if it contributes more difficulty.
                        self.skill_current_section_peak = self.skill_current_section_peak.max(strain);

                    // * If the queue is empty then we should start the section from the current object instead.
                    // * The queue can be empty if we're starting off of the back of a new peak, or if we drained through all the queue
                    // * and the current object is still later than the section end.
                    } else {
                        // * We don't have any prior strains to take as a reference, so end the new section `MaxSectionLength` after it starts.
                        self.skill_current_section_end = self.skill_current_section_begin + Self::MAX_SECTION_LENGTH;
                        self.start_new_section_from(self.skill_current_section_begin, curr, objects);
                    }
                }
            }

            fn save_current_peak(&mut self, section_length: f64) {
                let peak = crate::any::difficulty::skills_new::variable_length_strain_skill::StrainPeak::new(self.skill_current_section_peak, section_length);

                self.skill_strain_peaks.cs_add_in_place(peak);
                self.skill_total_length += section_length;

                // * Remove from the back of our strain peaks if there's any which are too deep to contribute to difficulty.
                // * `maxStoredLength` dictates for us how many sections will preserve at least 99.999% of the difficulty value.
                while self.skill_total_length > Self::MAX_STORED_LENGTH * Self::MAX_SECTION_LENGTH {
                    let Some(strain_peak) = self.skill_strain_peaks.pop() else {
                        break;
                    };
                    self.skill_total_length -= strain_peak.section_length;
                }
            }

            fn start_new_section_from<'a>(
                &mut self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                // * The maximum strain of the new section is not zero by default
                // * This means we need to capture the strain level at the beginning of the new section, and use that as the initial peak level.
                self.skill_current_section_peak = self.calculate_initial_strain(time, curr, objects);
            }

            #[expect(unused_variables, reason = "placeholder")]
            fn calculate_initial_strain<'a>(
                &self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                todo!()
            }

            fn into_current_strain_peaks(self) -> Vec<crate::any::difficulty::skills_new::variable_length_strain_skill::StrainPeak> {
                self.skill_strain_peaks
            }

            fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
                crate::any::difficulty::skills_new::count_top_weighted_strains(
                    &self.skill_object_difficulties,
                    difficulty_value,
                    Self::DECAY_WEIGHT,
                )
            }
        }
    };

    // Implement `HarmonicSkill` trait
    ( @impl HarmonicSkill $name:ident $objects:ty[$object:ty] ) => {
        define_new_skill!( @impl Skill $name $objects[$object] );

        impl HarmonicSkill for $name {
            fn object_difficulty_of<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                self.object_difficulty_of(curr, objects)
            }

            fn into_transformed_difficulties(self) -> Vec<f64> {
                self.get_transformed_difficulties(self.get_object_difficulties().to_vec())
            }

            fn difficulty_value(
                transformed_object_difficulties: Vec<f64>,
                object_weight_sum: &mut f64,
            ) -> f64 {
                crate::any::difficulty::skills_new::harmonic_skill::harmonic_skill_difficulty_value(
                    &transformed_object_difficulties,
                    object_weight_sum,
                    Self::HARMONIC_SCALE,
                    Self::DECAY_EXPONENT,
                )
            }

            fn into_difficulty_value(mut self) -> f64 {
                Self::difficulty_value(
                    self.get_transformed_difficulties(self.get_object_difficulties().to_vec()),
                    &mut self.skill_object_weight_sum,
                )
            }

            fn cloned_difficulty_value(&mut self) -> f64 {
                Self::difficulty_value(
                    self.get_transformed_difficulties(self.skill_object_difficulties.clone()),
                    &mut self.skill_object_weight_sum,
                )
            }

            fn count_top_weighted_object_difficulties(&self, difficulty_value: f64) -> f64 {
                crate::any::difficulty::skills_new::count_top_weighted_object_difficulties(
                    difficulty_value,
                    &self.skill_object_difficulties,
                    self.skill_object_weight_sum,
                    0.88,
                    10.0,
                    Some(1.1)
                )
            }
        }
    };
}
