use std::time::Duration;
use bevy::color::palettes::basic::{BLACK, BLUE};
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::{Deserialize, Serialize};
use bevy::text::FontStyle;
use bevy::time::common_conditions::on_timer;
use crate::components::Enemy;

//选择题
#[derive(Debug)]
pub struct Question {
    question: String,
    answer: String,
    options: Vec<String>,
}

#[derive(Component, Debug)]
pub struct QuestionOption(String);
//练习卷
#[derive(Debug, Resource)]
pub struct Exercise {
    question: Question,
}

#[derive(Debug)]
pub struct ExercisePlugin;
impl Plugin for ExercisePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_question.run_if(
            on_timer(Duration::from_secs(2))
        ));
    }
}

fn update_question(mut commands: Commands,
                   enemy_query: Query<Entity, (With<Enemy>, Without<QuestionOption>)>,
                  // exercise: Res<Exercise>
) {
    for enemy in enemy_query.iter() {
        println!("show question option on enemy");
        let question_option = commands.spawn((
            Text2d("A".to_string()),//默认字体不支持中文
            TextColor(BLACK.into()),
            TextFont {
                font_size: 50.0,
                .. Default::default()
            },
        )).id();
        let mut enemy = commands.entity(enemy);
        enemy.add_child(question_option);
        enemy.insert(QuestionOption("A".into()));
    }

}