use crate::components::Enemy;
use crate::EnemyCount;
use bevy::color::palettes::basic::{BLACK, BLUE, GREEN, WHITE};
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::{Deserialize, Serialize};
use bevy::text::FontStyle;
use bevy::time::common_conditions::on_timer;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng, RngCore};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use bevy::audio::PlaybackMode;
use regex::Regex;

const OPTIONS: &str = "abcdefghijklmnopqrstuvwxyz";
//选择题
#[derive(Debug, Default)]
pub struct Question {
	question: String,
	word: String,
	answer: String,
	options: Vec<String>,
}
impl Question {
	pub fn from_str(question_str: &str) -> Self {
		let idx = question_str.find(":").unwrap_or(question_str.len() - 1);
		let re = Regex::new(r"[^a-zA-Z]").unwrap();
		let word = question_str[..idx].trim().to_lowercase();
		let mean = question_str[idx + 1..].trim().to_string();
		let mut rng = thread_rng();
		let answer = re.replace_all(&word, "").chars().choose(&mut rng).unwrap_or_default().to_string();
		let question = word.replacen(&answer, "[ ]", 1);
		let question = format!("{} :{}", question, mean);
		let mut options = OPTIONS.replace(&answer, "");
		let mut question_options = vec![];
		for _ in 0..4 {
			let letter = options.chars().choose(&mut rng).unwrap_or_default().to_string();
			options = options.replace(&letter, "");
			question_options.push(letter);
		}
		question_options.push(answer.clone());
		question_options.shuffle(&mut rng);
		Question {
			question,
			word,
			answer,
			options: question_options,
		}
	}
}
#[derive(Component, Debug)]
pub struct QuestionOption(pub(crate) String);
#[derive(Component, Debug)]
pub struct QuestionPanel(String);
//练习卷
#[derive(Debug, Resource, Default)]
pub struct Exercise {
	questions: Vec<Question>,
	current_index: usize,
}
#[derive(Component, Debug)]
pub struct WordAuidoPlayer(pub String);
impl Exercise {
	pub fn get(&self) -> Option<&Question> {
		self.questions.get(self.current_index)
	}
	pub fn check(&self, answer: &str) -> bool {
		if let Some(question) = self.get() {
			return question.answer.eq(answer);
		}
		false
	}
	pub fn get_questions(&self) -> &Vec<Question> {
		&self.questions
	}
	pub fn next(&mut self) {
		self.current_index += 1;
	}

	pub fn from_str(question_str: &str) -> Self {
		let questions = question_str
			.lines()
			.map(|word| word.trim())
			.map(|word| Question::from_str(word))
			.collect::<Vec<Question>>();

		Self {
			questions,
			..Default::default()
		}
	}
}

#[derive(Debug)]
pub struct ExercisePlugin;
impl Plugin for ExercisePlugin {
	fn build(&self, app: &mut App) {
		let text_content =
			fs::read_to_string("assets/exercise/unit3.txt").expect("Failed to read file");

		app.insert_resource(AudioRes(HashMap::new()))
			.insert_resource(Exercise::from_str(&text_content))
			.add_systems(Startup, setup)
			.add_systems(Update, update_question.run_if(on_timer(Duration::from_secs(2))));
	}
}

#[derive(Resource)]
struct AudioRes(pub HashMap<String, Handle<AudioSource>>);
fn setup(
	mut commands: Commands,
	exercise: Res<Exercise>,
	mut audio_res: ResMut<AudioRes>,
	asset_server: Res<AssetServer>,
) {
	exercise.get_questions().iter().for_each(|question| {
		let path = format!("audios/{}_1.mp3", &question.word.replace(' ', "_"));
		let audio_handle: Handle<AudioSource> = asset_server.load(path);
		audio_res.0.insert(question.word.clone(), audio_handle);
	});

	commands.spawn((
		Text2d::new("Question1".to_string()),
		TextColor(WHITE.into()),
		TextFont {
			font_size: 50.0,
			font: asset_server.load("fonts/fangsong.ttf"),
			..Default::default()
		},
		Transform::from_xyz(0.0, 10.0, 99.0),
		QuestionPanel("Question1".to_string()),
	));
}
fn update_question(
	mut commands: Commands,
	enemy_query: Query<Entity, (With<Enemy>, Without<QuestionOption>)>,
	mut question_panel: Query<(Entity, &mut Text2d), With<QuestionPanel>>,
	enemy_count: Res<EnemyCount>,
	exercise: Res<Exercise>,
	asset_server: Res<AssetServer>,
	audios_res: Res<AudioRes>,
	audio_player: Query<Entity, With<AudioPlayer>>,
) {
	if exercise.get().is_none() {
		return;
	}
	let question = exercise.get().unwrap();
	if let Ok(mut question_panel) = question_panel.get_single_mut() {
		if(question_panel.1 .0 != question.question) {
			question_panel.1 .0 = question.question.clone();
			if let Some(audio) = audios_res.0.get(&question.word) {
				commands.spawn((
					AudioPlayer(audio.clone()),
					PlaybackSettings {
						mode: PlaybackMode::Once,
						.. Default::default()
					},
					WordAuidoPlayer(question.word.clone()),
				));
			}
		}

	};
	let options = &question.options;

	let mut rng = rand::thread_rng();

	for enemy in enemy_query.iter() {
		let word = options.choose(&mut rng);
		println!("show question option: {:?} on enemy", word);
		let question_option_word = if (enemy_count.0 + 1) % 5 == 0 {
			&question.answer
		} else {
			options.choose(&mut rng).unwrap_or(&question.answer)
		};
		let question_option = commands
			.spawn((
				Text2d(question_option_word.clone()), //默认字体不支持中文
				TextColor(BLACK.into()),
				TextFont {
					font_size: 40.0,
					font: asset_server.load("fonts/fangsong.ttf"),
					..Default::default()
				},
				Transform {
					translation: Vec3::new(0.0, 12.0, 99.),
					scale: Vec3::new(1., 0.8, 1.),
					..Default::default()
				},
				//Transform::from_xyz(0.0, 10.0, 99.0)
			))
			.id();
		let mut enemy = commands.entity(enemy);
		enemy.add_child(question_option);
		enemy.insert(QuestionOption(question_option_word.clone()));
	}

	//remove audio player that is not used
	audio_player.iter().for_each(|entity| {
		commands.entity(entity).despawn();
	})
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::fs::{File};
	use std::io::copy;
    use std::path::Path;
    use crate::exercise::{Exercise, Question};
	use reqwest::blocking::get;
	use crate::exercise;

	#[test]
	fn test_question_from_str() {
		println!("{:?}", Question::from_str("hello"));
	}
	#[test]
	fn test_exercise_from_str() {
		println!(
			"{:?}",
			Exercise::from_str(
				r#"hello:哈喽
        world:世界"#
			)
		);
	}

    fn download_mp3(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Send a GET request to the URL
        let mut response = get(url)?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(format!("Failed to download: HTTP {}", response.status()).into());
        }

        // Create the output directory if it doesn't exist
        if let Some(parent) = Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the output file
        let mut file = File::create(output_path)?;

        // Copy the response content to the file
        copy(&mut response, &mut file)?;

        println!("MP3 downloaded successfully to {}", output_path);
        Ok(())
    }

    #[test]
    fn test_download_mp3() {
		let text_content =
			fs::read_to_string("assets/exercise/unit3.txt").expect("Failed to read file");
		let exercise = Exercise::from_str(&text_content);
		exercise.questions.iter().for_each(|question| {
			let word = &question.word;
			let t = 1;
			let url = format!("https://dict.youdao.com/dictvoice?audio={}&type=1", word.trim().to_lowercase());
			let output_path = format!("assets/audios/{}_{}.mp3", word.replace(' ', "_"), t);
			if let Ok(_) = download_mp3(&url, &output_path) {
				println!("Downloaded MP3 {} successfully", word);
			}
		});
    }

}
