macro_rules! define_skill {
    // Entry point without `new` function
    (
        $( #[$meta:meta] )*
        $vis:vis struct $skill:ident: $trait:ident => $objects:ty[$object:ty] {
            $( $field_name:ident: $field_type:ty $( = $field_default:expr )?, )*
        }
    ) => {
        define_skill! {
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
        define_skill! {
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
        define_skill! {
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
        define_skill! {
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

    // Extend `StrainDecaySkill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields StrainDecaySkill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_skill! {
            @$trait $objects[$object]
            extend_fields StrainSkill
            fields {
                $( $fields )*
                strain_decay_skill_current_strain f64 = 0.0, // <-
            }
            $( $rest )*
        }
    };

    // Extend `StrainSkill`'s fields
    (
        @$trait:ident $objects:ty[$object:ty]
        extend_fields StrainSkill // <-
        fields { $( $fields:tt )* }
        $( $rest:tt )*
    ) => {
        define_skill! {
            @$trait $objects[$object]
            fields {
                $( $fields )*
                strain_skill_current_section_peak f64 = 0.0, // <-
                strain_skill_current_section_end f64 = 0.0,  // <-
                strain_skill_strain_peaks crate::util::strains_vec::StrainsVec
                    = crate::util::strains_vec::StrainsVec::with_capacity(256), // <-
                // TODO: use `StrainsVec`?
                strain_skill_object_strains Vec<f64> = Vec::with_capacity(256), // <-
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
        define_skill! {
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
        define_skill! {
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
        define_skill! {
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
                any::difficulty::{
                    object::{IDifficultyObject, IDifficultyObjects, HasStartTime},
                    skills::{StrainSkill, StrainDecaySkill},
                },
                util::strains_vec::StrainsVec,
            };

            define_skill!( @impl $trait $name $objects[$object] );
        };
    };

    // Implement `StrainSkill` trait
    ( @impl StrainSkill $name:ident $objects:ty[$object:ty] ) => {
        impl StrainSkill for $name {
            type DifficultyObject<'a> = $object;
            type DifficultyObjects<'a> = $objects;

            fn process<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                let section_length = f64::from(Self::SECTION_LENGTH);

                // * The first object doesn't generate a strain, so we begin with an incremented section end
                if curr.idx == 0 {
                    self.strain_skill_current_section_end =
                        f64::ceil(curr.start_time / section_length) * section_length;
                }

                while curr.start_time > self.strain_skill_current_section_end {
                    self.save_current_peak();
                    self.start_new_section_from(
                        self.strain_skill_current_section_end,
                        curr,
                        objects
                    );
                    self.strain_skill_current_section_end += section_length;
                }

                let strain = self.strain_value_at(curr, objects);
                self.strain_skill_current_section_peak
                    = f64::max(strain, self.strain_skill_current_section_peak);

                // * Store the strain value for the object
                self.strain_skill_object_strains.push(strain);
            }

            fn object_strains(&self) -> &[f64] {
                &self.strain_skill_object_strains
            }

            fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
                crate::any::difficulty::skills::count_top_weighted_strains(
                    &self.strain_skill_object_strains,
                    difficulty_value,
                )
            }

            fn save_current_peak(&mut self) {
                self.strain_skill_strain_peaks.push(self.strain_skill_current_section_peak);
            }

            fn start_new_section_from<'a>(
                &mut self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) {
                self.strain_skill_current_section_peak
                    = self.calculate_initial_strain(time, curr, objects);
            }

            fn into_current_strain_peaks(self) -> StrainsVec {
                Self::get_current_strain_peaks(
                    self.strain_skill_strain_peaks,
                    self.strain_skill_current_section_peak,
                )
            }

            fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
                crate::any::difficulty::skills::difficulty_value(
                    current_strain_peaks,
                    Self::DECAY_WEIGHT,
                )
            }

            fn into_difficulty_value(self) -> f64 {
                Self::difficulty_value(
                    Self::get_current_strain_peaks(
                        self.strain_skill_strain_peaks,
                        self.strain_skill_current_section_peak,
                    )
                )
            }

            fn cloned_difficulty_value(&self) -> f64 {
                Self::difficulty_value(
                    Self::get_current_strain_peaks(
                        self.strain_skill_strain_peaks.clone(),
                        self.strain_skill_current_section_peak,
                    )
                )
            }
        }
    };

    // Implement `StrainDecaySkill` and `StrainSkill` traits
    ( @impl StrainDecaySkill $name:ident $objects:ty[$object:ty] ) => {
        define_skill!( @impl StrainSkill $name $objects[$object] );

        impl StrainDecaySkill for $name {
            fn calculate_initial_strain<'a>(
                &self,
                time: f64,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                let prev_start_time = curr
                    .previous(0, objects)
                    .map_or(0.0, HasStartTime::start_time);

                self.strain_decay_skill_current_strain
                    * Self::strain_decay(time - prev_start_time)
            }

            fn strain_value_at<'a>(
                &mut self,
                curr: &Self::DifficultyObject<'a>,
                objects: &Self::DifficultyObjects<'a>,
            ) -> f64 {
                self.strain_decay_skill_current_strain
                    *= Self::strain_decay(curr.delta_time);
                self.strain_decay_skill_current_strain
                    += self.strain_value_of(curr, objects) * Self::SKILL_MULTIPLIER;

                self.strain_decay_skill_current_strain
            }

            fn strain_decay(ms: f64) -> f64 {
                crate::any::difficulty::skills::strain_decay(ms, Self::STRAIN_DECAY_BASE)
            }
        }
    };
}
